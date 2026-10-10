import type { AppSession } from '../../stores/appSession'

import type {
  TicketWatchHistoryLoadReason,
  TicketWatchLoadReason,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

export interface TicketWatchSessionSyncDeps {
  /** 复位看板/订阅/解锁等全部会话私有状态（页面保持所有权的 reset 集合）。 */
  resetSessionScopedBoards: () => void
  loadViewerMembershipTier: () => Promise<void>
  loadCurrentBoard: (reason: TicketWatchLoadReason) => Promise<void>
  /** 历史看板重载（history / history-stats 可见时按新会话重新读取）。 */
  loadHistoryMatches: (reason: TicketWatchHistoryLoadReason) => Promise<void>
  /** 页面可见身份：页面隐藏/卸载后恢复链不再继续发起读取。 */
  isPageVisible: () => boolean
}

export interface TicketWatchSessionSync {
  /**
   * 页面 watch(session.revision) 的处理入口：会话被替代（退出/新登录/失效）时
   * 立即丢弃旧账号私有看板数据；后台静默恢复/登录发布已认证用户后，
   * 可见页面按当前 activeTab 重载被清空的看板（current → 当前比赛；
   * history / history-stats → 历史列表，选中/缓存/trend 随加载重建），无需再次切 Tab。
   * 不触碰监控开关与轮询：恢复不会自动开启用户未开启的监控，隐藏页面不发起重载、不启动轮询。
   */
  handleSessionRevisionChange(): void
}

export function createTicketWatchSessionSync(
  state: TicketWatchState,
  session: AppSession,
  deps: TicketWatchSessionSyncDeps,
): TicketWatchSessionSync {
  async function reloadAfterSessionRestore(): Promise<void> {
    // 恢复链跨多个 await：捕获会话 revision / 球队 / 页面可见身份，
    // 任一身份变化（退出/切换/隐藏）都终止旧 reload，不继续发起读取。
    const requestRevision = session.revision.value
    const requestTeam = state.selectedTeam.value
    const isCurrentContext = () =>
      session.revision.value === requestRevision &&
      state.selectedTeam.value === requestTeam &&
      deps.isPageVisible()

    await deps.loadViewerMembershipTier()
    if (!isCurrentContext()) {
      return
    }
    // 会员权益锁定（取关公众号）时维持锁屏，不加载看板。
    if (state.membershipBenefitsLocked.value) {
      return
    }
    // 只重载当前可见 tab 对应的看板；另一块已清空的数据留待下次 onShow 按新会话加载。
    if (state.activeTab.value === 'current') {
      await deps.loadCurrentBoard('initial')
    } else {
      await deps.loadHistoryMatches('initial')
    }
  }

  function handleSessionRevisionChange(): void {
    deps.resetSessionScopedBoards()
    // 只有「已确认用户被发布」（恢复/登录成功）才重载；退出/失效保持清空后的游客视图。
    // 审核中不加载会员相关看板数据；隐藏页面不在后台发起重载（下次 onShow 统一加载）。
    if (
      session.currentUser.value &&
      !state.systemConfigUnderReview.value &&
      deps.isPageVisible()
    ) {
      void reloadAfterSessionRestore()
    }
  }

  return { handleSessionRevisionChange }
}
