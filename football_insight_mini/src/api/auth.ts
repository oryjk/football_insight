/**
 * 认证相关原子请求：本模块不再写/清 access token（fi_access_token），
 * token 与当前用户的发布/清除统一由 stores/appSession 在核对 revision 后执行。
 */
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

export function login(payload: LoginPayload): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: '/auth/login',
    method: 'POST',
    data: payload,
  })
}

export function register(payload: RegisterPayload): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: '/auth/register',
    method: 'POST',
    data: payload,
  })
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
  })
}

export function loginWithMiniWechat(code: string): Promise<MiniWechatLoginResponse> {
  return request<MiniWechatLoginResponse>({
    url: '/auth/mini-wechat/login',
    method: 'POST',
    data: { code },
  })
}

export function bindMiniWechatAccount(payload: MiniWechatBindPayload): Promise<AuthResponse> {
  return request<AuthResponse>({
    url: '/auth/mini-wechat/bind',
    method: 'POST',
    data: payload,
  })
}

export function logout(): Promise<void> {
  return request<void>({ url: '/auth/logout', method: 'POST', auth: true })
}

export function createWechatOfficialBindCode(): Promise<{ code: string; expires_in_seconds: number }> {
  return request<{ code: string; expires_in_seconds: number }>({
    url: '/auth/wechat-official-bind-code',
    method: 'POST',
    auth: true,
  })
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
