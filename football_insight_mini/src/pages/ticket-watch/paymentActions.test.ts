import { afterEach, expect, test } from 'bun:test'
import { requestTicketWatchPayment } from './paymentActions'

const testGlobal = globalThis as unknown as { uni: typeof uni }
const previousUni = testGlobal.uni
afterEach(() => { testGlobal.uni = previousUni })

test('digital purchases reject ordinary payment parameters without charging', async () => {
  let ordinaryCalls = 0
  testGlobal.uni = {
    requestPayment: (options: { success: () => void }) => { ordinaryCalls += 1; options.success() },
  } as unknown as typeof uni
  const error = await requestTicketWatchPayment({ timeStamp: '1', nonceStr: 'n', package: 'p', signType: 'MD5', paySign: 's' }).catch((error: Error) => error)
  expect(error instanceof Error).toBe(true)
  expect((error as Error).message.includes('支付配置已更新')).toBe(true)
  expect(ordinaryCalls).toBe(0)
})

const virtualParams = { mode: 'short_series_goods' as const, signData: '{}', paySig: 'pay', signature: 'user' }

test('virtual payment calls the platform with signed data and never ordinary payment', async () => {
  let calls = 0
  testGlobal.uni = {
    canIUse: () => true,
    requestVirtualPayment: (options: { mode: string; signData: string; success: () => void }) => {
      calls += 1
      expect(options.mode).toBe('short_series_goods')
      expect(options.signData).toBe('{}')
      options.success()
    },
    requestPayment: () => { throw new Error('ordinary payment must not be called') },
  } as unknown as typeof uni
  await requestTicketWatchPayment(virtualParams)
  expect(calls).toBe(1)
})

test('a provider signature failure is not reported as a user cancellation', async () => {
  testGlobal.uni = {
    canIUse: () => true,
    requestVirtualPayment: (options: { fail: (error: unknown) => void }) => options.fail({ errCode: -15006, errMsg: 'requestVirtualPayment:fail' }),
  } as unknown as typeof uni
  const error = await requestTicketWatchPayment(virtualParams).catch((error: Error) => error)
  expect((error as Error).message).toBe('支付未完成，请稍后重试')
})

test('unsupported clients reject before calling any payment API', async () => {
  testGlobal.uni = { canIUse: () => false } as unknown as typeof uni
  const error = await requestTicketWatchPayment(virtualParams).catch((error: Error) => error)
  expect((error as Error).message.includes('最新版微信')).toBe(true)
})


test('payment hooks retain raw failures and existing user-facing cancellation and expiry behavior', async () => {
  for (const [failure, expectedEvent, expectedMessage] of [
    [{ errCode: -2, errMsg: 'raw provider message' }, 'pay_cancelled', '支付已取消'],
    [{ errCode: -3, errMsg: 'CANCEL' }, 'pay_cancelled', '支付已取消'],
    [{ errCode: -15007, errMsg: 'expired' }, 'pay_failed', '支付会话已过期，请重新购买'],
  ] as const) {
    const events: any[] = []
    testGlobal.uni = { canIUse: () => true, requestVirtualPayment: (options: any) => options.fail(failure) } as unknown as typeof uni
    const attempt = { attemptId: 'id', report: (event: string, extra?: unknown) => { events.push({ event, ...extra as object }) }, bindOrder: () => {} }
    expect((await requestTicketWatchPayment(virtualParams, attempt).catch(error => error)).message).toBe(expectedMessage)
    expect(events).toEqual([{ event: 'pay_sheet_invoked' }, { event: expectedEvent, error_code: String(failure.errCode), error_msg: failure.errMsg }])
    expect((await requestTicketWatchPayment(virtualParams).catch(error => error)).message).toBe(expectedMessage)
  }
})

test('payment success reports invoked then success and swallowing observer errors preserves resolution', async () => {
  const events: string[] = []
  testGlobal.uni = { canIUse: () => true, requestVirtualPayment: (options: any) => options.success() } as unknown as typeof uni
  await requestTicketWatchPayment(virtualParams, { attemptId: 'id', report: (event) => { events.push(event) }, bindOrder: () => {} })
  expect(events).toEqual(['pay_sheet_invoked', 'pay_success_client'])
  await expect(requestTicketWatchPayment(virtualParams, { attemptId: 'id', report: () => { throw new Error('observer failed') }, bindOrder: () => {} })).resolves.toBe(undefined)
})
