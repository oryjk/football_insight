import type { CurrentUser } from '../../types/auth'

export interface InsightsSessionSync {
  /**
   * 页面 watch(session.currentUser) 的处理入口：后台静默恢复/登录发布已认证用户后，
   * 直接加载会员洞察数据，无需再次切 Tab；退出清空用户时不加载（模板已切游客视图）；
   * 审核中不拉取会员数据。
   */
  handleSessionUserPublished(user: CurrentUser | null): void
}

export function createInsightsSessionSync(deps: {
  isUnderReview: () => boolean
  loadInsights: () => Promise<void>
}): InsightsSessionSync {
  return {
    handleSessionUserPublished(user) {
      if (user && !deps.isUnderReview()) {
        void deps.loadInsights()
      }
    },
  }
}
