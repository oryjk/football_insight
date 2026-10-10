import {
  createMatchIdOrder,
  getMatchIdEntitlement,
} from '../../api/matchIdUnlock'

import {
  ApiRequestError,
  extractApiErrorMessage,
  isUnauthorizedError,
} from '../../utils/apiError'
import { useAppSession, type AppSession } from '../../stores/appSession'

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
  session: AppSession = useAppSession(),
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
    currentBoardGeneration,
  } = state

  /**
   * 解锁权益/下单/支付是账号私有操作：revision 覆盖账号切换，
   * board generation 覆盖 A→B→A（切走再切回同一比赛）的上下文替换。
   */
  function isCurrentMatchContext(requestRevision: number, requestGeneration: number): boolean {
    return session.revision.value === requestRevision
      && currentBoardGeneration.value === requestGeneration
  }

  function closeMatchIdSheet(): void {
    if (matchIdSheetState.value === 'paying') {
      return
    }

    matchIdSheetVisible.value = false
  }

  /**
   * 会话切换/退出时重置解锁操作状态与私有权益：旧下单/支付的 finally
   * 因 revision 防护跳过清理，paying 与 entitlement 只能在这里复位，
   * 否则新账号的 openMatchIdSheet 一直被 paying 挡住。
   */
  function resetMatchIdSessionState(): void {
    matchIdSheetVisible.value = false
    matchIdSheetState.value = 'locked'
    matchIdEntitlement.value = null
  }

  async function refreshMatchIdEntitlement(): Promise<void> {
    const match = currentMatch.value
    if (!match) {
      return
    }

    // 解锁权益是账号私有数据：旧账号/旧看板上下文的结果晚到时不发布。
    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch {
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
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

    if (!session.hasAccessToken.value) {
      promptMatchIdLogin()
      return
    }

    matchIdSheetVisible.value = true
    matchIdSheetState.value = 'loading'
    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch (error) {
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        // 请求期间会话已切换/退出或看板上下文被替换：旧结果静默放弃。
        return
      }
      matchIdSheetVisible.value = false
      if (isUnauthorizedError(error)) {
        // token 失效与未登录走同一个登录引导，避免弹窗闪开即关、体感"没反应"。
        // 仅失效当前 revision 的会话，不影响请求期间登录的新账号。
        session.invalidateSession(requestRevision)
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
    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value

    try {
      const order = await createMatchIdOrder(match.match_id)

      // 上下文（含 A→B→A 后的同名比赛）已失效的旧订单不得拉起微信支付。
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
      await requestTicketWatchPayment(order.wx_pay_params)
      const paid = await waitForPaidOrder(order.order_no)
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
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
      if (!isCurrentMatchContext(requestRevision, requestGeneration)) {
        return
      }
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
      if (
        isCurrentMatchContext(requestRevision, requestGeneration)
        && matchIdSheetState.value === 'paying'
      ) {
        matchIdSheetState.value = 'locked'
      }
    }
  }

  return { closeMatchIdSheet, openMatchIdSheet, payForMatchId, resetMatchIdSessionState }
}
