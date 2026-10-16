import { afterEach, expect, test } from 'bun:test'
import { classifyPaymentFailure, startPaymentAttempt } from './paymentTelemetry'

const testGlobal = globalThis as unknown as { uni: typeof uni }
const previousUni = testGlobal.uni
const originalSetTimeout = globalThis.setTimeout
let timers: Array<() => void> = []
let requests: any[] = []
let token = 'account-a'
let dialogs = 0
let toasts = 0
function setup() {
  timers = []; requests = []; token = 'account-a'; dialogs = 0; toasts = 0
  globalThis.setTimeout = ((callback: () => void, ms: number) => {
    expect(ms).toBe(500); timers.push(callback); return 1
  }) as typeof setTimeout
  testGlobal.uni = {
    getStorageSync: () => token,
    getSystemInfoSync: () => ({ SDKVersion: '3.8', platform: 'android', system: 'Android 14', brand: 'brand', model: 'model', openid: 'private' }),
    getNetworkType: ({ success }: any) => success({ networkType: 'wifi' }),
    request: (options: any) => { requests.push(options); options.success({ statusCode: 204 }) },
    showModal: () => { dialogs += 1 },
    showToast: () => { toasts += 1 },
  } as unknown as typeof uni
}
afterEach(() => { testGlobal.uni = previousUni; globalThis.setTimeout = originalSetTimeout })

test('classifies exact cancellation predicate and preserves raw provider fields', () => {
  for (const failure of [{ errCode: -2, errMsg: 'provider' }, { errCode: -3, errMsg: 'CANCEL' }, { errMsg: '用户取消' }]) {
    expect(classifyPaymentFailure(failure)).toEqual({ event: 'pay_cancelled', error_code: failure.errCode === undefined ? undefined : String(failure.errCode), error_msg: failure.errMsg })
  }
  expect(classifyPaymentFailure({ errCode: -15007, errMsg: 'expired' })).toEqual({ event: 'pay_failed', error_code: '-15007', error_msg: 'expired' })
  expect(classifyPaymentFailure({})).toEqual({ event: 'pay_failed', error_code: undefined, error_msg: undefined })
})

test('batches within 500ms, binds order and only collects allowed detail', () => {
  setup()
  const attempt = startPaymentAttempt('membership', 'V7')
  expect(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(attempt.attemptId)).toBe(true)
  attempt.report('purchase_tapped'); attempt.report('order_create_requested')
  attempt.bindOrder('order-1')
  expect(requests.length).toBe(0); expect(timers.length).toBe(1)
  timers.shift()!()
  expect(requests.length).toBe(1)
  const payload = requests[0].data
  expect(payload.attempt_id).toBe(attempt.attemptId)
  expect(payload.order_no).toBe('order-1')
  expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'order_create_requested'])
  expect(payload.events[0].detail).toMatchObject({ platform: 'android', wx_lib_version: '3.8', network_type: 'wifi' })
  expect(typeof payload.events[0].detail.elapsed_ms).toBe('number')
  expect(typeof payload.events[0].detail.mini_version).toBe('string')
  expect(payload.events[0].detail.openid).toBe(undefined)
})

test('splits batches at ten events', () => {
  setup()
  const attempt = startPaymentAttempt('match_id_unlock', '583')
  for (let i = 0; i < 21; i++) attempt.report('purchase_tapped')
  timers.shift()!()
  expect(requests.map(request => request.data.events.length)).toEqual([10, 10, 1])
})

test('skips guests and discards old-account events before flush', () => {
  setup(); token = ''
  startPaymentAttempt('membership', 'V7').report('purchase_tapped')
  expect(timers.length).toBe(0)
  token = 'account-a'
  const attempt = startPaymentAttempt('membership', 'V7')
  attempt.report('purchase_tapped'); token = 'account-b'; timers.shift()!()
  attempt.report('pay_failed')
  expect(requests.length).toBe(0); expect(timers.length).toBe(0)
})

test('old backend 404/500 and network failure are silent and never reject report', async () => {
  setup()
  let calls = 0
  testGlobal.uni.request = ((options: any) => {
    calls += 1
    if (calls === 3) options.fail({ errMsg: 'request:fail timeout' })
    else options.success({ statusCode: calls === 1 ? 404 : 500, data: 'old backend' })
  }) as typeof uni.request
  for (let i = 0; i < 3; i++) {
    const attempt = startPaymentAttempt('reflux_subscription', 'monthly')
    attempt.report('purchase_tapped'); timers.shift()!()
  }
  await Promise.resolve(); await Promise.resolve()
  expect(calls).toBe(3)
  expect(dialogs).toBe(0)
  expect(toasts).toBe(0)
})

test('missing platform/storage APIs and throwing reporting transport never escape', () => {
  setup()
  testGlobal.uni = {} as typeof uni
  startPaymentAttempt('membership', 'V7').report('purchase_tapped')
  setup()
  testGlobal.uni.request = (() => { throw new Error('transport broke') }) as typeof uni.request
  const attempt = startPaymentAttempt('membership', 'V7')
  attempt.report('purchase_tapped'); timers.shift()!()
})


test('large multi-byte metadata and raw errors respect 1KB detail and 8KB request limits', () => {
  setup()
  testGlobal.uni.getSystemInfoSync = (() => ({ SDKVersion: '👀'.repeat(100), platform: 'android', system: '👀'.repeat(100), brand: '👀'.repeat(100), model: '👀'.repeat(100) })) as typeof uni.getSystemInfoSync
  testGlobal.uni.getNetworkType = ((options: any) => options.success({ networkType: '👀'.repeat(1000) })) as typeof uni.getNetworkType
  const attempt = startPaymentAttempt('membership', 'V7')
  for (let i = 0; i < 10; i++) attempt.report('pay_failed', { error_code: '-15007', error_msg: '👀'.repeat(500), detail: { http_status: 503, token: 'private' } as any })
  timers.shift()!()
  expect(requests.reduce((count, request) => count + request.data.events.length, 0)).toBe(10)
  for (const request of requests) {
    expect(new TextEncoder().encode(JSON.stringify(request.data)).length <= 8192).toBe(true)
    for (const event of request.data.events) {
      expect(new TextEncoder().encode(JSON.stringify(event.detail)).length <= 1024).toBe(true)
      expect(Array.from(event.error_msg).length).toBe(300)
      expect(event.detail.token).toBe(undefined)
    }
  }
})
