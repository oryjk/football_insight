export const AUTH_TOKEN_STORAGE_KEY = 'fi_access_token'
export const POST_LOGIN_REDIRECT_STORAGE_KEY = 'fi_post_login_redirect'
export const MANUAL_LOGOUT_STORAGE_KEY = 'fi_manual_logout'

export interface PostLoginRedirectTarget {
  url: string
  type: 'switchTab' | 'navigateTo'
}

export function getAccessToken(): string | null {
  const token = uni.getStorageSync(AUTH_TOKEN_STORAGE_KEY)
  return typeof token === 'string' && token.length > 0 ? token : null
}

export function setAccessToken(token: string | null): void {
  if (token) {
    uni.setStorageSync(AUTH_TOKEN_STORAGE_KEY, token)
    return
  }

  uni.removeStorageSync(AUTH_TOKEN_STORAGE_KEY)
}

export function getPostLoginRedirectTarget(): PostLoginRedirectTarget | null {
  const target = uni.getStorageSync(POST_LOGIN_REDIRECT_STORAGE_KEY)
  if (!target || typeof target !== 'object') {
    return null
  }

  const record = target as Record<string, unknown>
  if (typeof record.url !== 'string') {
    return null
  }

  if (record.type !== 'switchTab' && record.type !== 'navigateTo') {
    return null
  }

  return {
    url: record.url,
    type: record.type,
  }
}

export function setPostLoginRedirectTarget(target: PostLoginRedirectTarget | null): void {
  if (target) {
    uni.setStorageSync(POST_LOGIN_REDIRECT_STORAGE_KEY, target)
    return
  }

  uni.removeStorageSync(POST_LOGIN_REDIRECT_STORAGE_KEY)
}

/**
 * 手动退出标记：用户主动退出后持久化，跨冷启动禁止 MP 静默微信恢复；
 * 只有明确的认证操作（密码/微信/绑定/测试登录）才解除。只读写本 key，
 * 不触碰其他本地数据。
 */
export function hasManualLogout(): boolean {
  const flag = uni.getStorageSync(MANUAL_LOGOUT_STORAGE_KEY)
  return flag === 'true' || flag === true
}

export function setManualLogout(): void {
  uni.setStorageSync(MANUAL_LOGOUT_STORAGE_KEY, 'true')
}

export function clearManualLogout(): void {
  uni.removeStorageSync(MANUAL_LOGOUT_STORAGE_KEY)
}
