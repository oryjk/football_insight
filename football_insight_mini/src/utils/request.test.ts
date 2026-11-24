import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { ApiRequestError } from './apiError'
import { createApiRequester } from './request'

const main = 'https://oryjk.cn:18443/football-api/v1'
const backup = 'https://backup.oryjk.cn:19443/football-api/v1'
const globalWithUni = globalThis as unknown as { uni?: unknown }
const originalUni = globalWithUni.uni
type Request = UniApp.RequestOptions
let calls: Request[]
let modals: UniApp.ShowModalOptions[]

beforeEach(() => {
  calls = []
  modals = []
  globalWithUni.uni = {
    request(options: Request) { calls.push(options) },
    showModal(options: UniApp.ShowModalOptions) { modals.push(options) },
    getStorageSync() { return 'test-token' },
  }
})

afterEach(() => {
  for (const modal of modals) modal.complete?.({ errMsg: 'showModal:ok' })
  globalWithUni.uni = originalUni
})

function respond(index: number, statusCode = 200, data = { ok: true }) {
  calls[index].success?.({ statusCode, data, errMsg: 'request:ok', header: {}, cookies: [] })
}

function fail(index: number, errMsg = 'request:fail timeout') {
  calls[index].fail?.({ errMsg })
}

const urls = () => calls.map(({ url }) => url)

async function expectRequestError(result: Promise<unknown>, expected: Record<string, unknown>) {
  const error = await result.catch((error: unknown) => error)
  expect(error).toBeInstanceOf(ApiRequestError)
  expect(error).toMatchObject(expected)
}

describe('production mini-program API failover', () => {
  test('reroutes an authenticated GET following a transport failure and sticks to the working backup', async () => {
    const { request } = createApiRequester(main, backup)
    const result = request({ url: '/auth/me?view=full', auth: true, headers: { 'X-Fi-Test': 'value' } })
    fail(0, 'request:fail connection reset')
    expect(urls()).toEqual([main + '/auth/me?view=full', backup + '/auth/me?view=full'])
    expect(calls[1].header).toMatchObject({ Authorization: 'Bearer test-token', 'X-Fi-Test': 'value' })
    expect(modals.length).toBe(0)
    respond(1)
    expect(await result).toEqual({ ok: true })

    const write = request({ url: '/payment/order', method: 'POST', data: { sku: 42 } })
    expect(calls[2]).toMatchObject({ url: backup + '/payment/order', method: 'POST', data: { sku: 42 } })
    fail(2)
    await expectRequestError(write, { networkFailed: true })
    expect(calls.length).toBe(3)
  })

  for (const status of [502, 503, 504]) {
    test(`tries the alternate after HTTP ${status}`, async () => {
      const { request } = createApiRequester(main, backup)
      const result = request({ url: '/rounds' })
      respond(0, status)
      expect(calls[1].url).toBe(backup + '/rounds')
      respond(1)
      await expect(result).resolves.toEqual({ ok: true })
    })
  }

  for (const status of [401, 404, 429, 500]) {
    test(`preserves HTTP ${status} without rerouting`, async () => {
      const { request } = createApiRequester(main, backup)
      const result = request({ url: '/auth/me', auth: true })
      respond(0, status)
      await expectRequestError(result, { statusCode: status, networkFailed: false, body: { ok: true } })
      expect(calls.length).toBe(1)
      expect(modals.length).toBe(0)
    })
  }

  test('preserves the primary gateway response when the alternate also fails', async () => {
    const { request } = createApiRequester(main, backup)
    const result = request({ url: '/rounds' })
    respond(0, 503)
    fail(1)
    await expectRequestError(result, { statusCode: 503, networkFailed: false, body: { ok: true } })
    expect(calls.length).toBe(2)
    expect(modals.length).toBe(0)
  })

  test('shows a network dialog only after both routes fail', async () => {
    const { request } = createApiRequester(main, backup)
    const result = request({ url: '/rounds' })
    fail(0)
    expect(modals.length).toBe(0)
    fail(1)
    await expectRequestError(result, { networkFailed: true })
    expect(calls.length).toBe(2)
    expect(modals.length).toBe(1)
  })

  test('returns an authoritative 401 from the alternate without latching onto it', async () => {
    const { request } = createApiRequester(main, backup)
    const result = request({ url: '/auth/me', auth: true })
    fail(0)
    respond(1, 401)
    await expectRequestError(result, { statusCode: 401, networkFailed: false })
    expect(modals.length).toBe(0)
    const next = request({ url: '/rounds' })
    expect(calls[2].url).toBe(main + '/rounds')
    respond(2)
    await next
  })

  test('retains a newer primary recovery when an earlier backup reply arrives late', async () => {
    let now = 0
    const { request } = createApiRequester(main, backup, () => now)
    const early = request({ url: '/early' })
    const slow = request({ url: '/slow' })
    fail(0)
    fail(1)
    respond(2)
    await early

    now = 60_001
    const probe = request({ url: '/probe' })
    expect(calls[4].url).toBe(main + '/probe')
    respond(4)
    await probe
    respond(3)
    await slow

    const next = request({ url: '/next' })
    expect(calls[5].url).toBe(main + '/next')
    respond(5)
    await next
  })

  test('gates recovery probes on reads, coalesces their cooldown and restores primary on success', async () => {
    let now = 0
    const { request } = createApiRequester(main, backup, () => now)
    const first = request({ url: '/rounds' })
    fail(0)
    respond(1)
    await first

    const beforeCooldown = request({ url: '/before' })
    respond(2)
    await beforeCooldown

    now = 60_001
    const write = request({ url: '/write', method: 'PUT' })
    const probe = request({ url: '/probe' })
    const parallel = request({ url: '/parallel' })
    expect(urls().slice(2)).toEqual([backup + '/before', backup + '/write', main + '/probe', backup + '/parallel'])
    respond(3)
    respond(4)
    respond(5)
    await Promise.all([write, probe, parallel])

    const recovered = request({ url: '/restored' })
    expect(calls[6].url).toBe(main + '/restored')
    respond(6)
    await recovered
  })

  test('stays on backup when the cooldown probe finds primary unavailable', async () => {
    let now = 0
    const { request } = createApiRequester(main, backup, () => now)
    const first = request({ url: '/rounds' })
    respond(0, 504)
    respond(1)
    await first

    now = 60_001
    const probe = request({ url: '/probe' })
    respond(2, 503)
    respond(3)
    await probe

    const next = request({ url: '/next' })
    expect(urls().slice(2)).toEqual([main + '/probe', backup + '/probe', backup + '/next'])
    expect(modals.length).toBe(0)
    respond(4)
    await next
  })

  test('prepares the first authenticated write with one shared credential-free read probe', async () => {
    const { request, prepareApiRoute } = createApiRequester(main, backup)
    const probe = prepareApiRoute()
    expect(probe).toBe(prepareApiRoute())
    expect(calls[0]).toMatchObject({ url: main + '/system/public-config', method: 'GET', timeout: 3_000 })
    expect(calls[0].header?.Authorization).toBe(undefined)
    fail(0)
    expect(calls[1].url).toBe(backup + '/system/public-config')
    respond(1)
    await probe

    expect(prepareApiRoute()).toBe(undefined)
    const login = request({ url: '/auth/mini-wechat/login', method: 'POST', data: { code: 'test-code' } })
    expect(calls[2]).toMatchObject({ url: backup + '/auth/mini-wechat/login', method: 'POST', data: { code: 'test-code' } })
    respond(2)
    await login
  })

  test('preserves the same-host timeout retry for absolute reads and sends writes once', async () => {
    const { request } = createApiRequester(main, backup)
    const external = request({ url: 'https://external.example/resource', silentNetworkError: true })
    fail(0)
    fail(1)
    await expectRequestError(external, { networkFailed: true })
    const write = request({ url: '/order', method: 'POST' })
    respond(2, 503)
    await expectRequestError(write, { statusCode: 503 })
    expect(urls()).toEqual(['https://external.example/resource', 'https://external.example/resource', main + '/order'])
    expect(modals.length).toBe(0)
  })

  test('a GET with server-side changes uses the active route without probing or replaying', async () => {
    let now = 0
    const { request } = createApiRequester(main, backup, () => now)
    const first = request({ url: '/rounds' })
    fail(0)
    respond(1)
    await first

    now = 60_001
    const status = request({ url: '/payment/order/123', allowFailover: false })
    expect(calls[2].url).toBe(backup + '/payment/order/123')
    fail(2)
    await expectRequestError(status, { networkFailed: true })
    expect(calls.length).toBe(3)
  })

  test('keeps the legacy GET timeout retry when using a single configured origin', async () => {
    const { request, prepareApiRoute } = createApiRequester('https://h5.example/api/v1', null)
    expect(prepareApiRoute()).toBe(undefined)
    const result = request({ url: '/rounds' })
    fail(0)
    expect(urls()).toEqual(['https://h5.example/api/v1/rounds', 'https://h5.example/api/v1/rounds'])
    respond(1)
    await expect(result).resolves.toEqual({ ok: true })
  })
})
