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

  // 解锁 sheet 请求 owner：open/refresh 各自递增，close/reset 使之失效；
  // 同一 generation 内关闭再打开时，旧权益请求的迟到成功/失败不得覆盖、
  // 关闭新 sheet 或弹旧错误。
  let matchIdSheetRequestOwner = 0

  function closeMatchIdSheet(): void {
    if (matchIdSheetState.value === 'paying') {
      return
    }

    // 关闭使在途的权益查询失效：随后重开 sheet 时，
    // 旧请求的迟到结果不得重开/覆盖/关闭新 sheet。
    matchIdSheetRequestOwner += 1
    matchIdSheetVisible.value = false
  }

  /**
   * 会话切换/退出时重置解锁操作状态与私有权益：旧下单/支付的 finally
   * 因 revision 防护跳过清理，paying 与 entitlement 只能在这里复位，
   * 否则新账号的 openMatchIdSheet 一直被 paying 挡住。
   */
  function resetMatchIdSessionState(): void {
    // 在途的权益/下单/支付流程同时按 owner 失效，迟到结果不得复活旧状态。
    matchIdSheetRequestOwner += 1
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
    // 与 openMatchIdSheet 共用 sheet owner：并发的旧查询不得覆盖后来发布的权益。
    const requestRevision = session.revision.value
    const requestGeneration = currentBoardGeneration.value
    const owner = ++matchIdSheetRequestOwner
    const isStale = () =>
      owner !== matchIdSheetRequestOwner
      || !isCurrentMatchContext(requestRevision, requestGeneration)
    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      if (isStale()) {
        return
      }
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch {
      if (isStale()) {
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
    // 打开持有新的 sheet owner：此前在途的权益查询（含被关闭的旧 sheet）
    // 全部失效，其迟到的成功/失败不得覆盖、关闭新 sheet 或弹旧错误。
    const owner = ++matchIdSheetRequestOwner
    const isStale = () =>
      owner !== matchIdSheetRequestOwner
      || !isCurrentMatchContext(requestRevision, requestGeneration)
    try {
      const entitlement = await getMatchIdEntitlement(match.match_id)
      if (isStale()) {
        return
      }
      matchIdEntitlement.value = entitlement
      matchIdSheetState.value = resolveMatchIdSheetState(entitlement)
    } catch (error) {
      if (isStale()) {
        // 请求期间会话已切换/退出、看板上下文被替换或 sheet 已关闭重开：
        // 旧结果静默放弃。
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
    // 下单/支付同属 sheet 上下文：close/reset 会递增 owner，
    // 失效后的旧流程不再发起确认读、不再改 sheet 状态。
    const owner = matchIdSheetRequestOwner
    const isStale = () =>
      owner !== matchIdSheetRequestOwner
      || !isCurrentMatchContext(requestRevision, requestGeneration)

    try {
      const order = await createMatchIdOrder(match.match_id)

      // 上下文（含 A→B→A 后的同名比赛）已失效的旧订单不得拉起微信支付。
      if (isStale()) {
        return
      }
      await requestTicketWatchPayment(order.wx_pay_params)
      // 原生支付等待期间上下文可能已失效：已执行的真实支付不取消，
      // 但失效页面不再发起确认读与后续结果发布。
      if (isStale()) {
        return
      }
      const paid = await waitForPaidOrder(order.order_no)
      if (isStale()) {
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
      if (isStale()) {
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
        !isStale()
        && matchIdSheetState.value === 'paying'
      ) {
        matchIdSheetState.value = 'locked'
      }
    }
  }

  return { closeMatchIdSheet, openMatchIdSheet, payForMatchId, resetMatchIdSessionState }
}
