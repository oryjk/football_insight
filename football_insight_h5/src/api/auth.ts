import { request } from '../lib/request'
import { setAccessToken } from '../lib/auth'
import type { H5TestLoginUser } from '../types/auth'

// 与后端 H5 测试登录白名单（H5_TEST_LOGIN_USER_IDS）配套，仅测试环境可用；未配置时接口 403。
export function listH5TestLoginUsers(): Promise<{ items: H5TestLoginUser[] }> {
  return request<{ items: H5TestLoginUser[] }>({ url: '/auth/h5-test-login/users' })
}

export async function loginAsH5TestUser(userId: string): Promise<void> {
  const response = await request<{ access_token: string }>({
    url: '/auth/h5-test-login',
    method: 'POST',
    data: { user_id: userId },
  })
  setAccessToken(response.access_token)
}
