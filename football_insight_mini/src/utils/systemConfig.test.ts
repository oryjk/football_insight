import { afterEach, beforeEach, describe, expect, test } from 'bun:test'

import {
  loadSystemConfig,
  loadSystemConfigUnderReview,
  resolveSystemConfigUnderReview,
} from './systemConfig'

interface FakeUniRequestOptions {
  url: string
  success: (response: { statusCode: number; data: unknown }) => void
}

type Responder = () => { statusCode: number; data: unknown }

const globalWithUni = globalThis as unknown as { uni?: unknown }
const previousUni = globalWithUni.uni
let responder: Responder
let requestCount: number

const realDateNow = Date.now
let nowValue = realDateNow()

beforeEach(() => {
  responder = () => ({ statusCode: 500, data: null })
  requestCount = 0
  nowValue += 61_000
  Date.now = () => nowValue
  globalWithUni.uni = {
    getStorageSync: () => '',
    setStorageSync: () => {},
    removeStorageSync: () => {},
    showModal: () => {},
    request: (options: FakeUniRequestOptions) => {
      requestCount += 1
      options.success(responder())
    },
  }
})

afterEach(() => {
  globalWithUni.uni = previousUni
  Date.now = realDateNow
})

describe('resolveSystemConfigUnderReview', () => {
  test('treats reviewing status as review mode', () => {
    expect(resolveSystemConfigUnderReview({ is_reviewing: true })).toBe(true)
  })

  test('defaults to normal mode when backend config is absent or not under review', () => {
    expect(resolveSystemConfigUnderReview(null)).toBe(false)
    expect(resolveSystemConfigUnderReview({ is_reviewing: false })).toBe(false)
  })
})

describe('loadSystemConfig 委托 miniReview 共享 store', () => {
  test('查询成功时返回审核状态，underReview 跟随 is_reviewing', async () => {
    responder = () => ({
      statusCode: 200,
      data: {
        project_code: 'football_insight_mini',
        version: '1.0.0',
        is_reviewing: true,
        status_text: '审核中',
      },
    })

    await expect(loadSystemConfig()).resolves.toMatchObject({ is_reviewing: true })
    await expect(loadSystemConfigUnderReview()).resolves.toBe(true)
  })

  test('查询失败返回 null，underReview 兼容为 false', async () => {
    responder = () => ({ statusCode: 500, data: null })

    await expect(loadSystemConfig()).resolves.toBeNull()
    await expect(loadSystemConfigUnderReview()).resolves.toBe(false)
  })

  test('60 秒缓存内重复调用只请求一次', async () => {
    responder = () => ({
      statusCode: 200,
      data: {
        project_code: 'football_insight_mini',
        version: '1.0.0',
        is_reviewing: false,
        status_text: '已过审',
      },
    })

    await loadSystemConfig()
    await loadSystemConfig()
    expect(requestCount).toBe(1)
  })
})
