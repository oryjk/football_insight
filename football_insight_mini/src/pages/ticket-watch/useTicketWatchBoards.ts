import { getCurrentUser } from '../../api/auth'
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
  getCurrentUser,
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
) {
  const {
    getCurrentUser,
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

    try {
      const user = await getCurrentUser()
      currentUser.value = user
      viewerMembershipTier.value = user?.membership_tier?.trim() || 'V1'
    } catch {
      currentUser.value = null
      viewerMembershipTier.value = 'V1'
    }
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

    currentTrackedInterests.value =
      await getTicketWatchTrackedInterests(matchId)
  }

  function resetCurrentBoard(): void {
    currentMatch.value = null
    currentSections.value = []
    currentTrackedInterests.value = []
    refluxSubscriptionSubscribed.value = false
    refluxSubscriptionEmail.value = ''
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
      if (selectedTeam.value === 'yunnanyukun') {
        await loadYukunCurrentBoard()
      } else {
        await loadChengduCurrentBoard()
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

  async function loadChengduCurrentBoard(): Promise<void> {
    await ensureRegions()
    const response = await getCurrentTicketWatchBoard()
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

  async function loadYukunCurrentBoard(): Promise<void> {
    const response = await getYukunCurrentTicketWatchMatch()
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
    cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
    historySectionsCache.value = {
      ...historySectionsCache.value,
      [matchId]: grouped,
    }
    return grouped
  }

  async function getYukunHistoryInventorySections(
    matchId: number,
  ): Promise<TicketWatchGroupedInventorySection[]> {
    const match =
      historyMatches.value.find((item) => item.match_id === matchId) ?? null
    const since = requireInventorySince(match?.sale_start_at)
    const [inventory, yukunRegions] = await Promise.all([
      getYukunTicketWatchInventory(matchId, since),
      getYukunTicketWatchRegions(matchId, since),
    ])
    const grouped = groupInventoryByPrice(yukunRegions, inventory)
    cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
    historySectionsCache.value = {
      ...historySectionsCache.value,
      [matchId]: grouped,
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

    const grouped = await getHistoryInventorySections(matchId)
    displayedHistoryMatchId.value = matchId
    historySections.value = grouped
  }

  async function handleHistoryMatchSelect(matchId: number): Promise<void> {
    if (selectedHistoryMatchId.value === matchId) {
      return
    }

    selectedHistoryMatchId.value = matchId
    const requestToken = ++historySelectionRequestToken
    historySelectionLoading.value = true

    try {
      const grouped = await getHistoryInventorySections(matchId)

      if (
        requestToken !== historySelectionRequestToken ||
        selectedHistoryMatchId.value !== matchId
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
