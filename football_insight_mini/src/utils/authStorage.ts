export const AUTH_TOKEN_STORAGE_KEY = 'fi_access_token'
export const POST_LOGIN_REDIRECT_STORAGE_KEY = 'fi_post_login_redirect'

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
