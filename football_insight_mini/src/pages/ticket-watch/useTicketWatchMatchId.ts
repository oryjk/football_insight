import {
  createMatchIdOrder,
  getMatchIdEntitlement,
} from '../../api/matchIdUnlock'

import {
  ApiRequestError,
  extractApiErrorMessage,
  isUnauthorizedError,
} from '../../utils/apiError'
import { getAccessToken } from '../../utils/authStorage'

import { resolveMatchIdSheetState } from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

import { waitForPaidOrder, requestTicketWatchPayment } from './paymentActions'

const matchIdActions = {
  createMatchIdOrder,
  getMatchIdEntitlement,
  waitForPaidOrder,
  requestTicketWatchPayment,
}

export function useTicketWatchMatchId(
  state: TicketWatchState,
  goToUserPage: () => void,
  actions = matchIdActions,
) {
  const {
    createMatchIdOrder,
    getMatchIdEntitlement,
    waitForPaidOrder,
    requestTicketWatchPayment,
  } = actions
  const {
    currentMatch,
    currentUser,
    systemConfigUnderReview,
    matchIdSheetVisible,
    matchIdSheetState,
    matchIdEntitlement,
    canRequestWxPayment,
  } = state

  function closeMatchIdSheet(): void {
    if (matchIdSheetState.value === 'paying') {
      return
    }

    matchIdSheetVisible.value = false
  }

  async function refreshMatchIdEntitlement(): Promise<void> {
    const match = currentMatch.value
    if (!match) {
      return
    }

    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch {
      matchIdSheetState.value = 'locked'
    }
  }

  function promptMatchIdLogin(): void {
    uni.showModal({
      title: '先登录再查看',
      content: '登录后才能查看比赛 ID，现在去“我的”页登录吗？',
      confirmText: '去登录',
      success: ({ confirm }) => {
        if (!confirm) {
          return
        }

        uni.switchTab({
          url: '/pages/user/index',
        })
      },
    })
  }

  async function openMatchIdSheet(): Promise<void> {
    const match = currentMatch.value
    if (!match || matchIdSheetState.value === 'paying') {
      return
    }

    if (systemConfigUnderReview.value) {
      uni.showToast({ title: '该功能审核期间暂不可用', icon: 'none' })
      return
    }

    if (!getAccessToken()) {
      promptMatchIdLogin()
      return
    }

    matchIdSheetVisible.value = true
    matchIdSheetState.value = 'loading'
    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch (error) {
      matchIdSheetVisible.value = false
      if (isUnauthorizedError(error)) {
        // token 失效与未登录走同一个登录引导，避免弹窗闪开即关、体感"没反应"。
        promptMatchIdLogin()
        return
      }
      uni.showToast({
        title: extractApiErrorMessage(error, '获取解锁状态失败'),
        icon: 'none',
      })
    }
  }

  async function payForMatchId(): Promise<void> {
    const match = currentMatch.value
    if (!match || matchIdSheetState.value === 'paying') {
      return
    }

    if (!currentUser.value?.has_wechat_binding) {
      goToUserPage()
      return
    }

    if (!canRequestWxPayment.value) {
      uni.showToast({ title: '请在微信小程序内完成支付', icon: 'none' })
      return
    }

    matchIdSheetState.value = 'paying'

    try {
      const order = await createMatchIdOrder(match.match_id)

      await requestTicketWatchPayment(order.wx_pay_params)
      const paid = await waitForPaidOrder(order.order_no)
      if (paid) {
        uni.showToast({ title: '已解锁', icon: 'success' })
        await refreshMatchIdEntitlement()
      } else {
        matchIdSheetState.value = 'locked'
        uni.showToast({
          title: '支付成功，解锁确认中，请稍后再试',
          icon: 'none',
        })
      }
    } catch (error) {
      if (error instanceof ApiRequestError && error.statusCode === 409) {
        await refreshMatchIdEntitlement()
        return
      }

      uni.showToast({
        title: extractApiErrorMessage(error, '下单失败'),
        icon: 'none',
      })
      matchIdSheetState.value = 'locked'
    } finally {
      if (matchIdSheetState.value === 'paying') {
        matchIdSheetState.value = 'locked'
      }
    }
  }

  return { closeMatchIdSheet, openMatchIdSheet, payForMatchId }
}
