import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { getCurrentUser } from './auth'
import { ApiRequestError } from '../utils/apiError'

interface FakeUniRequestOptions {
  url: string
  success: (response: { statusCode: number; data: unknown }) => void
}

type Responder = () => { statusCode: number; data: unknown }

const globalWithUni = globalThis as unknown as { uni?: unknown }
const previousUni = globalWithUni.uni
let responder: Responder

beforeEach(() => {
  responder = () => ({ statusCode: 500, data: null })
  globalWithUni.uni = {
    getStorageSync: () => '',
    setStorageSync: () => {},
    removeStorageSync: () => {},
    showModal: () => {},
    request: (options: FakeUniRequestOptions) => {
      options.success(responder())
    },
  }
})

afterEach(() => {
  globalWithUni.uni = previousUni
})

async function captureError(action: () => Promise<unknown>): Promise<unknown> {
  try {
    await action()
    return null
  } catch (error) {
    return error
  }
}

describe('getCurrentUser 认证错误分流', () => {
  test('200 返回当前用户', async () => {
    responder = () => ({ statusCode: 200, data: { id: 'u1', display_name: '测试' } })
    await expect(getCurrentUser()).resolves.toMatchObject({ id: 'u1' })
  })

  test('401 无论后端文案如何都返回 null 而不是抛错', async () => {
    responder = () => ({ statusCode: 401, data: { message: '登录已过期' } })
    await expect(getCurrentUser()).resolves.toBeNull()
  })

  test('401 纯文本响应体也返回 null', async () => {
    responder = () => ({ statusCode: 401, data: 'Unauthorized' })
    await expect(getCurrentUser()).resolves.toBeNull()
  })

  test('403 继续抛出并保留状态码', async () => {
    responder = () => ({ statusCode: 403, data: { message: '无权限' } })
    const error = await captureError(() => getCurrentUser())
    expect(error).toBeInstanceOf(ApiRequestError)
    expect((error as ApiRequestError).statusCode).toBe(403)
  })

  test('500 即使文案包含“未登录”也继续抛出（旧实现的误吞场景）', async () => {
    responder = () => ({ statusCode: 500, data: { message: '服务器开小差（未登录字样）' } })
    const error = await captureError(() => getCurrentUser())
    expect(error).toBeInstanceOf(ApiRequestError)
    expect((error as ApiRequestError).statusCode).toBe(500)
  })

  test('500 空响应体时错误文案带状态码', async () => {
    responder = () => ({ statusCode: 500, data: null })
    const error = await captureError(() => getCurrentUser())
    expect((error as ApiRequestError).message).toBe('请求失败（500）')
  })
})
