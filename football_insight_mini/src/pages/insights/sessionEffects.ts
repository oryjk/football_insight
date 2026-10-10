import type { CurrentUser } from '../../types/auth'

export interface InsightsSessionSync {
  /**
   * 页面 watch(session.currentUser) 的处理入口：后台静默恢复/登录发布已认证用户后，
   * 直接加载会员洞察数据，无需再次切 Tab；退出清空用户时不加载（模板已切游客视图）；
   * 审核中不拉取会员数据。
   */
  handleSessionUserPublished(user: CurrentUser | null): void
  /**
   * 页面 watch(session.revision) 的处理入口：会话身份变化（退出/新登录意图/失效/
   * 业务 401）时同步丢弃旧账号私有缓存/选择/展开/错误/loading，跨账号不保留旧数据；
   * 正常同账号刷新不触发（revision 不变），缓存保留。
   */
  handleSessionRevisionChange(): void
}

export function createInsightsSessionSync(deps: {
  isUnderReview: () => boolean
  loadInsights: () => Promise<void>
  resetSessionScopedState: () => void
}): InsightsSessionSync {
  return {
    handleSessionUserPublished(user) {
      if (user && !deps.isUnderReview()) {
        void deps.loadInsights()
      }
    },
    handleSessionRevisionChange() {
      deps.resetSessionScopedState()
    },
  }
}
