// https://developers.weixin.qq.com/miniprogram/dev/api/navigate/wx.openOfficialAccountProfile.html
const OFFICIAL_ACCOUNT_ORIGINAL_ID = 'gh_ab5b67d82e12'

type OfficialAccountApi = {
  openOfficialAccountProfile?: (options: { username: string; fail: () => void }) => void
}

function showManualOpenGuide(): void {
  uni.showModal({
    title: '打开何止编程',
    content: '请在微信中搜索公众号「何止编程」，进入公众号后发送已复制的绑定指令。',
    showCancel: false,
    confirmText: '知道了',
  })
}

export function openUserOfficialAccount(): void {
  // #ifdef MP-WEIXIN
  const api = uni as unknown as OfficialAccountApi
  if (typeof uni.canIUse === 'function' && uni.canIUse('openOfficialAccountProfile')
    && typeof api.openOfficialAccountProfile === 'function') {
    try {
      api.openOfficialAccountProfile({
        username: OFFICIAL_ACCOUNT_ORIGINAL_ID,
        fail: showManualOpenGuide,
      })
    } catch {
      showManualOpenGuide()
    }
    return
  }
  // #endif
  showManualOpenGuide()
}
