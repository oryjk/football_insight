import type { TicketWatchMatchSummary } from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import type { AppSession } from '../../stores/appSession'
import {
  applyBlockInterestsToSections,
  groupInventoryByPrice,
  resolveCurrentBoardLoadStrategy,
  requireInventorySince,
  type TicketWatchLoadReason,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'
import type { TicketWatchBoardApi } from './ticketWatchBoardTypes'
import type { TicketWatchBoardRequests } from './ticketWatchBoardRequests'

export function createTicketWatchCurrentBoard(
  state: TicketWatchState,
  api: TicketWatchBoardApi,
  session: AppSession,
  requests: TicketWatchBoardRequests,
  ensureRegions: () => Promise<void>,
  refreshRefluxSubscriptionStatus: () => Promise<void>,
) {
  const {
    getCurrentTicketWatchBoard,
    getTicketWatchTrackedInterests,
    getYukunCurrentTicketWatchMatch,
    getYukunTicketWatchInventory,
    getYukunTicketWatchRegions,
  } = api
  const {
    selectedTeam,
    currentLoading,
    currentErrorMessage,
    currentMessage,
    regions,
    currentMatch,
    currentSections,
    hasLoadedCurrentBoard,
    currentBoardGeneration,
    currentBoardRequestInFlight,
    currentUser,
    refluxSubscriptionEmail,
    refluxSubscriptionSubscribed,
    currentTrackedInterests,
    resetCurrentMatchScopedState,
  } = state

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
    requests.currentBoardRequestSeq += 1
    requests.activeCurrentBoardRequest = null
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

  async function loadCurrentBoard(
    reason: TicketWatchLoadReason = 'initial',
  ): Promise<void> {
    const requestRevision = session.revision.value
    const requestTeam = selectedTeam.value
    // 同会话同球队的在途请求直接复用（普通 poll 合并）；
    // reset/球队切换已作废旧 owner，新上下文允许立即发起新请求。
    const active = requests.activeCurrentBoardRequest
    if (
      active &&
      active.revision === requestRevision &&
      active.team === requestTeam
    ) {
      return
    }
    const owner = ++requests.currentBoardRequestSeq
    requests.activeCurrentBoardRequest = { owner, revision: requestRevision, team: requestTeam }
    const isOwner = () => requests.activeCurrentBoardRequest?.owner === owner

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
        requests.activeCurrentBoardRequest = null
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
      requests.activeCurrentBoardRequest?.owner !== owner ||
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
      requests.activeCurrentBoardRequest?.owner !== owner ||
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

  return {
    loadCurrentTrackedInterests,
    loadCurrentBoard,
    resetCurrentBoard,
  }
}
