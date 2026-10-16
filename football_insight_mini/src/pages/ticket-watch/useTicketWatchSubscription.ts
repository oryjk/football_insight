import { startPaymentAttempt } from '../../composables/paymentTelemetry'
import { prepareVirtualPayment } from '../../composables/prepareVirtualPayment'
import {
  createRefluxSubscriptionOrder,
  getRefluxSubscriptionPlans,
  getRefluxSubscriptionStatus,
} from '../../api/ticketWatch'

import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { useAppSession, type AppSession } from '../../stores/appSession'
import type { TicketWatchMatchSummary } from '../../types/ticketWatch'

import {
  isRefluxSubscriptionActiveForCurrentMatch,
  isValidNotificationEmail,
  selectPurchasableRefluxSubscriptionPlans,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

import { waitForPaidOrder, requestTicketWatchPayment } from './paymentActions'

const subscriptionActions = {
  prepareVirtualPayment,
  createRefluxSubscriptionOrder,
  getRefluxSubscriptionPlans,
  getRefluxSubscriptionStatus,
  waitForPaidOrder,
  requestTicketWatchPayment,
}

export function useTicketWatchSubscription(
  state: TicketWatchState,
  goToUserPage: () => void,
  actions = subscriptionActions,
  session: AppSession = useAppSession(),
) {
  const {
    createRefluxSubscriptionOrder,
    getRefluxSubscriptionPlans,
    getRefluxSubscriptionStatus,
    waitForPaidOrder,
    requestTicketWatchPayment,
    prepareVirtualPayment,
  } = actions
  const {
    selectedTeam,
    currentMatch,
    canRequestWxPayment,
    currentUser,
    currentBoardGeneration,
    refluxSubscriptionSheetVisible,
    refluxSubscriptionLoading,
    refluxSubscriptionSubmitting,
    refluxSubscriptionPlans,
    selectedRefluxSubscriptionPlanCode,
    refluxSubscriptionEmail,
    refluxSubscriptionSubscribed,
    refluxSubscriptionStatusLoading,
  } = state

  // 同类操作在同一 generation 内允许先后重叠（reset 已清 loading，旧请求仍在途）：
  // 只有当前 owner 能清自己的 loading，旧请求的 finally 不得清理新请求。
  let statusRequestOwner = 0
  let sheetRequestOwner = 0
  let submitRequestOwner = 0

  /**
   * 订阅状态是账号私有数据：只在 revision、board generation、比赛/球队快照一致时发布。
   * generation 覆盖 A→B→A：仅比较球队/比赛 ID 无法识别「切走再切回」后的旧请求。
   */
  function isCurrentSnapshot(
    requestRevision: number,
    requestGeneration: number,
    matchId: number,
    teamCode: string,
  ): boolean {
    return session.revision.value === requestRevision
      && currentBoardGeneration.value === requestGeneration
      && currentMatch.value?.match_id === matchId
      && selectedTeam.value === teamCode
  }

  async function refreshRefluxSubscriptionStatus(): Promise<void> {
    const match = currentMatch.value
    if (!match || !currentUser.value || refluxSubscriptionStatusLoading.value) {
      return
    }

    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    const teamCode = selectedTeam.value
    const owner = ++statusRequestOwner
    refluxSubscriptionStatusLoading.value = true

    try {
      const status = await getRefluxSubscriptionStatus(
        teamCode,
        inferCurrentMatchSeason(match),
        match.match_id,
      )

      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      refluxSubscriptionEmail.value = status.email_target?.target || ''
      refluxSubscriptionSubscribed.value =
        status.subscribed ||
        isRefluxSubscriptionActiveForCurrentMatch(
          status.active_subscriptions,
          teamCode,
          inferCurrentMatchSeason(match),
          match.match_id,
        )
    } catch {
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      refluxSubscriptionSubscribed.value = false
      refluxSubscriptionEmail.value = ''
    } finally {
      if (
        owner === statusRequestOwner
        && isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)
      ) {
        refluxSubscriptionStatusLoading.value = false
      }
    }
  }

  function inferCurrentMatchSeason(
    match: TicketWatchMatchSummary | null = currentMatch.value,
  ): number {
    const rawYear =
      match?.kickoff_at?.slice(0, 4) || match?.match_date?.slice(0, 4)
    const year = Number.parseInt(rawYear || '', 10)
    return Number.isFinite(year) && year > 0 ? year : 2026
  }

  async function openRefluxSubscriptionSheet(): Promise<void> {
    if (refluxSubscriptionLoading.value) {
      return
    }

    const match = currentMatch.value
    if (!match) {
      uni.showToast({ title: '暂无当前比赛', icon: 'none' })
      return
    }

    if (!currentUser.value) {
      goToUserPage()
      return
    }

    refluxSubscriptionSheetVisible.value = true
    refluxSubscriptionLoading.value = true
    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    const teamCode = selectedTeam.value
    const owner = ++sheetRequestOwner

    try {
      const [plans, status] = await Promise.all([
        getRefluxSubscriptionPlans(teamCode, match.match_id),
        getRefluxSubscriptionStatus(
          teamCode,
          inferCurrentMatchSeason(match),
          match.match_id,
        ),
      ])

      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      const availablePlans = selectPurchasableRefluxSubscriptionPlans(
        plans.plans,
        plans.active_subscriptions,
        teamCode,
        match.match_id,
      )

      refluxSubscriptionPlans.value = availablePlans
      selectedRefluxSubscriptionPlanCode.value = availablePlans[0]?.code ?? ''
      refluxSubscriptionEmail.value =
        status.email_target?.target || plans.email_target?.target || ''
      refluxSubscriptionSubscribed.value =
        status.subscribed ||
        isRefluxSubscriptionActiveForCurrentMatch(
          status.active_subscriptions,
          teamCode,
          inferCurrentMatchSeason(match),
          match.match_id,
        )
    } catch (error) {
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅套餐加载失败'),
        icon: 'none',
      })
      refluxSubscriptionSheetVisible.value = false
    } finally {
      if (
        owner === sheetRequestOwner
        && isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)
      ) {
        refluxSubscriptionLoading.value = false
      }
    }
  }

  function closeRefluxSubscriptionSheet(): void {
    if (refluxSubscriptionSubmitting.value) {
      return
    }

    refluxSubscriptionSheetVisible.value = false
  }

  async function submitRefluxSubscriptionOrder(): Promise<void> {
    const match = currentMatch.value
    if (!match || refluxSubscriptionSubmitting.value) {
      return
    }

    if (!canRequestWxPayment.value) {
      uni.showToast({ title: '请在微信小程序内完成支付', icon: 'none' })
      return
    }

    const email = refluxSubscriptionEmail.value.trim()
    if (!isValidNotificationEmail(email)) {
      uni.showToast({ title: '请填写有效邮箱', icon: 'none' })
      return
    }

    if (!selectedRefluxSubscriptionPlanCode.value) {
      uni.showToast({ title: '请选择订阅套餐', icon: 'none' })
      return
    }

    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    const teamCode = selectedTeam.value
    const owner = ++submitRequestOwner
    const attempt = startPaymentAttempt('reflux_subscription', selectedRefluxSubscriptionPlanCode.value)
    attempt.report('purchase_tapped')
    let telemetryStage: 'session' | 'order' | 'payment' = 'session'
    refluxSubscriptionSubmitting.value = true

    try {
      await prepareVirtualPayment()
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) return
      telemetryStage = 'order'
      attempt.report('order_create_requested')
      const order = await createRefluxSubscriptionOrder({
        plan_code: selectedRefluxSubscriptionPlanCode.value,
        team_code: teamCode,
        match_id: match.match_id,
        email,
      }, attempt.attemptId)
      attempt.bindOrder(order.order_no)
      telemetryStage = 'payment'

      // 上下文（含 A→B→A 后的同名比赛）已失效的旧订单不得拉起微信支付。
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      await requestTicketWatchPayment(order.params, attempt)
      // 原生支付等待期间上下文可能已失效（reset/换比赛/切账号）：
      // 已执行的真实支付不取消，但失效页面不再发起确认读与后续结果发布。
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      const paid = await waitForPaidOrder(order.order_no, () => isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode), attempt)
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      if (paid) {
        uni.showToast({ title: '订阅已开通', icon: 'success' })
        refluxSubscriptionSubscribed.value = true
        refluxSubscriptionSheetVisible.value = false
      } else {
        uni.showToast({ title: '支付成功，开通确认中', icon: 'none' })
      }
    } catch (error) {
      if (telemetryStage === 'session') attempt.report('session_prepare_failed', { error_msg: extractApiErrorMessage(error, '订阅下单失败') })
      if (telemetryStage === 'order') attempt.report('order_create_failed', { error_msg: extractApiErrorMessage(error, '订阅下单失败'), detail: error instanceof ApiRequestError ? { http_status: error.statusCode } : undefined })
      if (!isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)) {
        return
      }
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅下单失败'),
        icon: 'none',
      })
    } finally {
      if (
        owner === submitRequestOwner
        && isCurrentSnapshot(requestRevision, requestGeneration, match.match_id, teamCode)
      ) {
        refluxSubscriptionSubmitting.value = false
      }
    }
  }

  return {
    refreshRefluxSubscriptionStatus,
    openRefluxSubscriptionSheet,
    closeRefluxSubscriptionSheet,
    submitRefluxSubscriptionOrder,
  }
}
