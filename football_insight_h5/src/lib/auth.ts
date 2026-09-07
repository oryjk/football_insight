const TOKEN_STORAGE_KEY = 'fi_h5_access_token'

// 鉴权来源：1) URL ?token=xxx（小程序 web-view 嵌入时由小程序透传）2) localStorage 缓存 3) 测试登录写入。
export function initTokenFromUrl(): void {
  const url = new URL(window.location.href)
  const token = (url.searchParams.get('token') || '').trim()
  if (!token) return
  setAccessToken(token)
  url.searchParams.delete('token')
  window.history.replaceState(null, '', url.toString())
}

export function getAccessToken(): string {
  return localStorage.getItem(TOKEN_STORAGE_KEY) || ''
}

export function setAccessToken(token: string): void {
  localStorage.setItem(TOKEN_STORAGE_KEY, token)
}

export function clearAccessToken(): void {
  localStorage.removeItem(TOKEN_STORAGE_KEY)
}
