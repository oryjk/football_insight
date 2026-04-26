import {
  getPostLoginRedirectTarget,
  setPostLoginRedirectTarget,
  type PostLoginRedirectTarget,
} from './authStorage'

export function rememberPostLoginRedirect(target: PostLoginRedirectTarget): void {
  setPostLoginRedirectTarget(target)
}

export function consumePostLoginRedirect(): PostLoginRedirectTarget | null {
  const target = getPostLoginRedirectTarget()
  setPostLoginRedirectTarget(null)
  return target
}

export function navigateToPostLoginTarget(target: PostLoginRedirectTarget): Promise<void> {
  return new Promise((resolve, reject) => {
    const fail = (error: { errMsg?: string }) => reject(new Error(error.errMsg || '登录后跳转失败'))

    if (target.type === 'switchTab') {
      uni.switchTab({
        url: target.url,
        success: () => resolve(),
        fail,
      })
      return
    }

    uni.navigateTo({
      url: target.url,
      success: () => resolve(),
      fail,
    })
  })
}
