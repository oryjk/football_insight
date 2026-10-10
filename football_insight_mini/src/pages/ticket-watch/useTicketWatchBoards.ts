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
import type { TicketWatchTab, TicketWatchTeam } from './types'

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
    currentBoardGeneration,
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
    membershipBenefitsLocked,
    resetCurrentMatchScopedState,
  } = state
  let historySelectionRequestToken = 0
  // 看板请求 owner：reset/球队切换作废旧请求并允许新上下文立即重载；
  // 所有 success/catch/finally 都核对 owner，旧请求不写数据、不清新 loading。
  // owner 与订阅 generation（currentBoardGeneration）独立：合法 board 请求自己的
  // publishCurrentMatch 会递增订阅 generation，不能以 generation 判定自身新旧。
  interface BoardRequestOwner {
    owner: number
    revision: number
    team: TicketWatchTeam
  }
  let currentBoardRequestSeq = 0
  let activeCurrentBoardRequest: BoardRequestOwner | null = null
  let historyBoardRequestSeq = 0
  let activeHistoryBoardRequest: BoardRequestOwner | null = null
  let historyTrendRequestSeq = 0
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

  /**
   * 当前比赛上下文被替换（含从无到有的首次发布）：递增 generation，
   * 使按旧上下文发出的订阅/套餐/下单/支付请求永久失效，并统一复位
   * 旧比赛的订阅/解锁状态与 loading/sheet/paying 标记（旧请求的 finally
   * 按 generation 跳过清理，标记只能在这里复位）。
   * 同一比赛的普通库存轮询返回相同 match_id，不递增 generation 也不复位。
   */
  function publishCurrentMatch(match: TicketWatchMatchSummary | null): void {
    if ((match?.match_id ?? null) !== (currentMatch.value?.match_id ?? null)) {
      currentBoardGeneration.value += 1
      resetCurrentMatchScopedState()
    }
    currentMatch.value = match
  }

  function resetCurrentBoard(): void {
    // 先递增 generation：reset 前发出的订阅/下单请求在此刻永久失效，
    // 其迟到结果与 finally 不得写数据或清掉新请求的 loading。
    currentBoardGeneration.value += 1
    // 作废在途 board 请求 owner：reset 后新上下文允许立即重载；
    // 旧请求的成功/失败/finally 全部被 owner 检查挡住，不写数据、不清新 loading。
    currentBoardRequestSeq += 1
    activeCurrentBoardRequest = null
    currentBoardRequestInFlight.value = false
    currentLoading.value = false
    currentMatch.value = null
    currentSections.value = []
    currentTrackedInterests.value = []
    // 球队/比赛切换与自然 rollover 走同一个比赛状态 reset：旧订阅/解锁请求的
    // 快照失效、其 finally 跳过清理，loading/sheet/paying/权益标记在这里复位，
    // 否则后续刷新与开 sheet 一直被旧在途标记挡住。
    resetCurrentMatchScopedState()
    currentErrorMessage.value = ''
    hasLoadedCurrentBoard.value = false
    currentMessage.value = '暂无当前比赛。'
  }

  function resetHistoryBoard(): void {
    // 作废在途的历史列表/选择/trend 请求：旧请求的成功/catch/finally
    // 不得覆盖新上下文、不写共享缓存、不清新请求的 loading。
    historyBoardRequestSeq += 1
    activeHistoryBoardRequest = null
    historyBoardRequestInFlight.value = false
    historyLoading.value = false
    historySelectionRequestToken += 1
    historySelectionLoading.value = false
    historyTrendRequestSeq += 1
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

  async function loadCurrentBoard(
    reason: TicketWatchLoadReason = 'initial',
  ): Promise<void> {
    const requestRevision = session.revision.value
    const requestTeam = selectedTeam.value
    // 同会话同球队的在途请求直接复用（普通 poll 合并）；
    // reset/球队切换已作废旧 owner，新上下文允许立即发起新请求。
    const active = activeCurrentBoardRequest
    if (
      active &&
      active.revision === requestRevision &&
      active.team === requestTeam
    ) {
      return
    }
    const owner = ++currentBoardRequestSeq
    activeCurrentBoardRequest = { owner, revision: requestRevision, team: requestTeam }
    const isOwner = () => activeCurrentBoardRequest?.owner === owner

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
      if (requestTeam === 'yunnanyukun') {
        await loadYukunCurrentBoard(owner, requestRevision)
      } else {
        await loadChengduCurrentBoard(owner, requestRevision)
      }
      if (!isOwner() || requestRevision !== session.revision.value) {
        // 账号/上下文在看板请求期间切换：revision watch 已清理状态，旧结果不标记已加载。
        return
      }
      hasLoadedCurrentBoard.value = true
    } catch (error) {
      if (!isOwner() || requestRevision !== session.revision.value) {
        return
      }
      if (reason === 'initial' || !hasLoadedCurrentBoard.value) {
        currentErrorMessage.value = extractApiErrorMessage(
          error,
          '当前比赛回流加载失败，请稍后重试。',
        )
      }
    } finally {
      if (isOwner()) {
        activeCurrentBoardRequest = null
        currentLoading.value = false
        currentBoardRequestInFlight.value = false
      }
    }
  }

  async function loadChengduCurrentBoard(
    owner: number,
    requestRevision: number,
  ): Promise<void> {
    await ensureRegions()
    const response = await getCurrentTicketWatchBoard()
    if (
      activeCurrentBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    ) {
      return
    }
    publishCurrentMatch(response.current_match)
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

  async function loadYukunCurrentBoard(
    owner: number,
    requestRevision: number,
  ): Promise<void> {
    const isStale = () =>
      activeCurrentBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    const response = await getYukunCurrentTicketWatchMatch()
    if (isStale()) {
      return
    }
    const yukunMatch = response.current_match

    if (!yukunMatch) {
      publishCurrentMatch(null)
      currentMessage.value = response.message || '暂无云南玉昆的当前比赛。'
      currentSections.value = []
      currentTrackedInterests.value = []
      return
    }

    // publishCurrentMatch 递增的是订阅 generation，不影响本请求的 owner：
    // 后续库存/区域阶段仍是同一合法请求的延续。
    publishCurrentMatch(yukunMatch)
    currentMessage.value = ''
    void refreshRefluxSubscriptionStatus()
    const since = requireInventorySince(yukunMatch.sale_start_at)

    const [inventory, yukunRegions] = await Promise.all([
      getYukunTicketWatchInventory(yukunMatch.match_id, since),
      getYukunTicketWatchRegions(yukunMatch.match_id, since),
    ])
    if (isStale()) {
      return
    }

    currentSections.value = groupInventoryByPrice(yukunRegions, inventory)
    currentTrackedInterests.value = []
  }

  async function loadHistoryMatches(
    reason: TicketWatchHistoryLoadReason = 'initial',
  ): Promise<void> {
    const requestRevision = session.revision.value
    const requestTeam = selectedTeam.value
    // 同会话同球队的在途请求直接复用；reset/球队切换后的新上下文允许立即重载。
    const active = activeHistoryBoardRequest
    if (
      active &&
      active.revision === requestRevision &&
      active.team === requestTeam
    ) {
      return
    }
    const owner = ++historyBoardRequestSeq
    activeHistoryBoardRequest = { owner, revision: requestRevision, team: requestTeam }
    const isOwner = () => activeHistoryBoardRequest?.owner === owner

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
        activeHistoryBoardRequest = null
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
      activeHistoryBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    await ensureRegions()
    const matches = await getTicketWatchMatches()
    if (isStale()) {
      return
    }
    historyMatches.value = selectCompletedMatches(matches)
    void loadHistoryRefluxTrendTotals(
      selectRefluxStatsMatches(historyMatches.value),
    )

    if (!historyMatches.value.length) {
      selectedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // list 改变选中比赛（为空或已不在列表）时接管选择 owner；
    // 沿用当前选择时只捕获序号，用户在途/随后的选择操作会让本次发布失效。
    const selectionMissing =
      selectedHistoryMatchId.value === null ||
      !historyMatches.value.some(
        (match) => match.match_id === selectedHistoryMatchId.value,
      )
    if (selectionMissing) {
      selectedHistoryMatchId.value = historyMatches.value[0].match_id
    }

    await syncDisplayedHistoryMatch(
      selectedHistoryMatchId.value,
      isStale,
      selectionMissing,
    )
  }

  async function loadYukunHistoryMatches(
    owner: number,
    requestRevision: number,
  ): Promise<void> {
    const isStale = () =>
      activeHistoryBoardRequest?.owner !== owner ||
      requestRevision !== session.revision.value
    const matches = await getYukunTicketWatchMatches()
    if (isStale()) {
      return
    }
    historyMatches.value = selectCompletedMatches(matches)

    void loadHistoryRefluxTrendTotals(
      selectRefluxStatsMatches(historyMatches.value),
    )

    if (!historyMatches.value.length) {
      selectedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // list 改变选中比赛（为空或已不在列表）时接管选择 owner；
    // 沿用当前选择时只捕获序号，用户在途/随后的选择操作会让本次发布失效。
    const selectionMissing =
      selectedHistoryMatchId.value === null ||
      !historyMatches.value.some(
        (match) => match.match_id === selectedHistoryMatchId.value,
      )
    if (selectionMissing) {
      selectedHistoryMatchId.value = historyMatches.value[0].match_id
    }

    await syncDisplayedHistoryMatch(
      selectedHistoryMatchId.value,
      isStale,
      selectionMissing,
    )
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
    const owner = ++historyTrendRequestSeq
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
              owner !== historyTrendRequestSeq ||
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
      if (owner === historyTrendRequestSeq) {
        historyTrendLoading.value = false
      }
    }
  }

  async function syncDisplayedHistoryMatch(
    matchId: number | null,
    isStale: () => boolean = () => false,
    takeOverSelection = false,
  ): Promise<void> {
    if (matchId === null) {
      displayedHistoryMatchId.value = null
      historySections.value = []
      return
    }

    // 与 handleHistoryMatchSelect 共用选择发布序号：list 接管（改变选中比赛）时
    // 递增序号使在途用户选择失效，并由本次同步持有/复位 selectionLoading；
    // 沿用时仅捕获序号，用户在途或随后的选择操作都会让本次发布失效。
    const selectionToken = takeOverSelection
      ? ++historySelectionRequestToken
      : historySelectionRequestToken
    if (takeOverSelection) {
      historySelectionLoading.value = true
    }
    // 内部请求跨多个 await：发布前核对 board owner/revision/选择序号，
    // 旧上下文或被取代选择的结果不覆盖新视图。
    const requestRevision = session.revision.value
    const isSelectionStale = () =>
      isStale() ||
      requestRevision !== session.revision.value ||
      selectionToken !== historySelectionRequestToken
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
      if (
        takeOverSelection &&
        selectionToken === historySelectionRequestToken
      ) {
        historySelectionLoading.value = false
      }
    }
  }

  async function handleHistoryMatchSelect(matchId: number): Promise<void> {
    if (selectedHistoryMatchId.value === matchId) {
      return
    }

    selectedHistoryMatchId.value = matchId
    const requestToken = ++historySelectionRequestToken
    const requestRevision = session.revision.value
    const isStale = () =>
      requestToken !== historySelectionRequestToken ||
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
      if (requestToken === historySelectionRequestToken) {
        historySelectionLoading.value = false
      }
    }
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
