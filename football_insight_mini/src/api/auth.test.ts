import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import {
  bindMiniWechatAccount,
  getCurrentUser,
  getNotificationEmail,
  listH5TestLoginUsers,
  login,
  loginAsH5TestUser,
  loginWithMiniWechat,
  logout,
  register,
  resetPassword,
  updateNotificationEmail,
} from './auth'
import { ApiRequestError } from '../utils/apiError'
import { AUTH_TOKEN_STORAGE_KEY } from '../utils/authStorage'
import type { AuthResponse, MiniWechatLoginResponse } from '../types/auth'
import { captureAcquisitionSource } from '../utils/acquisitionSource'
import { createAppSession } from '../stores/appSession'

interface FakeRequestCall {
  url: string
  method?: string
  data?: unknown
  header?: Record<string, string>
}

interface FakeUniRequestOptions {
  url: string
  method?: string
  data?: unknown
  header?: Record<string, string>
  success: (response: { statusCode: number; data: unknown }) => void
}

type Responder = () => { statusCode: number; data: unknown }

const globalWithUni = globalThis as unknown as { uni?: unknown }
const previousUni = globalWithUni.uni
let responder: Responder
let requestCalls: FakeRequestCall[]
let storage: Record<string, unknown>

function authResponseData(token: string): AuthResponse {
  return {
    access_token: token,
    expires_at: '2026-02-01T00:00:00Z',
    user: {
      id: 'u1',
      display_name: '测试',
      account_identifier: 'tester',
      avatar_url: null,
      has_wechat_binding: false,
      membership_tier: 'FREE',
      created_at: '2026-01-01T00:00:00Z',
    },
  }
}

beforeEach(() => {
  responder = () => ({ statusCode: 500, data: null })
  requestCalls = []
  storage = {}
  globalWithUni.uni = {
    getStorageSync: (key: string) => storage[key] ?? '',
    setStorageSync: (key: string, value: unknown) => {
      storage[key] = value
    },
    removeStorageSync: (key: string) => {
      delete storage[key]
    },
    showModal: () => {},
    request: (options: FakeUniRequestOptions) => {
      requestCalls.push({
        url: options.url,
        method: options.method,
        data: options.data,
        header: options.header,
      })
      options.success(responder())
    },
  }
})

afterEach(() => {
  globalWithUni.uni = previousUni
})


/** bun:test 类型不含 expect.stringContaining，URL 用显式 includes 断言。 */
function expectRequestUrlContains(index: number, path: string): void {
  expect((requestCalls[index]?.url ?? '').includes(path)).toBe(true)
}

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

describe('认证 API 原子请求（不写/不清存储，token 由 appSession 发布）', () => {
  test('login 不写入 access token', async () => {
    responder = () => ({ statusCode: 200, data: authResponseData('token-login') })
    await expect(login({ account_identifier: 'a', password: 'x' })).resolves.toMatchObject({
      access_token: 'token-login',
    })
    expectRequestUrlContains(0, '/auth/login')
    expect(requestCalls[0]?.method).toBe('POST')
    expect(AUTH_TOKEN_STORAGE_KEY in storage).toBe(false)
  })

  test('register 不写入 access token', async () => {
    responder = () => ({ statusCode: 200, data: authResponseData('token-register') })
    await expect(register({ account_identifier: 'a', password: 'x' })).resolves.toMatchObject({
      access_token: 'token-register',
    })
    expect(AUTH_TOKEN_STORAGE_KEY in storage).toBe(false)
  })

  test('loginAsH5TestUser 不写入 access token', async () => {
    responder = () => ({ statusCode: 200, data: authResponseData('token-h5') })
    await expect(loginAsH5TestUser('u1')).resolves.toMatchObject({ access_token: 'token-h5' })
    expectRequestUrlContains(0, '/auth/h5-test-login')
    expect(requestCalls[0]?.method).toBe('POST')
    expect(requestCalls[0]?.data).toEqual({ user_id: 'u1' })
    expect(AUTH_TOKEN_STORAGE_KEY in storage).toBe(false)
  })

  test('loginWithMiniWechat authenticated 响应也不写入 access token', async () => {
    const data: MiniWechatLoginResponse = { status: 'authenticated', ...authResponseData('token-wechat') }
    responder = () => ({ statusCode: 200, data })
    await expect(loginWithMiniWechat('code-1')).resolves.toMatchObject({ status: 'authenticated' })
    expect(AUTH_TOKEN_STORAGE_KEY in storage).toBe(false)
  })

  test('bindMiniWechatAccount 不写入 access token', async () => {
    responder = () => ({ statusCode: 200, data: authResponseData('token-bind') })
    await expect(bindMiniWechatAccount({
      bind_token: 'bind-token',
      display_name: '测试',
      avatar_data_url: 'data:image/png;base64,x',
    })).resolves.toMatchObject({ access_token: 'token-bind' })
    expect(AUTH_TOKEN_STORAGE_KEY in storage).toBe(false)
  })

  test('logout 只发请求，finally 不再清 token（清理由 appSession 负责）', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-existing'
    responder = () => ({ statusCode: 200, data: null })
    await logout()
    expectRequestUrlContains(0, '/auth/logout')
    expect(requestCalls[0]?.method).toBe('POST')
    expect(requestCalls[0]?.header?.Authorization).toBe('Bearer token-existing')
    expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe('token-existing')
  })

  test('logout 请求失败也不清 token', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-existing'
    responder = () => ({ statusCode: 500, data: null })
    const error = await captureError(() => logout())
    expect(error).toBeInstanceOf(ApiRequestError)
    expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe('token-existing')
  })
})

describe('resetPassword 与邮箱 API 契约保持', () => {
  test('resetPassword 请求契约不变且不碰存储', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-existing'
    responder = () => ({ statusCode: 200, data: null })
    await resetPassword({ invite_code: 'INVITE', account_identifier: 'a', password: 'new-pass' })
    expectRequestUrlContains(0, '/auth/reset-password')
    expect(requestCalls[0]?.method).toBe('POST')
    expect(requestCalls[0]?.data).toEqual({ invite_code: 'INVITE', account_identifier: 'a', password: 'new-pass' })
    expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe('token-existing')
  })

  test('listH5TestLoginUsers 契约不变', async () => {
    responder = () => ({ statusCode: 200, data: { items: [{ id: 'u1', display_name: '测试', account_identifier: 'tester' }] } })
    await expect(listH5TestLoginUsers()).resolves.toMatchObject({ items: [{ id: 'u1' }] })
    expectRequestUrlContains(0, '/auth/h5-test-login/users')
  })

  test('getNotificationEmail / updateNotificationEmail 契约不变', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-existing'
    responder = () => ({ statusCode: 200, data: { email: 'a@example.com' } })

    await expect(getNotificationEmail()).resolves.toEqual({ email: 'a@example.com' })
    expectRequestUrlContains(0, '/ticket-watch/reflux-subscriptions/email')
    expect(requestCalls[0]?.header?.Authorization).toBe('Bearer token-existing')

    await expect(updateNotificationEmail('b@example.com')).resolves.toEqual({ email: 'a@example.com' })
    expectRequestUrlContains(1, '/ticket-watch/reflux-subscriptions/email')
    expect(requestCalls[1]?.method).toBe('PUT')
    expect(requestCalls[1]?.data).toEqual({ email: 'b@example.com' })
    expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe('token-existing')
  })
})

describe('first source through real session and API transport', () => {
  test('attaches the same first source through login and binding without changing auth payloads otherwise', async () => {
    responder = () => ({ statusCode: 200, data: { status: 'authenticated', ...authResponseData('token') } })
    await loginWithMiniWechat('old-client')
    expect(requestCalls[0]?.data).toEqual({ code: 'old-client' })
    captureAcquisitionSource({ query: { src: 'douyin', cid: 'first' }, scene: 1047 })
    const session = createAppSession(undefined, { getMiniWechatCode: async () => 'new-client' })
    await session.loginWithMiniWechat()
    await session.bindWechatAccount({ bind_token: 'bind', display_name: 'name', avatar_data_url: 'avatar' })
    await session.registerAccount({ account_identifier: 'a', password: 'secret' })
    await session.loginWithPassword({ account_identifier: 'a', password: 'secret' })
    for (const call of requestCalls.slice(1)) {
      expect((call.data as Record<string, unknown>).acquisition).toEqual({ src: 'douyin', cid: 'first', landing_scene: 1047 })
    }
    expect((requestCalls[2]?.data as Record<string, unknown>).bind_token).toBe('bind')
    await session.logoutSession()
    captureAcquisitionSource({ query: { src: 'xhs', cid: 'second' } })
    await session.loginWithPassword({ account_identifier: 'b', password: 'secret' })
    expect((requestCalls[requestCalls.length - 1]?.data as Record<string, unknown>).acquisition)
      .toEqual({ src: 'douyin', cid: 'first', landing_scene: 1047 })
  })
})
