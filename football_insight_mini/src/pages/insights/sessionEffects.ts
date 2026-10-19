import type { Ref } from 'vue'
import type { CurrentUser } from '../../types/auth'

export interface InsightsSessionSync {
  /**
   * 页面 watch(session.currentUser) 的处理入口：后台静默恢复/登录发布已认证用户后，
   * 直接加载会员洞察数据，无需再次切 Tab；退出清空用户时不加载（模板已切游客视图）。
   * 审核状态不影响会员洞察：审核中只隐藏回流看板入口与赛前倾向卡片（由页面模板处理）。
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
  loadInsights: () => Promise<void>
  resetSessionScopedState: () => void
}): InsightsSessionSync {
  return {
    handleSessionUserPublished(user) {
      if (user) {
        void deps.loadInsights()
      }
    },
    handleSessionRevisionChange() {
      deps.resetSessionScopedState()
    },
  }
}

/**
 * 洞察页 onShow 编排：先查审核状态并发布（与账号无关，任何情况都发布），再在同一会话内
 * loadUser → loadInsights。操作身份在第一个 await 之前建立；审核查询或 loadUser 等待期间
 * 会话被替代（切账号/退出/失效）时，旧 onShow 不再续私有请求，新会话由它自己的 watcher 负责。
 * 审核中不再中断加载：审核只隐藏回流看板入口与赛前倾向卡片。
 */
export async function runInsightsShow(deps: {
  revision: Ref<number>
  loadUnderReview: () => Promise<boolean>
  publishUnderReview: (underReview: boolean) => void
  loadUser: () => Promise<boolean>
  loadInsights: () => Promise<void>
}): Promise<void> {
  const shownRevision = deps.revision.value
  const underReview = await deps.loadUnderReview()
  deps.publishUnderReview(underReview)
  if (deps.revision.value !== shownRevision) {
    return
  }
  const stillCurrent = await deps.loadUser()
  if (!stillCurrent) {
    return
  }
  await deps.loadInsights()
}

/**
 * 页面每次 onShow 时赛前倾向区域（球队胶囊、倾向卡片）的就绪状态：
 * 首次进入还不知道审核状态，先隐藏；之后回到页面（如从战术板返回）审核状态已确认过且不在审核，
 * 保持显示、后台更新——无条件置 false 会让这些内容消失再出现，页面闪一下。
 * 审核优先：已知处于审核中时仍先隐藏，不为防闪烁冒险暴露审核期内容。
 */
export function resolveOutlookReadyOnShow(input: { alreadyConfirmed: boolean; underReview: boolean }): boolean {
  return input.alreadyConfirmed && !input.underReview
}
