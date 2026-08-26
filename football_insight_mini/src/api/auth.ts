import type {
  AuthResponse,
  CurrentUser,
  H5TestLoginUser,
  LoginPayload,
  MiniWechatBindPayload,
  MiniWechatLoginResponse,
  NotificationEmailResponse,
  RegisterPayload,
  ResetPasswordPayload,
} from '../types/auth'
import { setAccessToken } from '../utils/authStorage'
import { isUnauthorizedError } from '../utils/apiError'
import { request } from '../utils/request'

export async function getCurrentUser(): Promise<CurrentUser | null> {
  try {
    return await request<CurrentUser>({ url: '/auth/me', auth: true })
  } catch (error) {
    if (isUnauthorizedError(error)) {
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

export function listH5TestLoginUsers(): Promise<{ items: H5TestLoginUser[] }> {
  return request<{ items: H5TestLoginUser[] }>({ url: '/auth/h5-test-login/users' })
}

export function loginAsH5TestUser(userId: string): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: '/auth/h5-test-login',
    method: 'POST',
    data: { user_id: userId },
  }).then((response) => {
    setAccessToken(response.access_token)
    return response
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

export function getNotificationEmail(): Promise<NotificationEmailResponse> {
  return request<NotificationEmailResponse>({
    url: '/ticket-watch/reflux-subscriptions/email',
    auth: true,
  })
}

export function updateNotificationEmail(email: string): Promise<NotificationEmailResponse> {
  return request<NotificationEmailResponse>({
    url: '/ticket-watch/reflux-subscriptions/email',
    method: 'PUT',
    auth: true,
    data: { email },
  })
}
