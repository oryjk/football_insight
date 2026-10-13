export function goBackFromSecondaryPage(fallbackTab = '/pages/home/index'): void {
  if (getCurrentPages().length <= 1) {
    uni.switchTab({ url: fallbackTab })
    return
  }
  uni.navigateBack({ delta: 1, fail: () => uni.switchTab({ url: fallbackTab }) })
}

/** H5 使用公开页面路径，不携带登录 token 或其他账号数据。 */
export function copyPageShareLink(path: string): void {
  uni.setClipboardData({
    data: `https://match.oryjk.cn/football/#${path}`,
    success: () => uni.showToast({ title: '链接已复制', icon: 'none' }),
  })
}
