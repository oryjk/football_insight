import type { CurrentUser } from '../../types/auth'

export interface HomeSessionSync {
  /**
   * 页面 watch(session.currentUser) 的处理入口：后台静默恢复/登录发布用户、
   * 或退出/失效清空用户后，刷新助力面板（账号私有数据），无需重新进入页面。
   * 公开数据（overview/榜单）不依赖会话，不受影响。
   */
  handleSessionUserPublished(user: CurrentUser | null): void
}

export function createHomeSessionSync(deps: {
  loadSupportData: () => Promise<void>
}): HomeSessionSync {
  return {
    handleSessionUserPublished(_user) {
      void deps.loadSupportData()
    },
  }
}
