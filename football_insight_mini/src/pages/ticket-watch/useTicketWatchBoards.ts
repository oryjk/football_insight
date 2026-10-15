import {
  getCurrentTicketWatchBoard,
  getTicketWatchBlockInterests,
  getTicketWatchInventorySince,
  getTicketWatchMatches,
  getTicketWatchRegions,
  getTicketWatchTrackedInterests,
  getYukunTicketWatchMatches,
  getYukunCurrentTicketWatchMatch,
  getYukunTicketWatchInventory,
  getYukunTicketWatchRegions,
} from '../../api/ticketWatch'
import { useAppSession, type AppSession } from '../../stores/appSession'
import type { TicketWatchState } from './useTicketWatchState'
import type { TicketWatchTab } from './types'
import { createTicketWatchBoardRequests } from './ticketWatchBoardRequests'
import { createTicketWatchCurrentBoard } from './ticketWatchCurrentBoard'
import { createTicketWatchHistoryBoard } from './ticketWatchHistoryBoard'

const boardApi = {
  getCurrentTicketWatchBoard,
  getTicketWatchBlockInterests,
  getTicketWatchInventorySince,
  getTicketWatchMatches,
  getTicketWatchRegions,
  getTicketWatchTrackedInterests,
  getYukunTicketWatchMatches,
  getYukunCurrentTicketWatchMatch,
  getYukunTicketWatchInventory,
  getYukunTicketWatchRegions,
}

export function useTicketWatchBoards(
  state: TicketWatchState,
  refreshRefluxSubscriptionStatus: () => Promise<void>,
  api = boardApi,
  session: AppSession = useAppSession(),
) {
  const {
    getCurrentTicketWatchBoard,
    getTicketWatchBlockInterests,
    getTicketWatchInventorySince,
    getTicketWatchMatches,
    getTicketWatchRegions,
    getTicketWatchTrackedInterests,
    getYukunTicketWatchMatches,
    getYukunCurrentTicketWatchMatch,
    getYukunTicketWatchInventory,
    getYukunTicketWatchRegions,
  } = api
  const {
    regions,
    hasLoadedCurrentBoard,
    hasLoadedHistoryBoard,
    currentUser,
    viewerMembershipTier,
    systemConfigUnderReview,
    membershipBenefitsLocked,
  } = state
  const capturedApi = {
    getCurrentTicketWatchBoard,
    getTicketWatchBlockInterests,
    getTicketWatchInventorySince,
    getTicketWatchMatches,
    getTicketWatchRegions,
    getTicketWatchTrackedInterests,
    getYukunTicketWatchMatches,
    getYukunCurrentTicketWatchMatch,
    getYukunTicketWatchInventory,
    getYukunTicketWatchRegions,
  }
  const requests = createTicketWatchBoardRequests()
  const {
    loadCurrentTrackedInterests,
    loadCurrentBoard,
    resetCurrentBoard,
  } = createTicketWatchCurrentBoard(
    state,
    capturedApi,
    session,
    requests,
    ensureRegions,
    refreshRefluxSubscriptionStatus,
  )
  const {
    loadHistoryMatches,
    resetHistoryBoard,
    handleHistoryMatchSelect,
  } = createTicketWatchHistoryBoard(
    state, capturedApi, session, requests, ensureRegions,
  )

  async function ensureRegions(): Promise<void> {
    if (regions.value.length) {
      return
    }

    regions.value = await getTicketWatchRegions()
  }

  async function loadViewerMembershipTier(): Promise<void> {
    if (systemConfigUnderReview.value) {
      currentUser.value = null
      viewerMembershipTier.value = 'V1'
      return
    }

    // 用户/会员档位共享自 appSession；捕获 revision，旧账号结果不发布。
    const requestRevision = session.revision.value
    try {
      await session.ensureSessionReady()
    } catch {
      // 断网/5xx：session 保留 token 与已确认用户并记录可重试错误，
      // 本地镜像沿用 session 当前用户，不清会话。
    }
    if (requestRevision !== session.revision.value) {
      return
    }
    currentUser.value = session.currentUser.value
    viewerMembershipTier.value = session.currentUser.value?.membership_tier?.trim() || 'V1'
  }

  /**
   * 页内 tab 切换的补载入口（onShow 不会因页内 tab 切换再触发）：
   * 进入尚未加载的看板时按当前会话加载；已加载的看板复用，不重复请求。
   * history / history-stats 共用历史看板数据；不触碰监控开关与轮询。
   */
  function loadBoardForTab(tab: TicketWatchTab): void {
    if (systemConfigUnderReview.value || membershipBenefitsLocked.value) {
      return
    }
    if (tab === 'current') {
      if (!hasLoadedCurrentBoard.value) {
        void loadCurrentBoard('initial')
      }
    } else if (!hasLoadedHistoryBoard.value) {
      void loadHistoryMatches('initial')
    }
  }

  return {
    ensureRegions,
    loadViewerMembershipTier,
    loadCurrentTrackedInterests,
    loadCurrentBoard,
    loadHistoryMatches,
    loadBoardForTab,
    resetCurrentBoard,
    resetHistoryBoard,
    handleHistoryMatchSelect,
  }
}
