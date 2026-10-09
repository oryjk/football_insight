import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { createRuntimeConfigStore } from './runtimeConfig'
import { createMiniReviewStore } from './miniReview'
import type * as systemApi from '../api/system'
import type * as miniReviewApi from '../api/miniReview'
import type { MiniReviewStatus, PublicSystemConfig } from '../types/system'
import { ApiRequestError } from '../utils/apiError'

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

const configA: PublicSystemConfig = { wechat_login_enabled: true }
const configB: PublicSystemConfig = {
  wechat_login_enabled: false,
  membership_tier_rules: [{ code: 'V6', kind: 'paid', ticket_watch_poll_interval_seconds: 60 }],
}

const realDateNow = Date.now
let nowOverride: number | null = null

function setNow(value: number): void {
  nowOverride = value
}

function advanceNow(ms: number): void {
  nowOverride = (nowOverride ?? realDateNow()) + ms
}

function createSourceStub() {
  const stub = {
    calls: 0,
    handler: (): Promise<PublicSystemConfig> => Promise.reject(new Error('unexpected call')),
    source: {} as typeof systemApi,
  }
  stub.source = {
    getPublicSystemConfig: () => {
      stub.calls += 1
      return stub.handler()
    },
  } as unknown as typeof systemApi
  return stub
}

beforeEach(() => {
  nowOverride = null
  Date.now = () => nowOverride ?? realDateNow()
})

afterEach(() => {
  Date.now = realDateNow
})

describe('runtimeConfig store 加载与缓存', () => {
  test('并发调用合并在途请求', async () => {
    const stub = createSourceStub()
    const pending = deferred<PublicSystemConfig>()
    stub.handler = () => pending.promise
    const store = createRuntimeConfigStore(stub.source)

    const first = store.ensureRuntimeConfig()
    const second = store.ensureRuntimeConfig()
    pending.resolve(configA)

    await expect(first).resolves.toEqual(configA)
    await expect(second).resolves.toEqual(configA)
    expect(stub.calls).toBe(1)
    expect(store.config.value).toEqual(configA)
    expect(store.available.value).toBe(true)
    expect(store.ready.value).toBe(true)
  })

  test('成功值 60 秒内复用，超过后重新请求', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(configA)
    const store = createRuntimeConfigStore(stub.source)
    setNow(1_000_000)

    await store.ensureRuntimeConfig()
    await store.ensureRuntimeConfig()
    expect(stub.calls).toBe(1)

    advanceNow(61_000)
    await store.ensureRuntimeConfig()
    expect(stub.calls).toBe(2)
  })

  test('force 绕过成功缓存，但仍合并在途请求', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(configA)
    const store = createRuntimeConfigStore(stub.source)

    await store.ensureRuntimeConfig()

    const pending = deferred<PublicSystemConfig>()
    stub.handler = () => pending.promise
    const first = store.ensureRuntimeConfig({ force: true })
    const second = store.ensureRuntimeConfig({ force: true })
    pending.resolve(configB)
    await first
    await second

    // 首次 1 次 + force 合并后 1 次
    expect(stub.calls).toBe(2)
    expect(store.config.value).toEqual(configB)
  })
})

describe('runtimeConfig store 失败语义', () => {
  test('失败后返回 null、error 可观察，下次调用重试', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const store = createRuntimeConfigStore(stub.source)

    await expect(store.ensureRuntimeConfig()).resolves.toBeNull()
    expect(store.ready.value).toBe(true)
    expect(store.available.value).toBe(false)
    expect(store.errorMessage.value).toBe('请求失败（500）')

    stub.handler = () => Promise.resolve(configA)
    await expect(store.ensureRuntimeConfig()).resolves.toEqual(configA)
    expect(stub.calls).toBe(2)
    expect(store.available.value).toBe(true)
    expect(store.errorMessage.value).toBe('')
  })

  test('刷新失败保留最后一次成功 config，available/error 反映本次失败', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(configA)
    const store = createRuntimeConfigStore(stub.source)
    setNow(1_000_000)

    await store.ensureRuntimeConfig()

    advanceNow(61_000)
    stub.handler = () => Promise.reject(new ApiRequestError('网络连接不可用，请检查网络后重试', 0, true))
    // 刷新失败返回 null，不能用旧成功值冒充本次确认成功。
    await expect(store.ensureRuntimeConfig({ force: true })).resolves.toBeNull()

    expect(store.config.value).toEqual(configA)
    expect(store.available.value).toBe(false)
    expect(store.errorMessage.value).toBe('网络连接不可用，请检查网络后重试')
    expect(store.ready.value).toBe(true)

    // 失败不标成永久已加载：下次调用重新请求。
    stub.handler = () => Promise.resolve(configB)
    await expect(store.ensureRuntimeConfig()).resolves.toEqual(configB)
    expect(stub.calls).toBe(3)
  })

  test('ready 与 available 区分「已尝试」和「已获取」', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const store = createRuntimeConfigStore(stub.source)

    expect(store.ready.value).toBe(false)
    expect(store.available.value).toBe(false)

    await store.ensureRuntimeConfig()
    expect(store.ready.value).toBe(true)
    expect(store.available.value).toBe(false)
  })

  test('与 miniReview store 独立加载，一个失败不影响另一个', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const configStore = createRuntimeConfigStore(stub.source)

    const reviewStatus: MiniReviewStatus = {
      project_code: 'football_insight_mini',
      version: '1.0.0',
      is_reviewing: false,
      status_text: '已过审',
    }
    const reviewStore = createMiniReviewStore({
      getMiniReviewStatus: () => Promise.resolve(reviewStatus),
    } as unknown as typeof miniReviewApi)

    await expect(configStore.ensureRuntimeConfig()).resolves.toBeNull()
    await expect(reviewStore.ensureMiniReviewReady()).resolves.toEqual(reviewStatus)

    expect(configStore.available.value).toBe(false)
    expect(reviewStore.available.value).toBe(true)
    expect(reviewStore.status.value).toEqual(reviewStatus)
  })
})
