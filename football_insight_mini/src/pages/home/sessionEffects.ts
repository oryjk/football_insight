import type { CurrentUser } from '../../types/auth'

export interface HomeSessionSync {
  /**
   * 页面 watch(session.currentUser) 的处理入口：后台静默恢复/登录发布用户、
   * 或退出/失效清空用户后，刷新助力面板（账号私有数据），无需重新进入页面。
   * 公开数据（overview/榜单）不依赖会话，不受影响。
   */
  handleSessionUserPublished(user: CurrentUser | null): void
  /**
   * 页面 watch(session.revision) 的处理入口：会话身份变化（退出/新登录意图/失效/
   * 业务 401）时同步丢弃旧账号私有缓存/选择/弹层，跨账号不保留旧数据；
   * 正常同账号刷新不触发（revision 不变），缓存保留。
   */
  handleSessionRevisionChange(): void
}

export function createHomeSessionSync(deps: {
  loadSupportData: () => Promise<void>
  resetSessionScopedState: () => void
}): HomeSessionSync {
  return {
    handleSessionUserPublished(_user) {
      void deps.loadSupportData()
    },
    handleSessionRevisionChange() {
      deps.resetSessionScopedState()
    },
  }
}
