import type {
  AuthResponse,
  CurrentUser,
  LoginPayload,
  MiniWechatBindPayload,
  MiniWechatLoginResponse,
  RegisterPayload,
  ResetPasswordPayload,
} from '../types/auth'
import { setAccessToken } from '../utils/authStorage'
import { request } from '../utils/request'

export async function getCurrentUser(): Promise<CurrentUser | null> {
  try {
    return await request<CurrentUser>({ url: '/auth/me', auth: true })
  } catch (error) {
    if (error instanceof Error && error.message.includes('401')) {
      return null
    }
    if (error instanceof Error && error.message.includes('未登录')) {
      return null
    }
    if (error instanceof Error && error.message.includes('Unauthorized')) {
      return null
    }
    if (error instanceof Error && error.message.includes('not logged in')) {
      return null
    }
    throw error
  }
}

export async function login(payload: LoginPayload): Promise<AuthResponse> {
  const response = await request<AuthResponse>({
    url: '/auth/login',
    method: 'POST',
    data: payload,
  })
  setAccessToken(response.access_token)
  return response
}

export async function register(payload: RegisterPayload): Promise<AuthResponse> {
  const response = await request<AuthResponse>({
    url: '/auth/register',
    method: 'POST',
    data: payload,
  })
  setAccessToken(response.access_token)
  return response
}

export function resetPassword(payload: ResetPasswordPayload): Promise<void> {
  return request<void>({
    url: '/auth/reset-password',
    method: 'POST',
    data: payload,
  })
}

export function loginWithMiniWechat(code: string): Promise<MiniWechatLoginResponse> {
  return request<MiniWechatLoginResponse>({
    url: '/auth/mini-wechat/login',
    method: 'POST',
    data: { code },
  }).then((response) => {
    if (response.status === 'authenticated') {
      setAccessToken(response.access_token)
    }

    return response
  })
}

export async function bindMiniWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse> {
  const response = await request<AuthResponse>({
    url: '/auth/mini-wechat/bind',
    method: 'POST',
    data: payload,
  })
  setAccessToken(response.access_token)
  return response
}

export async function logout(): Promise<void> {
  try {
    await request<void>({ url: '/auth/logout', method: 'POST', auth: true })
  } finally {
    setAccessToken(null)
  }
}
