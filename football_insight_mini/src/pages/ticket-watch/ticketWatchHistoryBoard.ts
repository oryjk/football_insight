import type {
  TicketWatchBlockInterest,
  TicketWatchGroupedInventorySection,
  TicketWatchMatchSummary,
} from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import type { AppSession } from '../../stores/appSession'
import {
  applyBlockInterestsToSections,
  groupInventoryByPrice,
  resolveHistoryBoardLoadStrategy,
  requireInventorySince,
  selectCompletedMatches,
  selectRefluxStatsMatches,
  type TicketWatchHistoryLoadReason,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'
import { resolveFallbackMatchId, sumInventoryOccurrences } from './presentation'
import type { TicketWatchBoardApi } from './ticketWatchBoardTypes'
import type { TicketWatchBoardRequests } from './ticketWatchBoardRequests'

export function createTicketWatchHistoryBoard(
  state: TicketWatchState,
  api: TicketWatchBoardApi,
  session: AppSession,
  requests: TicketWatchBoardRequests,
  ensureRegions: () => Promise<void>,
) {
  const {
    getTicketWatchBlockInterests,
    getTicketWatchInventorySince,
    getTicketWatchMatches,
    getYukunTicketWatchMatches,
    getYukunTicketWatchInventory,
    getYukunTicketWatchRegions,
  } = api
  const {
    selectedTeam,
    historyLoading,
    historyErrorMessage,
    regions,
    historyMatches,
    selectedHistoryMatchId,
    historySections,
    historyTrendTotals,
    hasLoadedHistoryBoard,
    historyBoardRequestInFlight,
    historySelectionLoading,
    historyTrendLoading,
    displayedHistoryMatchId,
    historySectionsCache,
  } = state

  async function getMatchInterestSections(
    matchId: number,
  ): Promise<TicketWatchBlockInterest[]> {
    return getTicketWatchBlockInterests(matchId)
  }

  function resetHistoryBoard(): void {
    // 作废在途的历史列表/选择/trend 请求：旧请求的成功/catch/finally
    // 不得覆盖新上下文、不写共享缓存、不清新请求的 loading。
    requests.historyBoardRequestSeq += 1
    requests.activeHistoryBoardRequest = null
    historyBoardRequestInFlight.value = false
    historyLoading.value = false
    requests.historySelectionRequestToken += 1
    historySelectionLoading.value = false
    requests.historyTrendRequestSeq += 1
    historyTrendLoading.value = false
    historyMatches.value = []
    historySections.value = []
    historySectionsCache.value = {}
    historyTrendTotals.value = {}
    selectedHistoryMatchId.value = null
    displayedHistoryMatchId.value = null
    historyErrorMessage.value = ''
    hasLoadedHistoryBoard.value = false
  }

  async function loadHistoryMatches(
    reason: TicketWatchHistoryLoadReason = 'initial',
  ): Promise<void> {
    const requestRevision = session.revision.value
    const requestTeam = selectedTeam.value
    // 同会话同球队的在途请求直接复用；reset/球队切换后的新上下文允许立即重载。
    const active = requests.activeHistoryBoardRequest
    if (
      active &&
      active.revision === requestRevision &&
      active.team === requestTeam
    ) {
      return
    }
    const owner = ++requests.historyBoardRequestSeq
    requests.activeHistoryBoardRequest = { owner, revision: requestRevision, team: requestTeam }
    const isOwner = () => requests.activeHistoryBoardRequest?.owner === owner

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
      if (requestTeam === 'yunnanyukun') {
        await loadYukunHistoryMatches(owner, requestRevision)
      } else {
        await loadChengduHistoryMatches(owner, requestRevision)
      }
      if (!isOwner() || requestRevision !== session.revision.value) {
        return
      }
      hasLoadedHistoryBoard.value = true
    } catch (error) {
      if (!isOwner() || requestRevision !== session.revision.value) {
        return
      }
      historyErrorMessage.value = extractApiErrorMessage(
        error,
        '历史比赛库存加载失败，请稍后重试。',
      )
    } finally {
      if (isOwner()) {
        requests.activeHistoryBoardRequest = null
        historyLoading.value = false
        historyBoardRequestInFlight.value = false
      }
    }
  }

  async function loadChengduHistoryMatches(
    owner: number,
    requestRevision: number,
  ): Promise<void> {
    const isStale = () =>
      requests.activeHistoryBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    await ensureRegions()
    const matches = await getTicketWatchMatches()
    if (isStale()) {
      return
    }
    historyMatches.value = selectCompletedMatches(matches)
    const candidates = selectRefluxStatsMatches(historyMatches.value)
    void loadHistoryRefluxTrendTotals(candidates)

    await syncInitialHistorySelection(candidates, isStale)
  }

  async function loadYukunHistoryMatches(
    owner: number,
    requestRevision: number,
  ): Promise<void> {
    const isStale = () =>
      requests.activeHistoryBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    const matches = await getYukunTicketWatchMatches()
    if (isStale()) {
      return
    }
    historyMatches.value = selectCompletedMatches(matches)

    const candidates = selectRefluxStatsMatches(historyMatches.value, '云南玉昆')
    void loadHistoryRefluxTrendTotals(candidates)

    await syncInitialHistorySelection(candidates, isStale)
  }

  async function syncInitialHistorySelection(
    candidates: TicketWatchMatchSummary[],
    isStale: () => boolean,
  ): Promise<void> {
    if (!candidates.length) {
      // Invalidate an in-flight selection when the refreshed list has no eligible games.
      requests.historySelectionRequestToken += 1
      historySelectionLoading.value = false
      selectedHistoryMatchId.value = null
      displayedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // Keep the candidate source for loading unknown totals, but never default to excluded clubs.
    if (!candidates.some(match => match.match_id === selectedHistoryMatchId.value)) {
      selectedHistoryMatchId.value = candidates[0].match_id
    }
    // Preserve the shared selection owner even when the list keeps the same selection.
    await syncDisplayedHistoryMatch(selectedHistoryMatchId.value, isStale)
  }

  async function getHistoryInventorySections(
    matchId: number,
    isStale: () => boolean = () => false,
  ): Promise<TicketWatchGroupedInventorySection[]> {
    const cached = historySectionsCache.value[matchId]
    if (cached) {
      return cached
    }

    if (selectedTeam.value === 'yunnanyukun') {
      return getYukunHistoryInventorySections(matchId, isStale)
    }

    const match =
      historyMatches.value.find((item) => item.match_id === matchId) ?? null
    // sections 混合了账号私有的兴趣标记：上下文失效后不再写入共享缓存。
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
    if (!isStale() && requestRevision === session.revision.value) {
      cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
      historySectionsCache.value = {
        ...historySectionsCache.value,
        [matchId]: grouped,
      }
    }
    return grouped
  }

  async function getYukunHistoryInventorySections(
    matchId: number,
    isStale: () => boolean = () => false,
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
    if (!isStale() && requestRevision === session.revision.value) {
      cacheHistoryTrendTotal(matchId, sumInventoryOccurrences(inventory))
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
    // trend 请求 owner：reset 作废旧请求，其迟到写入与 finally 不影响新上下文。
    const owner = ++requests.historyTrendRequestSeq
    const requestRevision = session.revision.value

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
            if (
              owner !== requests.historyTrendRequestSeq ||
              requestRevision !== session.revision.value
            ) {
              return
            }
            cacheHistoryTrendTotal(
              match.match_id,
              sumInventoryOccurrences(inventory),
            )
          }),
        )
      }
    } catch {
    } finally {
      if (owner === requests.historyTrendRequestSeq) {
        historyTrendLoading.value = false
      }
    }
  }

  async function syncDisplayedHistoryMatch(
    matchId: number | null,
    isStale: () => boolean = () => false,
  ): Promise<void> {
    if (matchId === null) {
      displayedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // 与 handleHistoryMatchSelect 共用选择发布序号：list 每次发起库存读取
    // （含沿用当前选择）都递增共享 owner——同一选择的旧在途读取（无论来自
    // 用户点击还是更早的 list）随之失效，其迟到成功不得回写 display/cache、
    // 迟到失败不得弹旧 toast 或逃回外层 historyError；本次同步持有
    // selectionLoading，且仅在自己仍是当前 owner 时复位，旧 finally 不得
    // 清新请求的 loading。
    const selectionToken = ++requests.historySelectionRequestToken
    historySelectionLoading.value = true
    // 内部请求跨多个 await：发布前核对 board owner/revision/选择序号，
    // 旧上下文或被取代选择的结果不覆盖新视图。
    const requestRevision = session.revision.value
    const isSelectionStale = () =>
      isStale() ||
      requestRevision !== session.revision.value ||
      selectionToken !== requests.historySelectionRequestToken
    try {
      const grouped = await getHistoryInventorySections(
        matchId,
        isSelectionStale,
      )
      if (isSelectionStale()) {
        return
      }
      displayedHistoryMatchId.value = matchId
      historySections.value = grouped
    } catch (error) {
      // 旧 list/被取代选择的库存失败：不逃回外层 list catch 写 historyError
      // 遮住已成功的新选择；仍是当前上下文的失败向上抛，由 list 错误分支处理。
      if (isSelectionStale()) {
        return
      }
      throw error
    } finally {
      if (selectionToken === requests.historySelectionRequestToken) {
        historySelectionLoading.value = false
      }
    }
  }

  async function handleHistoryMatchSelect(matchId: number): Promise<void> {
    if (selectedHistoryMatchId.value === matchId) {
      return
    }

    selectedHistoryMatchId.value = matchId
    const requestToken = ++requests.historySelectionRequestToken
    const requestRevision = session.revision.value
    const isStale = () =>
      requestToken !== requests.historySelectionRequestToken ||
      requestRevision !== session.revision.value
    historySelectionLoading.value = true

    try {
      const grouped = await getHistoryInventorySections(matchId, isStale)

      if (isStale() || selectedHistoryMatchId.value !== matchId) {
        return
      }

      displayedHistoryMatchId.value = matchId
      historySections.value = grouped
    } catch (error) {
      if (isStale()) {
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
      if (requestToken === requests.historySelectionRequestToken) {
        historySelectionLoading.value = false
      }
    }
  }

  return {
    loadHistoryMatches,
    resetHistoryBoard,
    handleHistoryMatchSelect,
  }
}
