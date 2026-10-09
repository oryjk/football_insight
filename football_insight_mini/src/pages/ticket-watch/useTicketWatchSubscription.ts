import {
  createRefluxSubscriptionOrder,
  getRefluxSubscriptionPlans,
  getRefluxSubscriptionStatus,
} from '../../api/ticketWatch'

import { extractApiErrorMessage } from '../../utils/apiError'

import {
  isRefluxSubscriptionActiveForCurrentMatch,
  isValidNotificationEmail,
  selectPurchasableRefluxSubscriptionPlans,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

import { waitForPaidOrder, requestTicketWatchPayment } from './paymentActions'

export function useTicketWatchSubscription(
  state: TicketWatchState,
  goToUserPage: () => void,
) {
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

  async function refreshRefluxSubscriptionStatus(): Promise<void> {
    const match = currentMatch.value
    if (!match || !currentUser.value || refluxSubscriptionStatusLoading.value) {
      return
    }

    refluxSubscriptionStatusLoading.value = true

    try {
      const status = await getRefluxSubscriptionStatus(
        resolveSelectedTeamCode(),
        inferCurrentMatchSeason(),
        match.match_id,
      )

      refluxSubscriptionEmail.value = status.email_target?.target || ''
      refluxSubscriptionSubscribed.value =
        status.subscribed ||
        isRefluxSubscriptionActiveForCurrentMatch(
          status.active_subscriptions,
          resolveSelectedTeamCode(),
          inferCurrentMatchSeason(),
          match.match_id,
        )
    } catch {
      refluxSubscriptionSubscribed.value = false
      refluxSubscriptionEmail.value = ''
    } finally {
      refluxSubscriptionStatusLoading.value = false
    }
  }

  function resolveSelectedTeamCode(): string {
    return selectedTeam.value
  }

  function inferCurrentMatchSeason(): number {
    const match = currentMatch.value
    const rawYear =
      match?.kickoff_at?.slice(0, 4) || match?.match_date?.slice(0, 4)
    const year = Number.parseInt(rawYear || '', 10)
    return Number.isFinite(year) && year > 0 ? year : 2026
  }

  async function openRefluxSubscriptionSheet(): Promise<void> {
    if (refluxSubscriptionLoading.value) {
      return
    }

    if (!currentMatch.value) {
      uni.showToast({ title: '暂无当前比赛', icon: 'none' })
      return
    }

    if (!currentUser.value) {
      goToUserPage()
      return
    }

    refluxSubscriptionSheetVisible.value = true
    refluxSubscriptionLoading.value = true

    try {
      const [plans, status] = await Promise.all([
        getRefluxSubscriptionPlans(
          resolveSelectedTeamCode(),
          currentMatch.value.match_id,
        ),
        getRefluxSubscriptionStatus(
          resolveSelectedTeamCode(),
          inferCurrentMatchSeason(),
          currentMatch.value.match_id,
        ),
      ])

      const availablePlans = selectPurchasableRefluxSubscriptionPlans(
        plans.plans,
        plans.active_subscriptions,
        resolveSelectedTeamCode(),
        currentMatch.value.match_id,
      )

      refluxSubscriptionPlans.value = availablePlans
      selectedRefluxSubscriptionPlanCode.value = availablePlans[0]?.code ?? ''
      refluxSubscriptionEmail.value =
        status.email_target?.target || plans.email_target?.target || ''
      refluxSubscriptionSubscribed.value =
        status.subscribed ||
        isRefluxSubscriptionActiveForCurrentMatch(
          status.active_subscriptions,
          resolveSelectedTeamCode(),
          inferCurrentMatchSeason(),
          currentMatch.value.match_id,
        )
    } catch (error) {
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅套餐加载失败'),
        icon: 'none',
      })
      refluxSubscriptionSheetVisible.value = false
    } finally {
      refluxSubscriptionLoading.value = false
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

    refluxSubscriptionSubmitting.value = true

    try {
      const order = await createRefluxSubscriptionOrder({
        plan_code: selectedRefluxSubscriptionPlanCode.value,
        team_code: resolveSelectedTeamCode(),
        match_id: match.match_id,
        email,
      })

      await requestTicketWatchPayment(order.params)
      const paid = await waitForPaidOrder(order.order_no)
      if (paid) {
        uni.showToast({ title: '订阅已开通', icon: 'success' })
        refluxSubscriptionSubscribed.value = true
        refluxSubscriptionSheetVisible.value = false
      } else {
        uni.showToast({ title: '支付成功，开通确认中', icon: 'none' })
      }
    } catch (error) {
      uni.showToast({
        title: extractApiErrorMessage(error, '订阅下单失败'),
        icon: 'none',
      })
    } finally {
      refluxSubscriptionSubmitting.value = false
    }
  }

  return {
    refreshRefluxSubscriptionStatus,
    openRefluxSubscriptionSheet,
    closeRefluxSubscriptionSheet,
    submitRefluxSubscriptionOrder,
  }
}
