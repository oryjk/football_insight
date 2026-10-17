import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { createMiniReviewStore } from './miniReview'
import type * as miniReviewApi from '../api/miniReview'
import type { MiniReviewStatus } from '../types/system'
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

function buildStatus(isReviewing: boolean, statusText = isReviewing ? '审核中' : '已过审'): MiniReviewStatus {
  return {
    project_code: 'football_insight_mini',
    version: '1.0.0',
    is_reviewing: isReviewing,
    status_text: statusText,
  }
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
    handler: (): Promise<MiniReviewStatus> => Promise.reject(new Error('unexpected call')),
    source: {} as typeof miniReviewApi,
  }
  stub.source = {
    getMiniReviewStatus: () => {
      stub.calls += 1
      return stub.handler()
    },
  } as unknown as typeof miniReviewApi
  return stub
}

beforeEach(() => {
  nowOverride = null
  Date.now = () => nowOverride ?? realDateNow()
})

afterEach(() => {
  Date.now = realDateNow
})

describe('miniReview 开发构建', () => {
  test('开发构建保留真实审核状态，但不限制功能展示', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(true))
    const store = createMiniReviewStore(stub.source, { isDevelopment: true })

    expect(store.shouldRestrictFeatures.value).toBe(false)
    await store.ensureMiniReviewReady()

    expect(store.status.value?.is_reviewing).toBe(true)
    expect(store.isUnderReview.value).toBe(false)
    expect(store.shouldRestrictFeatures.value).toBe(false)
    store.setMiniReviewStatus(buildStatus(true, '管理员设置审核中'))
    expect(store.status.value?.status_text).toBe('管理员设置审核中')
    expect(store.shouldRestrictFeatures.value).toBe(false)
  })

  test('开发构建查询失败也不限制展示，仍保留错误供诊断', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new Error('查询失败'))
    const store = createMiniReviewStore(stub.source, { isDevelopment: true })

    await store.ensureMiniReviewReady()

    expect(store.isUnderReview.value).toBe(false)
    expect(store.shouldRestrictFeatures.value).toBe(false)
    expect(store.available.value).toBe(false)
    expect(store.errorMessage.value).toBe('查询失败')
  })

  test('生产构建保留查询前和审核中的展示限制', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(true))
    const store = createMiniReviewStore(stub.source, { isDevelopment: false })

    expect(store.shouldRestrictFeatures.value).toBe(true)
    await store.ensureMiniReviewReady()
    expect(store.isUnderReview.value).toBe(true)
    expect(store.shouldRestrictFeatures.value).toBe(true)
    store.setMiniReviewStatus(buildStatus(false))
    expect(store.shouldRestrictFeatures.value).toBe(false)
  })
})

describe('miniReview store 加载与缓存', () => {
  test('并发调用合并在途请求', async () => {
    const stub = createSourceStub()
    const pending = deferred<MiniReviewStatus>()
    stub.handler = () => pending.promise
    const store = createMiniReviewStore(stub.source)

    const first = store.ensureMiniReviewReady()
    const second = store.ensureMiniReviewReady()
    pending.resolve(buildStatus(true))

    await expect(first).resolves.toMatchObject({ is_reviewing: true })
    await expect(second).resolves.toMatchObject({ is_reviewing: true })
    expect(stub.calls).toBe(1)
    expect(store.isUnderReview.value).toBe(true)
    expect(store.available.value).toBe(true)
  })

  test('成功值 60 秒内复用，超过后重新请求', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(false))
    const store = createMiniReviewStore(stub.source)
    setNow(1_000_000)

    await store.ensureMiniReviewReady()
    await store.ensureMiniReviewReady()
    expect(stub.calls).toBe(1)

    advanceNow(61_000)
    await store.ensureMiniReviewReady()
    expect(stub.calls).toBe(2)
  })

  test('force 绕过成功缓存，但仍合并在途请求', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(false))
    const store = createMiniReviewStore(stub.source)

    await store.ensureMiniReviewReady()

    const pending = deferred<MiniReviewStatus>()
    stub.handler = () => pending.promise
    const first = store.ensureMiniReviewReady({ force: true })
    const second = store.ensureMiniReviewReady({ force: true })
    pending.resolve(buildStatus(true))
    await first
    await second

    expect(stub.calls).toBe(2)
    expect(store.isUnderReview.value).toBe(true)
  })
})

describe('miniReview store 账号变化', () => {
  test('登录账号变化后重新查询，旧身份的在途结果不能覆盖新结果', async () => {
    const stub = createSourceStub()
    const anonymous = deferred<MiniReviewStatus>()
    stub.handler = () => anonymous.promise
    const store = createMiniReviewStore(stub.source)

    const first = store.ensureMiniReviewReady()
    stub.handler = () => Promise.resolve(buildStatus(false))
    const refreshed = store.refreshForIdentityChange()
    // 旧身份（匿名）的请求稍后才返回审核中，必须被丢弃。
    anonymous.resolve(buildStatus(true))
    await first
    await refreshed

    expect(stub.calls).toBe(2)
    expect(store.isUnderReview.value).toBe(false)
    expect(store.loading.value).toBe(false)
  })

  test('退出登录后重新查询回到审核中', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(false))
    const store = createMiniReviewStore(stub.source)
    await store.ensureMiniReviewReady()
    expect(store.isUnderReview.value).toBe(false)

    stub.handler = () => Promise.resolve(buildStatus(true))
    await store.refreshForIdentityChange()
    expect(store.isUnderReview.value).toBe(true)
  })
})

describe('miniReview store 失败语义', () => {
  test('查询失败返回 null、isUnderReview 为 false，下次调用可重试', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const store = createMiniReviewStore(stub.source)

    await expect(store.ensureMiniReviewReady()).resolves.toBeNull()
    expect(store.status.value).toBeNull()
    expect(store.isUnderReview.value).toBe(false)
    expect(store.ready.value).toBe(true)
    expect(store.available.value).toBe(false)
    expect(store.errorMessage.value).toBe('请求失败（500）')

    stub.handler = () => Promise.resolve(buildStatus(true))
    await expect(store.ensureMiniReviewReady()).resolves.toMatchObject({ is_reviewing: true })
    expect(stub.calls).toBe(2)
    expect(store.isUnderReview.value).toBe(true)
  })

  test('ready 与 available 区分「已尝试」和「已获取」', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.reject(new ApiRequestError('请求失败（500）', 500))
    const store = createMiniReviewStore(stub.source)

    expect(store.ready.value).toBe(false)
    expect(store.available.value).toBe(false)

    await store.ensureMiniReviewReady()
    expect(store.ready.value).toBe(true)
    expect(store.available.value).toBe(false)
  })
})

describe('miniReview store 管理员切换竞态', () => {
  test('GET pending 时管理员 PUT，旧 GET 返回不覆盖 PUT 结果', async () => {
    const stub = createSourceStub()
    const pendingGet = deferred<MiniReviewStatus>()
    stub.handler = () => pendingGet.promise
    const store = createMiniReviewStore(stub.source)

    const ensurePromise = store.ensureMiniReviewReady()
    const putStatus = buildStatus(true, '管理员切换为审核中')
    store.setMiniReviewStatus(putStatus)

    // 旧 GET 晚到，携带的是 PUT 之前的值。
    pendingGet.resolve(buildStatus(false, '已过审'))
    await expect(ensurePromise).resolves.toEqual(putStatus)

    expect(store.status.value).toEqual(putStatus)
    expect(store.isUnderReview.value).toBe(true)
    expect(store.available.value).toBe(true)
    expect(stub.calls).toBe(1)
  })

  test('GET pending 时管理员 PUT，旧 GET 失败也不清掉 PUT 结果', async () => {
    const stub = createSourceStub()
    const pendingGet = deferred<MiniReviewStatus>()
    stub.handler = () => pendingGet.promise
    const store = createMiniReviewStore(stub.source)

    const ensurePromise = store.ensureMiniReviewReady()
    const putStatus = buildStatus(true, '管理员切换为审核中')
    store.setMiniReviewStatus(putStatus)

    pendingGet.reject(new ApiRequestError('请求失败（500）', 500))
    await expect(ensurePromise).resolves.toEqual(putStatus)

    expect(store.status.value).toEqual(putStatus)
    expect(store.isUnderReview.value).toBe(true)
    expect(store.available.value).toBe(true)
    expect(store.errorMessage.value).toBe('')
  })

  test('PUT 发布后 60 秒内的 ensure 复用 PUT 结果', async () => {
    const stub = createSourceStub()
    stub.handler = () => Promise.resolve(buildStatus(false))
    const store = createMiniReviewStore(stub.source)
    setNow(1_000_000)

    store.setMiniReviewStatus(buildStatus(true))
    await expect(store.ensureMiniReviewReady()).resolves.toMatchObject({ is_reviewing: true })
    expect(stub.calls).toBe(0)
  })
})
