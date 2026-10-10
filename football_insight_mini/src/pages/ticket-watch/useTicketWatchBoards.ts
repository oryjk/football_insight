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

import type {
  TicketWatchBlockInterest,
  TicketWatchGroupedInventorySection,
  TicketWatchMatchSummary,
} from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import { useAppSession, type AppSession } from '../../stores/appSession'

import {
  applyBlockInterestsToSections,
  groupInventoryByPrice,
  resolveCurrentBoardLoadStrategy,
  resolveHistoryBoardLoadStrategy,
  requireInventorySince,
  selectCompletedMatches,
  selectRefluxStatsMatches,
  type TicketWatchHistoryLoadReason,
  type TicketWatchLoadReason,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

import { resolveFallbackMatchId, sumInventoryOccurrences } from './presentation'

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
    selectedTeam,
    currentLoading,
    historyLoading,
    currentErrorMessage,
    historyErrorMessage,
    currentMessage,
    regions,
    currentMatch,
    historyMatches,
    selectedHistoryMatchId,
    currentSections,
    historySections,
    historyTrendTotals,
    hasLoadedCurrentBoard,
    hasLoadedHistoryBoard,
    currentBoardRequestInFlight,
    historyBoardRequestInFlight,
    historySelectionLoading,
    historyTrendLoading,
    displayedHistoryMatchId,
    historySectionsCache,
    currentUser,
    viewerMembershipTier,
    systemConfigUnderReview,
    refluxSubscriptionEmail,
    refluxSubscriptionSubscribed,
    refluxSubscriptionSheetVisible,
    refluxSubscriptionLoading,
    refluxSubscriptionSubmitting,
    refluxSubscriptionStatusLoading,
    currentTrackedInterests,
  } = state
  let historySelectionRequestToken = 0
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

  async function getMatchInterestSections(
    matchId: number,
  ): Promise<TicketWatchBlockInterest[]> {
    return getTicketWatchBlockInterests(matchId)
  }

  async function loadCurrentTrackedInterests(matchId: number): Promise<void> {
    if (!currentUser.value) {
      currentTrackedInterests.value = []
      return
    }

    // 兴趣区块是账号私有数据：旧账号的结果晚到时不发布。
    const requestRevision = session.revision.value
    const interests = await getTicketWatchTrackedInterests(matchId)
    if (requestRevision !== session.revision.value) {
      return
    }
    currentTrackedInterests.value = interests
  }

  function resetCurrentBoard(): void {
    currentMatch.value = null
    currentSections.value = []
    currentTrackedInterests.value = []
    refluxSubscriptionSubscribed.value = false
    refluxSubscriptionEmail.value = ''
    // 球队/比赛切换使旧订阅请求的快照失效、其 finally 跳过清理：
    // loading/sheet 标记在这里复位，否则后续刷新与开 sheet 一直被在途标记挡住。
    refluxSubscriptionStatusLoading.value = false
    refluxSubscriptionLoading.value = false
    refluxSubscriptionSubmitting.value = false
    refluxSubscriptionSheetVisible.value = false
    currentErrorMessage.value = ''
    hasLoadedCurrentBoard.value = false
    currentMessage.value = '暂无当前比赛。'
  }

  function resetHistoryBoard(): void {
    historyMatches.value = []
    historySections.value = []
    historySectionsCache.value = {}
    historyTrendTotals.value = {}
    selectedHistoryMatchId.value = null
    displayedHistoryMatchId.value = null
    historyErrorMessage.value = ''
    hasLoadedHistoryBoard.value = false
  }

  async function loadCurrentBoard(
    reason: TicketWatchLoadReason = 'initial',
  ): Promise<void> {
    if (currentBoardRequestInFlight.value) {
      return
    }

    const strategy = resolveCurrentBoardLoadStrategy(
      hasLoadedCurrentBoard.value,
      reason,
    )
    currentBoardRequestInFlight.value = true

    if (strategy.showBlockingLoading) {
      currentLoading.value = true
    }

    if (strategy.clearErrorBeforeLoad) {
      currentErrorMessage.value = ''
    }

    try {
      const requestRevision = session.revision.value
      if (selectedTeam.value === 'yunnanyukun') {
        await loadYukunCurrentBoard(requestRevision)
      } else {
        await loadChengduCurrentBoard(requestRevision)
      }
      if (requestRevision !== session.revision.value) {
        // 账号在看板请求期间切换/退出：revision watch 已清理状态，旧结果不标记已加载。
        return
      }
      hasLoadedCurrentBoard.value = true
    } catch (error) {
      if (reason === 'initial' || !hasLoadedCurrentBoard.value) {
        currentErrorMessage.value = extractApiErrorMessage(
          error,
          '当前比赛回流加载失败，请稍后重试。',
        )
      }
    } finally {
      currentLoading.value = false
      currentBoardRequestInFlight.value = false
    }
  }

  async function loadChengduCurrentBoard(requestRevision: number): Promise<void> {
    await ensureRegions()
    const response = await getCurrentTicketWatchBoard()
    if (requestRevision !== session.revision.value) {
      return
    }
    currentMatch.value = response.current_match
    currentMessage.value =
      response.message ||
      (response.group_ticket_active ? '当前为套票窗口。' : '暂无当前比赛。')

    if (response.current_match) {
      currentSections.value = applyBlockInterestsToSections(
        groupInventoryByPrice(regions.value, response.inventory),
        response.block_interests,
      )
      currentTrackedInterests.value = response.tracked_interests
      void refreshRefluxSubscriptionStatus()
    } else {
      currentSections.value = []
      currentTrackedInterests.value = []
      refluxSubscriptionSubscribed.value = false
      refluxSubscriptionEmail.value = ''
    }
  }

  async function loadYukunCurrentBoard(requestRevision: number): Promise<void> {
    const response = await getYukunCurrentTicketWatchMatch()
    if (requestRevision !== session.revision.value) {
      return
    }
    const yukunMatch = response.current_match

    if (!yukunMatch) {
      currentMatch.value = null
      currentMessage.value = response.message || '暂无云南玉昆的当前比赛。'
      currentSections.value = []
      currentTrackedInterests.value = []
      return
    }

    currentMatch.value = yukunMatch
    currentMessage.value = ''
    void refreshRefluxSubscriptionStatus()
    const since = requireInventorySince(yukunMatch.sale_start_at)

    const [inventory, yukunRegions] = await Promise.all([
      getYukunTicketWatchInventory(yukunMatch.match_id, since),
      getYukunTicketWatchRegions(yukunMatch.match_id, since),
    ])
    if (requestRevision !== session.revision.value) {
      return
    }

    currentSections.value = groupInventoryByPrice(yukunRegions, inventory)
    currentTrackedInterests.value = []
  }

  async function loadHistoryMatches(
    reason: TicketWatchHistoryLoadReason = 'initial',
  ): Promise<void> {
    if (historyBoardRequestInFlight.value) {
      return
    }

    const strategy = resolveHistoryBoardLoadStrategy(
      hasLoadedHistoryBoard.value,
      reason,
    )
    historyBoardRequestInFlight.value = true

    if (strategy.showBlockingLoading) {
      historyLoading.value = true
    }

    if (strategy.clearErrorBeforeLoad) {
      historyErrorMessage.value = ''
    }

    try {
      if (selectedTeam.value === 'yunnanyukun') {
        await loadYukunHistoryMatches()
      } else {
        await loadChengduHistoryMatches()
      }
      hasLoadedHistoryBoard.value = true
    } catch (error) {
      historyErrorMessage.value = extractApiErrorMessage(
        error,
        '历史比赛库存加载失败，请稍后重试。',
      )
    } finally {
      historyLoading.value = false
      historyBoardRequestInFlight.value = false
    }
  }

  async function loadChengduHistoryMatches(): Promise<void> {
    await ensureRegions()
    const matches = await getTicketWatchMatches()
    historyMatches.value = selectCompletedMatches(matches)
    void loadHistoryRefluxTrendTotals(
      selectRefluxStatsMatches(historyMatches.value),
    )

    if (!historyMatches.value.length) {
      selectedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    if (
      selectedHistoryMatchId.value === null ||
      !historyMatches.value.some(
        (match) => match.match_id === selectedHistoryMatchId.value,
      )
    ) {
      selectedHistoryMatchId.value = historyMatches.value[0].match_id
    }

    await syncDisplayedHistoryMatch(selectedHistoryMatchId.value)
  }

  async function loadYukunHistoryMatches(): Promise<void> {
    historyMatches.value = selectCompletedMatches(
      await getYukunTicketWatchMatches(),
    )

    void loadHistoryRefluxTrendTotals(
      selectRefluxStatsMatches(historyMatches.value),
    )

    if (!historyMatches.value.length) {
      selectedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    if (
      selectedHistoryMatchId.value === null ||
      !historyMatches.value.some(
        (match) => match.match_id === selectedHistoryMatchId.value,
      )
    ) {
      selectedHistoryMatchId.value = historyMatches.value[0].match_id
    }

    await syncDisplayedHistoryMatch(selectedHistoryMatchId.value)
  }

  async function getHistoryInventorySections(
    matchId: number,
  ): Promise<TicketWatchGroupedInventorySection[]> {
    const cached = historySectionsCache.value[matchId]
    if (cached) {
      return cached
    }

    if (selectedTeam.value === 'yunnanyukun') {
      return getYukunHistoryInventorySections(matchId)
    }

    const match =
      historyMatches.value.find((item) => item.match_id === matchId) ?? null
    // sections 混合了账号私有的兴趣标记：revision 变化后不再写入共享缓存。
    const requestRevision = session.revision.value
    const [inventory, interests] = await Promise.all([
      getTicketWatchInventorySince(
        matchId,
        requireInventorySince(match?.sale_start_at),
        resolveFallbackMatchId(match),
      ),
      getMatchInterestSections(matchId),
    ])
    const grouped = applyBlockInterestsToSections(
      groupInventoryByPrice(regions.value, inventory),
      interests,
    )
    // trend totals 是纯公共库存数据，不做 revision 防护。
    cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
    if (requestRevision === session.revision.value) {
      historySectionsCache.value = {
        ...historySectionsCache.value,
        [matchId]: grouped,
      }
    }
    return grouped
  }

  async function getYukunHistoryInventorySections(
    matchId: number,
  ): Promise<TicketWatchGroupedInventorySection[]> {
    const match =
      historyMatches.value.find((item) => item.match_id === matchId) ?? null
    const since = requireInventorySince(match?.sale_start_at)
    const requestRevision = session.revision.value
    const [inventory, yukunRegions] = await Promise.all([
      getYukunTicketWatchInventory(matchId, since),
      getYukunTicketWatchRegions(matchId, since),
    ])
    const grouped = groupInventoryByPrice(yukunRegions, inventory)
    cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
    if (requestRevision === session.revision.value) {
      historySectionsCache.value = {
        ...historySectionsCache.value,
        [matchId]: grouped,
      }
    }
    return grouped
  }

  function cacheHistoryTrendTotal(
    matchId: number,
    totalOccurrences: number,
  ): void {
    if (historyTrendTotals.value[matchId] === totalOccurrences) {
      return
    }

    historyTrendTotals.value = {
      ...historyTrendTotals.value,
      [matchId]: totalOccurrences,
    }
  }

  async function loadHistoryRefluxTrendTotals(
    matches: TicketWatchMatchSummary[],
  ): Promise<void> {
    if (historyTrendLoading.value) {
      return
    }

    const targetMatches = matches
      .slice(0, 12)
      .filter((match) => historyTrendTotals.value[match.match_id] === undefined)

    if (!targetMatches.length) {
      return
    }

    historyTrendLoading.value = true

    try {
      const isYukun = selectedTeam.value === 'yunnanyukun'
      for (let i = 0; i < targetMatches.length; i += 3) {
        const batch = targetMatches.slice(i, i + 3)
        await Promise.all(
          batch.map(async (match) => {
            const inventory = isYukun
              ? await getYukunTicketWatchInventory(
                  match.match_id,
                  requireInventorySince(match.sale_start_at),
                )
              : await getTicketWatchInventorySince(
                  match.match_id,
                  requireInventorySince(match.sale_start_at),
                  resolveFallbackMatchId(match),
                )
            cacheHistoryTrendTotal(
              match.match_id,
              sumInventoryOccurrences(inventory),
            )
          }),
        )
      }
    } catch {
    } finally {
      historyTrendLoading.value = false
    }
  }

  async function syncDisplayedHistoryMatch(
    matchId: number | null,
  ): Promise<void> {
    if (matchId === null) {
      displayedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // 内部请求跨多个 await：发布前核对 revision，旧会话的结果不覆盖新会话视图。
    const requestRevision = session.revision.value
    const grouped = await getHistoryInventorySections(matchId)
    if (requestRevision !== session.revision.value) {
      return
    }
    displayedHistoryMatchId.value = matchId
    historySections.value = grouped
  }

  async function handleHistoryMatchSelect(matchId: number): Promise<void> {
    if (selectedHistoryMatchId.value === matchId) {
      return
    }

    selectedHistoryMatchId.value = matchId
    const requestToken = ++historySelectionRequestToken
    const requestRevision = session.revision.value
    historySelectionLoading.value = true

    try {
      const grouped = await getHistoryInventorySections(matchId)

      if (
        requestToken !== historySelectionRequestToken ||
        selectedHistoryMatchId.value !== matchId ||
        requestRevision !== session.revision.value
      ) {
        return
      }

      displayedHistoryMatchId.value = matchId
      historySections.value = grouped
    } catch (error) {
      if (requestToken !== historySelectionRequestToken) {
        return
      }

      uni.showToast({
        title: extractApiErrorMessage(
          error,
          '该场比赛库存加载失败，请稍后重试。',
        ),
        icon: 'none',
      })
    } finally {
      if (requestToken === historySelectionRequestToken) {
        historySelectionLoading.value = false
      }
    }
  }

  return {
    ensureRegions,
    loadViewerMembershipTier,
    loadCurrentTrackedInterests,
    loadCurrentBoard,
    loadHistoryMatches,
    resetCurrentBoard,
    resetHistoryBoard,
    handleHistoryMatchSelect,
  }
}
