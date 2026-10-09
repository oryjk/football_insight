import {
  createRefluxSubscriptionOrder,
  getRefluxSubscriptionPlans,
  getRefluxSubscriptionStatus,
} from '../../api/ticketWatch'

import { extractApiErrorMessage } from '../../utils/apiError'
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
  } = actions
  const {
    selectedTeam,
    currentMatch,
    canRequestWxPayment,
    currentUser,
    refluxSubscriptionSheetVisible,
    refluxSubscriptionLoading,
    refluxSubscriptionSubmitting,
    refluxSubscriptionPlans,
    selectedRefluxSubscriptionPlanCode,
    refluxSubscriptionEmail,
    refluxSubscriptionSubscribed,
    refluxSubscriptionStatusLoading,
  } = state

  /** 订阅状态是账号私有数据：只在 revision 未变且比赛/球队快照一致时发布。 */
  function isCurrentSnapshot(
    requestRevision: number,
    matchId: number,
    teamCode: string,
  ): boolean {
    return session.revision.value === requestRevision
      && currentMatch.value?.match_id === matchId
      && selectedTeam.value === teamCode
  }

  async function refreshRefluxSubscriptionStatus(): Promise<void> {
    const match = currentMatch.value
    if (!match || !currentUser.value || refluxSubscriptionStatusLoading.value) {
      return
    }

    const requestRevision = session.revision.value
    const teamCode = selectedTeam.value
    refluxSubscriptionStatusLoading.value = true

    try {
      const status = await getRefluxSubscriptionStatus(
        teamCode,
        inferCurrentMatchSeason(match),
        match.match_id,
      )

      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
        return
      }
      refluxSubscriptionSubscribed.value = false
      refluxSubscriptionEmail.value = ''
    } finally {
      if (isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
    const teamCode = selectedTeam.value

    try {
      const [plans, status] = await Promise.all([
        getRefluxSubscriptionPlans(teamCode, match.match_id),
        getRefluxSubscriptionStatus(
          teamCode,
          inferCurrentMatchSeason(match),
          match.match_id,
        ),
      ])

      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
        return
      }
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅套餐加载失败'),
        icon: 'none',
      })
      refluxSubscriptionSheetVisible.value = false
    } finally {
      if (isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
    const teamCode = selectedTeam.value
    refluxSubscriptionSubmitting.value = true

    try {
      const order = await createRefluxSubscriptionOrder({
        plan_code: selectedRefluxSubscriptionPlanCode.value,
        team_code: teamCode,
        match_id: match.match_id,
        email,
      })

      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
        return
      }
      await requestTicketWatchPayment(order.params)
      const paid = await waitForPaidOrder(order.order_no)
      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
      if (!isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
        return
      }
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅下单失败'),
        icon: 'none',
      })
    } finally {
      if (isCurrentSnapshot(requestRevision, match.match_id, teamCode)) {
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
