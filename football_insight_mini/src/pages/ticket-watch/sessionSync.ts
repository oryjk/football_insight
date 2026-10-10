import type { AppSession } from '../../stores/appSession'

import type { TicketWatchLoadReason } from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

export interface TicketWatchSessionSyncDeps {
  /** 复位看板/订阅/解锁等全部会话私有状态（页面保持所有权的 reset 集合）。 */
  resetSessionScopedBoards: () => void
  loadViewerMembershipTier: () => Promise<void>
  loadCurrentBoard: (reason: TicketWatchLoadReason) => Promise<void>
}

export interface TicketWatchSessionSync {
  /**
   * 页面 watch(session.revision) 的处理入口：会话被替代（退出/新登录/失效）时
   * 立即丢弃旧账号私有看板数据；后台静默恢复/登录发布已认证用户后，
   * 可见页面直接重载当前看板（档位/订阅/兴趣随请求刷新），无需再次切 Tab。
   * 不触碰监控开关与轮询：恢复不会自动开启用户未开启的监控，隐藏页面不启动轮询。
   */
  handleSessionRevisionChange(): void
}

export function createTicketWatchSessionSync(
  state: TicketWatchState,
  session: AppSession,
  deps: TicketWatchSessionSyncDeps,
): TicketWatchSessionSync {
  async function reloadAfterSessionRestore(): Promise<void> {
    await deps.loadViewerMembershipTier()
    // 会员权益锁定（取关公众号）时维持锁屏，不加载看板。
    if (state.membershipBenefitsLocked.value) {
      return
    }
    await deps.loadCurrentBoard('initial')
  }

  function handleSessionRevisionChange(): void {
    deps.resetSessionScopedBoards()
    // 只有「已确认用户被发布」（恢复/登录成功）才重载；退出/失效保持清空后的游客视图。
    // 审核中不加载会员相关看板数据。
    if (session.currentUser.value && !state.systemConfigUnderReview.value) {
      void reloadAfterSessionRestore()
    }
  }

  return { handleSessionRevisionChange }
}
