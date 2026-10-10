import { afterEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick, ref, watch, type Ref } from 'vue'
import * as ticketApi from '../../api/ticketWatch'
import type { CurrentUser } from '../../types/auth'
import type {
  RefluxSubscriptionPlansResponse,
  RefluxSubscriptionStatusResponse,
  TicketWatchCurrentBoardResponse,
  TicketWatchCurrentMatchResponse,
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
  TicketWatchTrackedInterest,
} from '../../types/ticketWatch'
import { ApiRequestError } from '../../utils/apiError'
import type { AppSession, SessionStatus } from '../../stores/appSession'
import { useTicketWatchBoards } from './useTicketWatchBoards'
import { useTicketWatchMatchId } from './useTicketWatchMatchId'
import { useTicketWatchPolling } from './useTicketWatchPolling'
import { useTicketWatchState } from './useTicketWatchState'
import { useTicketWatchSubscription } from './useTicketWatchSubscription'
import { createTicketWatchSessionSync } from './sessionSync'

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((accept, decline) => {
    resolve = accept
    reject = decline
  })
  return { promise, resolve, reject }
}

function match(id: number): TicketWatchMatchSummary {
  return {
    match_id: id,
    external_match_id: String(id),
    round_number: id,
    sale_start_at: '2026-09-01T10:00:00+08:00',
    match_date: '2026-09-01',
    match_time: '19:30',
    kickoff_at: '2026-09-01T19:30:00+08:00',
    home_team_name: '成都蓉城',
    away_team_name: '客队',
    is_current: true,
  }
}

const source = { ...ticketApi }
const testGlobal = globalThis as typeof globalThis & { uni: typeof uni }
const originalUni = testGlobal.uni
const originalSetInterval = globalThis.setInterval
const originalClearInterval = globalThis.clearInterval

/** 页面 composable 的会话注入 stub：状态用可写 ref，action 默认不应被调用。 */
type SessionStub = AppSession & { revision: Ref<number> }

function createSessionStub(overrides: Partial<AppSession> = {}): SessionStub {
  return {
    currentUser: ref<CurrentUser | null>(null),
    hasAccessToken: ref(false),
    status: ref<SessionStatus>('guest'),
    errorMessage: ref(''),
    revision: ref(0),
    ensureSessionReady: async () => null,
    loginWithPassword: async () => {
      throw new Error('unused')
    },
    registerAccount: async () => {
      throw new Error('unused')
    },
    loginAsTestUser: async () => {
      throw new Error('unused')
    },
    loginWithMiniWechat: async () => {
      throw new Error('unused')
    },
    bootstrapSession: async () => null,
    bindWechatAccount: async () => {
      throw new Error('unused')
    },
    logoutSession: async () => {},
    invalidateSession: () => {},
    ...overrides,
  }
}

afterEach(() => {
  testGlobal.uni = originalUni
  globalThis.setInterval = originalSetInterval
  globalThis.clearInterval = originalClearInterval
})

describe('ticket watch page effects', () => {
  test('keeps the latest historical selection when requests finish out of order', async () => {
    const state = useTicketWatchState()
    const first = deferred<TicketWatchInventoryEntry[]>()
    const second = deferred<TicketWatchInventoryEntry[]>()
    state.historyMatches.value = [match(1), match(2)]
    state.regions.value = [
      { block_name: 'A1', price: '180', usable_count: 10, estate: 0 },
    ]
    const observedSince: string[] = []
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getTicketWatchInventorySince: async (id, since) => {
        observedSince.push(since || '')
        return id === 1 ? first.promise : second.promise
      },
      getTicketWatchBlockInterests: async () => [],
    }, createSessionStub())
    const oldSelection = boards.handleHistoryMatchSelect(1)
    const newSelection = boards.handleHistoryMatchSelect(2)
    second.resolve([
      {
        block_name: 'A1',
        occurrences: 9,
        latest_time: '2026-09-01T10:15:00+08:00',
      },
    ])
    await newSelection
    first.resolve([
      {
        block_name: 'A1',
        occurrences: 1,
        latest_time: '2026-09-01T10:12:00+08:00',
      },
    ])
    await oldSelection
    expect(state.displayedHistoryMatchId.value).toBe(2)
    expect(state.historySections.value[0].total_occurrences).toBe(9)
    expect(state.historySelectionLoading.value).toBe(false)
    expect(observedSince).toEqual([
      '2026-09-01T10:10:00+08:00',
      '2026-09-01T10:10:00+08:00',
    ])
  })

  test('rejects missing sale start before requesting yukun inventory', async () => {
    const state = useTicketWatchState()
    state.selectedTeam.value = 'yunnanyukun'
    let inventoryCalls = 0
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getYukunCurrentTicketWatchMatch: async () => ({
        current_match: { ...match(1), sale_start_at: null },
        group_ticket_active: false,
        message: '',
      }),
      getYukunTicketWatchInventory: async () => {
        inventoryCalls += 1
        return []
      },
    }, createSessionStub())
    await boards.loadCurrentBoard()
    expect(inventoryCalls).toBe(0)
    expect(state.hasLoadedCurrentBoard.value).toBe(false)
    expect(state.currentErrorMessage.value.includes('sale_start_at')).toBe(true)
  })

  test('keeps successful board data mounted when a silent poll fails', async () => {
    const state = useTicketWatchState()
    state.currentMatch.value = match(1)
    state.hasLoadedCurrentBoard.value = true
    state.currentLoading.value = false
    state.regions.value = [
      { block_name: 'A1', price: '180', usable_count: 10, estate: 0 },
    ]
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getCurrentTicketWatchBoard: async () => {
        throw new Error('offline')
      },
    }, createSessionStub())
    await boards.loadCurrentBoard('poll')
    expect(state.currentMatch.value?.match_id).toBe(1)
    expect(state.currentLoading.value).toBe(false)
    expect(state.currentErrorMessage.value).toBe('')
  })

  test('holds the payment guard through the native dialog and order confirmation', async () => {
    testGlobal.uni = { showToast: () => {} } as unknown as typeof uni
    const state = useTicketWatchState()
    state.currentMatch.value = match(1)
    state.currentUser.value = { has_wechat_binding: true } as CurrentUser
    state.canRequestWxPayment.value = true
    state.matchIdSheetVisible.value = true
    const payment = deferred<void>()
    const confirmation = deferred<boolean>()
    let orders = 0
    const actions = useTicketWatchMatchId(state, () => {}, {
      createMatchIdOrder: async () => {
        orders += 1
        return {
          order_no: 'order',
          wx_pay_params: {
            timeStamp: '',
            nonceStr: '',
            package: '',
            signType: 'RSA',
            paySign: '',
          },
        }
      },
      getMatchIdEntitlement: async () => ({
        unlocked: true,
        via: 'purchase',
        effective_tier: 'V1',
      }),
      requestTicketWatchPayment: () => payment.promise,
      waitForPaidOrder: () => confirmation.promise,
    }, createSessionStub())
    const pending = actions.payForMatchId()
    await Promise.resolve()
    await actions.payForMatchId()
    actions.closeMatchIdSheet()
    expect(orders).toBe(1)
    expect(state.matchIdSheetState.value).toBe('paying')
    expect(state.matchIdSheetVisible.value).toBe(true)
    payment.resolve()
    await Promise.resolve()
    await actions.payForMatchId()
    expect(orders).toBe(1)
    expect(state.matchIdSheetState.value).toBe('paying')
    confirmation.resolve(true)
    await pending
    expect(state.matchIdSheetState.value).toBe('unlocked')
  })

  test('stops polling and freshness clocks when tabs change or the page scope is disposed', async () => {
    const scheduled = new Map<number, number>()
    let nextId = 0
    globalThis.setInterval = ((_callback: unknown, delay: number) => {
      const id = ++nextId
      scheduled.set(id, delay)
      return id
    }) as typeof setInterval
    globalThis.clearInterval = ((id: number) => {
      scheduled.delete(id)
    }) as typeof clearInterval
    const scope = effectScope()
    const state = useTicketWatchState()
    const polling = scope.run(() =>
      useTicketWatchPolling(state, async () => {}, createSessionStub()),
    )!
    state.isMonitoringActive.value = true
    polling.startPolling()
    polling.startFreshnessClock()
    expect([...scheduled.values()]).toEqual([600000, 15000])
    state.activeTab.value = 'history'
    await nextTick()
    expect([...scheduled.values()]).toEqual([15000])
    state.activeTab.value = 'current'
    await nextTick()
    expect(scheduled.size).toBe(2)
    scope.stop()
    expect(scheduled.size).toBe(0)
  })
  test('does not restart polling if the page hides while its initial request is pending', async () => {
    const scheduled = new Set<number>()
    let nextId = 0
    globalThis.setInterval = ((_callback: unknown, _delay: number) => {
      const id = ++nextId
      scheduled.add(id)
      return id
    }) as typeof setInterval
    globalThis.clearInterval = ((id: number) => {
      scheduled.delete(id)
    }) as typeof clearInterval
    const request = deferred<void>()
    const scope = effectScope()
    const state = useTicketWatchState()
    const polling = scope.run(() =>
      useTicketWatchPolling(state, () => request.promise, createSessionStub()),
    )!
    const started = polling.toggleMonitoring()
    polling.stopPolling()
    polling.stopFreshnessClock()
    request.resolve()
    await started
    expect(scheduled.size).toBe(0)
    scope.stop()
  })
})

describe('ticket watch account revision effects', () => {
  test('用户 A 的兴趣结果晚到时不污染切换账号后的看板', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const interestsA = deferred<TicketWatchTrackedInterest[]>()
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getTicketWatchTrackedInterests: () => interestsA.promise,
    }, session)

    const pending = boards.loadCurrentTrackedInterests(1)
    // A 的兴趣请求在途时切换到 B（revision 变化）。
    session.revision.value += 1
    interestsA.resolve([{ block_name: 'A1', started_at: '2026-09-01T10:00:00+08:00' }])
    await pending

    expect(state.currentTrackedInterests.value).toEqual([])
  })

  test('断网时会员档位镜像沿用 session 已确认用户，不误清会话', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub({
      currentUser: ref({ id: 'user-a', membership_tier: 'V6' } as CurrentUser),
      hasAccessToken: ref(true),
      status: ref<SessionStatus>('authenticated'),
      ensureSessionReady: async () => {
        throw new ApiRequestError('网络连接不可用，请检查网络后重试', 0, true)
      },
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    await boards.loadViewerMembershipTier()

    expect(state.currentUser.value?.id).toBe('user-a')
    expect(state.viewerMembershipTier.value).toBe('V6')
    expect(session.currentUser.value?.id).toBe('user-a')
    expect(session.hasAccessToken.value).toBe(true)
  })

  test('会话确认游客后会员档位镜像清空', async () => {
    const state = useTicketWatchState()
    state.currentUser.value = { id: 'user-a', membership_tier: 'V6' } as CurrentUser
    const session = createSessionStub()
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    await boards.loadViewerMembershipTier()

    expect(state.currentUser.value).toBeNull()
    expect(state.viewerMembershipTier.value).toBe('V1')
  })

  test('退出登录后旧的监控请求完成也不会重启轮询', async () => {
    const scheduled = new Set<number>()
    let nextId = 0
    globalThis.setInterval = ((_callback: unknown, _delay: number) => {
      const id = ++nextId
      scheduled.add(id)
      return id
    }) as typeof setInterval
    globalThis.clearInterval = ((id: number) => {
      scheduled.delete(id)
    }) as typeof clearInterval
    const request = deferred<void>()
    const session = createSessionStub()
    const scope = effectScope()
    const state = useTicketWatchState()
    const polling = scope.run(() =>
      useTicketWatchPolling(state, () => request.promise, session),
    )!

    const started = polling.toggleMonitoring()
    // 监控的首次请求在途时退出登录（revision 变化）。
    session.revision.value += 1
    await nextTick()
    request.resolve()
    await started

    expect(state.isMonitoringActive.value).toBe(false)
    expect(scheduled.size).toBe(0)
    scope.stop()
  })
})

describe('ticket watch subscription revision effects', () => {
  function unusedAction() {
    return async () => {
      throw new Error('unused')
    }
  }

  test('订阅状态刷新在会话被替代后 resolve，不恢复邮箱与订阅标记', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const statusRequest = deferred<RefluxSubscriptionStatusResponse>()
    const subscription = useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: unusedAction(),
      getRefluxSubscriptionPlans: unusedAction(),
      getRefluxSubscriptionStatus: () => statusRequest.promise,
      waitForPaidOrder: unusedAction(),
      requestTicketWatchPayment: unusedAction(),
    }, session)

    const pending = subscription.refreshRefluxSubscriptionStatus()
    // A 的刷新在途时切换到 B（revision 变化）。
    session.revision.value += 1
    statusRequest.resolve({
      subscribed: true,
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'a@example.com' },
    })
    await pending

    expect(state.refluxSubscriptionEmail.value).toBe('')
    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    // finally 同样被 guard：loading 由页面 revision watch 统一复位。
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)
  })

  test('订阅套餐在会话被替代后到达，不发布 plans/email，不关弹层不 toast', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const plansRequest = deferred<RefluxSubscriptionPlansResponse>()
    const statusRequest = deferred<RefluxSubscriptionStatusResponse>()
    const subscription = useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: unusedAction(),
      getRefluxSubscriptionPlans: () => plansRequest.promise,
      getRefluxSubscriptionStatus: () => statusRequest.promise,
      waitForPaidOrder: unusedAction(),
      requestTicketWatchPayment: unusedAction(),
    }, session)

    const pending = subscription.openRefluxSubscriptionSheet()
    await Promise.resolve()
    expect(state.refluxSubscriptionSheetVisible.value).toBe(true)

    session.revision.value += 1
    plansRequest.resolve({
      plans: [{
        code: 'plan-1',
        scope: 'single_match',
        team_code: 'chengdu',
        title: '单场订阅',
        description: '',
        price_cents: 100,
      }],
      active_subscriptions: [],
      email_target: null,
    })
    statusRequest.resolve({
      subscribed: true,
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'a@example.com' },
    })
    await pending

    expect(state.refluxSubscriptionPlans.value).toEqual([])
    expect(state.selectedRefluxSubscriptionPlanCode.value).toBe('')
    expect(state.refluxSubscriptionEmail.value).toBe('')
    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    expect(state.refluxSubscriptionLoading.value).toBe(true)
    expect(toasts).toEqual([])
  })

  test('订阅支付完成后会话被替代，不置 subscribed、不关弹层、不弹成功 toast', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.canRequestWxPayment.value = true
    state.refluxSubscriptionEmail.value = 'a@example.com'
    state.selectedRefluxSubscriptionPlanCode.value = 'plan-1'
    state.refluxSubscriptionSheetVisible.value = true
    const payment = deferred<void>()
    const confirmation = deferred<boolean>()
    const subscription = useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: async () => ({
        order_no: 'order-1',
        params: { timeStamp: '', nonceStr: '', package: '', signType: 'RSA', paySign: '' },
      }),
      getRefluxSubscriptionPlans: unusedAction(),
      getRefluxSubscriptionStatus: unusedAction(),
      waitForPaidOrder: () => confirmation.promise,
      requestTicketWatchPayment: () => payment.promise,
    }, session)

    const pending = subscription.submitRefluxSubscriptionOrder()
    await Promise.resolve()
    await Promise.resolve()
    expect(state.refluxSubscriptionSubmitting.value).toBe(true)

    payment.resolve()
    await Promise.resolve()
    // 支付完成、开通确认在途时切换到 B（revision 变化）。
    session.revision.value += 1
    confirmation.resolve(true)
    await pending

    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    expect(state.refluxSubscriptionSheetVisible.value).toBe(true)
    expect(state.refluxSubscriptionSubmitting.value).toBe(true)
    expect(toasts).toEqual([])
  })

  test('订阅状态请求在途时切换球队/比赛：reset 复位 loading，之后可再次刷新', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const statusA = deferred<RefluxSubscriptionStatusResponse>()
    let statusCalls = 0
    const subscription = useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: unusedAction(),
      getRefluxSubscriptionPlans: unusedAction(),
      getRefluxSubscriptionStatus: () => {
        statusCalls += 1
        return statusCalls === 1
          ? statusA.promise
          : Promise.resolve({
            subscribed: true,
            active_subscriptions: [],
            email_target: { channel: 'email', target: 'b@example.com' },
          })
      },
      waitForPaidOrder: unusedAction(),
      requestTicketWatchPayment: unusedAction(),
    }, session)
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    // 旧比赛的订阅状态请求在途。
    const pendingA = subscription.refreshRefluxSubscriptionStatus()
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)

    // 切换球队/比赛：reset 流程复位 loading（旧 finally 因快照失效跳过清理）。
    boards.resetCurrentBoard()
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
    expect(state.refluxSubscriptionSheetVisible.value).toBe(false)

    // 旧请求晚到：不恢复旧数据，也不再改动 loading。
    statusA.resolve({
      subscribed: true,
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'stale@example.com' },
    })
    await pendingA
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
    expect(state.refluxSubscriptionEmail.value).toBe('')

    // 新比赛下可以再次刷新并正常发布。
    state.currentMatch.value = match(2)
    await subscription.refreshRefluxSubscriptionStatus()
    expect(statusCalls).toBe(2)
    expect(state.refluxSubscriptionEmail.value).toBe('b@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
  })

  test('旧订阅请求晚到不清掉新请求的 loading', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const statusA = deferred<RefluxSubscriptionStatusResponse>()
    const statusB = deferred<RefluxSubscriptionStatusResponse>()
    let statusCalls = 0
    const subscription = useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: unusedAction(),
      getRefluxSubscriptionPlans: unusedAction(),
      getRefluxSubscriptionStatus: () => {
        statusCalls += 1
        return statusCalls === 1 ? statusA.promise : statusB.promise
      },
      waitForPaidOrder: unusedAction(),
      requestTicketWatchPayment: unusedAction(),
    }, session)
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingA = subscription.refreshRefluxSubscriptionStatus()
    boards.resetCurrentBoard()
    // 新比赛的新请求已开始。
    state.currentMatch.value = match(2)
    const pendingB = subscription.refreshRefluxSubscriptionStatus()
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)

    // 旧请求晚到：finally 跳过，不能清新请求的 loading。
    statusA.resolve({ subscribed: false, active_subscriptions: [], email_target: null })
    await pendingA
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)

    statusB.resolve({
      subscribed: true,
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'b@example.com' },
    })
    await pendingB
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
    expect(state.refluxSubscriptionEmail.value).toBe('b@example.com')
  })
})

describe('ticket watch matchId session reset effects', () => {
  test('解锁下单在途时切换账号：重置 sheet 状态与权益，新账号可重新打开并解锁', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub({ hasAccessToken: ref(true) })
    state.currentMatch.value = match(1)
    state.currentUser.value = { has_wechat_binding: true } as CurrentUser
    state.canRequestWxPayment.value = true
    state.matchIdSheetVisible.value = true
    const orderRequest = deferred<{
      order_no: string
      wx_pay_params: { timeStamp: string; nonceStr: string; package: string; signType: string; paySign: string }
    }>()
    const actions = useTicketWatchMatchId(state, () => {}, {
      createMatchIdOrder: () => orderRequest.promise,
      getMatchIdEntitlement: async () => ({
        unlocked: true,
        via: 'purchase',
        effective_tier: 'V1',
      }),
      requestTicketWatchPayment: async () => {},
      waitForPaidOrder: async () => true,
    }, session)

    // 旧账号下单在途。
    const pendingA = actions.payForMatchId()
    await Promise.resolve()
    expect(state.matchIdSheetState.value).toBe('paying')

    // 切换账号：会话 reset 重置解锁操作状态与私有权益。
    session.revision.value += 1
    actions.resetMatchIdSessionState()
    expect(state.matchIdSheetState.value).toBe('locked')
    expect(state.matchIdEntitlement.value).toBeNull()
    expect(state.matchIdSheetVisible.value).toBe(false)

    // 旧下单晚到：不恢复 paying，也不修改新流程。
    orderRequest.resolve({
      order_no: 'order-a',
      wx_pay_params: { timeStamp: '', nonceStr: '', package: '', signType: 'RSA', paySign: '' },
    })
    await pendingA
    expect(state.matchIdSheetState.value).toBe('locked')
    expect(state.matchIdEntitlement.value).toBeNull()

    // 新账号可重新打开 sheet 并发布自己的权益。
    await actions.openMatchIdSheet()
    expect(state.matchIdSheetVisible.value).toBe(true)
    expect(state.matchIdSheetState.value).toBe('unlocked')
    expect(state.matchIdEntitlement.value?.unlocked).toBe(true)
    expect(toasts).toEqual([])
  })
})

describe('ticket watch session sync（后台恢复后刷新私有区）', () => {
  interface SyncFixture {
    state: ReturnType<typeof useTicketWatchState>
    session: SessionStub
    sessionUser: Ref<CurrentUser | null>
    resetCalls: number
    tierLoads: number
    boardLoads: string[]
    historyLoads: string[]
    setVisible: (visible: boolean) => void
    stop: () => void
  }

  /** 与页面相同的接线，走真实 Vue watch：revision 变化由会话 stub 真实发布触发。 */
  function createSyncFixture(options?: {
    underReview?: boolean
    visible?: boolean
  }): SyncFixture {
    const state = useTicketWatchState()
    const sessionUser = ref<CurrentUser | null>(null)
    const session = createSessionStub({ currentUser: sessionUser })
    state.systemConfigUnderReview.value = options?.underReview ?? false
    // 制造一些旧会话留下的脏状态，验证 revision 变化时被复位。
    state.currentMatch.value = match(1)
    state.refluxSubscriptionStatusLoading.value = true
    state.refluxSubscriptionSheetVisible.value = true

    const fixture: SyncFixture = {
      state,
      session,
      sessionUser,
      resetCalls: 0,
      tierLoads: 0,
      boardLoads: [],
      historyLoads: [],
      setVisible: () => {},
      stop: () => {},
    }
    let visible = options?.visible ?? true
    fixture.setVisible = (value: boolean) => {
      visible = value
    }
    const sync = createTicketWatchSessionSync(state, session, {
      resetSessionScopedBoards: () => {
        fixture.resetCalls += 1
        state.currentMatch.value = null
        state.refluxSubscriptionStatusLoading.value = false
        state.refluxSubscriptionSheetVisible.value = false
      },
      loadViewerMembershipTier: async () => {
        fixture.tierLoads += 1
        state.currentUser.value = sessionUser.value
      },
      loadCurrentBoard: async (reason) => {
        fixture.boardLoads.push(reason)
      },
      loadHistoryMatches: async (reason) => {
        fixture.historyLoads.push(reason)
      },
      isPageVisible: () => visible,
    })

    const scope = effectScope()
    scope.run(() => {
      watch(session.revision, () => {
        sync.handleSessionRevisionChange()
      })
    })
    fixture.stop = () => scope.stop()
    return fixture
  }

  test('后台恢复发布已认证用户：复位旧状态并重载档位与当前看板，无需切 Tab', async () => {
    const fixture = createSyncFixture()

    // 后台静默恢复完成：session 发布用户并递增 revision。
    fixture.sessionUser.value = { id: 'user-a', membership_tier: 'V3' } as CurrentUser
    fixture.session.revision.value += 1
    await nextTick()
    await nextTick()

    expect(fixture.resetCalls).toBe(1)
    expect(fixture.state.currentMatch.value).toBeNull()
    expect(fixture.tierLoads).toBe(1)
    expect(fixture.boardLoads).toEqual(['initial'])
    expect(fixture.state.currentUser.value?.id).toBe('user-a')
    fixture.stop()
  })

  test('退出/失效（无用户发布）：只清理私有状态，不重载看板', async () => {
    const fixture = createSyncFixture()

    fixture.session.revision.value += 1
    await nextTick()

    expect(fixture.resetCalls).toBe(1)
    expect(fixture.tierLoads).toBe(0)
    expect(fixture.boardLoads).toEqual([])
    fixture.stop()
  })

  test('审核中即使恢复了用户也不加载会员看板数据', async () => {
    const fixture = createSyncFixture({ underReview: true })

    fixture.sessionUser.value = { id: 'user-a' } as CurrentUser
    fixture.session.revision.value += 1
    await nextTick()
    await nextTick()

    expect(fixture.resetCalls).toBe(1)
    expect(fixture.tierLoads).toBe(0)
    expect(fixture.boardLoads).toEqual([])
    fixture.stop()
  })

  test('会员权益锁定时恢复只刷新档位镜像，不加载看板', async () => {
    const fixture = createSyncFixture()

    fixture.sessionUser.value = {
      id: 'user-a',
      membership_tier: 'V3',
      membership_benefits_enabled: false,
    } as CurrentUser
    fixture.session.revision.value += 1
    await nextTick()
    await nextTick()

    expect(fixture.tierLoads).toBe(1)
    expect(fixture.boardLoads).toEqual([])
    fixture.stop()
  })

  test('隐藏页面的后台恢复不启动轮询、不发起重载', async () => {
    const scheduled = new Set<number>()
    let nextId = 0
    globalThis.setInterval = ((_callback: unknown, _delay: number) => {
      const id = ++nextId
      scheduled.add(id)
      return id
    }) as typeof setInterval
    globalThis.clearInterval = ((id: number) => {
      scheduled.delete(id)
    }) as typeof clearInterval

    const fixture = createSyncFixture({ visible: false })
    const scope = effectScope()
    // 页面已隐藏：freshness clock 停止（pageVisible=false），监控未开启。
    const polling = scope.run(() =>
      useTicketWatchPolling(fixture.state, async () => {}, fixture.session),
    )!
    polling.stopPolling()
    polling.stopFreshnessClock()

    fixture.sessionUser.value = { id: 'user-a' } as CurrentUser
    fixture.session.revision.value += 1
    await nextTick()
    await nextTick()

    // 隐藏页面只复位私有数据，不在后台发起重载（下次 onShow 统一加载）。
    expect(fixture.resetCalls).toBe(1)
    expect(fixture.tierLoads).toBe(0)
    expect(fixture.boardLoads).toEqual([])
    expect(fixture.historyLoads).toEqual([])
    expect(fixture.state.isMonitoringActive.value).toBe(false)
    expect(scheduled.size).toBe(0)
    scope.stop()
    fixture.stop()
  })
})

describe('ticket watch board generation（A→B→A 旧请求防复活）', () => {
  function unusedAction() {
    return async () => {
      throw new Error('unused')
    }
  }

  function statusResponse(email: string | null, subscribed: boolean): RefluxSubscriptionStatusResponse {
    return {
      subscribed,
      active_subscriptions: [],
      email_target: email ? { channel: 'email', target: email } : null,
    }
  }

  function createSubscriptionPair(
    state: ReturnType<typeof useTicketWatchState>,
    session: SessionStub,
    handlers: {
      getRefluxSubscriptionStatus?: () => Promise<RefluxSubscriptionStatusResponse>
      getRefluxSubscriptionPlans?: () => Promise<RefluxSubscriptionPlansResponse>
      createRefluxSubscriptionOrder?: () => Promise<{
        order_no: string
        params: { timeStamp: string; nonceStr: string; package: string; signType: string; paySign: string }
      }>
      waitForPaidOrder?: () => Promise<boolean>
      requestTicketWatchPayment?: () => Promise<void>
    },
  ) {
    return useTicketWatchSubscription(state, () => {}, {
      createRefluxSubscriptionOrder: handlers.createRefluxSubscriptionOrder ?? unusedAction(),
      getRefluxSubscriptionPlans: handlers.getRefluxSubscriptionPlans ?? unusedAction(),
      getRefluxSubscriptionStatus: handlers.getRefluxSubscriptionStatus ?? unusedAction(),
      waitForPaidOrder: handlers.waitForPaidOrder ?? unusedAction(),
      requestTicketWatchPayment: handlers.requestTicketWatchPayment ?? unusedAction(),
    }, session)
  }

  test('旧 A 状态请求 pending → B → A → 新 A pending → 旧 A 返回：邮箱/subscribed 不被写入，新 loading 仍 true', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const oldStatus = deferred<RefluxSubscriptionStatusResponse>()
    const newStatus = deferred<RefluxSubscriptionStatusResponse>()
    let statusCalls = 0
    const subscription = createSubscriptionPair(state, session, {
      getRefluxSubscriptionStatus: () => {
        statusCalls += 1
        return statusCalls === 1 ? oldStatus.promise : newStatus.promise
      },
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.refreshRefluxSubscriptionStatus()
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)

    // A → B → A：两次 reset（比赛上下文被替换），随后新 A 的订阅请求开始。
    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    const pendingNew = subscription.refreshRefluxSubscriptionStatus()
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)
    expect(statusCalls).toBe(2)

    // 旧 A 晚到：不覆盖新请求的上下文，也不清掉新 loading。
    oldStatus.resolve(statusResponse('stale@example.com', true))
    await pendingOld
    expect(state.refluxSubscriptionEmail.value).toBe('')
    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(true)

    newStatus.resolve(statusResponse('fresh@example.com', true))
    await pendingNew
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
  })

  test('新 A 先发布结果 → 旧 A 后成功：保留新结果，不清新 owner 的 loading', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const oldStatus = deferred<RefluxSubscriptionStatusResponse>()
    const newStatus = deferred<RefluxSubscriptionStatusResponse>()
    let statusCalls = 0
    const subscription = createSubscriptionPair(state, session, {
      getRefluxSubscriptionStatus: () => {
        statusCalls += 1
        return statusCalls === 1 ? oldStatus.promise : newStatus.promise
      },
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.refreshRefluxSubscriptionStatus()
    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    const pendingNew = subscription.refreshRefluxSubscriptionStatus()

    // 新 A 先返回并发布。
    newStatus.resolve(statusResponse('fresh@example.com', true))
    await pendingNew
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)

    // 旧 A 后成功：不覆盖新结果。
    oldStatus.resolve(statusResponse('stale@example.com', false))
    await pendingOld
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
  })

  test('新 A 先发布结果 → 旧 A 后失败：保留新结果，不弹旧错误、不清空新邮箱', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const oldStatus = deferred<RefluxSubscriptionStatusResponse>()
    let statusCalls = 0
    const subscription = createSubscriptionPair(state, session, {
      getRefluxSubscriptionStatus: () => {
        statusCalls += 1
        return statusCalls === 1
          ? oldStatus.promise
          : Promise.resolve(statusResponse('fresh@example.com', true))
      },
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.refreshRefluxSubscriptionStatus()
    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    await subscription.refreshRefluxSubscriptionStatus()
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')

    // 旧 A 后失败：catch 分支同样被 generation 挡住，不能清空新结果。
    oldStatus.reject(new ApiRequestError('网络连接不可用，请检查网络后重试', 0, true))
    await pendingOld
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
  })

  test('套餐加载在 A→B→A 后才返回：不发布旧 plans/email，不关新 sheet、不清新 loading', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    const oldPlans = deferred<RefluxSubscriptionPlansResponse>()
    const newPlans = deferred<RefluxSubscriptionPlansResponse>()
    let plansCalls = 0
    const subscription = createSubscriptionPair(state, session, {
      getRefluxSubscriptionPlans: () => {
        plansCalls += 1
        return plansCalls === 1 ? oldPlans.promise : newPlans.promise
      },
      getRefluxSubscriptionStatus: () =>
        Promise.resolve(statusResponse('fresh@example.com', false)),
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.openRefluxSubscriptionSheet()
    expect(state.refluxSubscriptionLoading.value).toBe(true)

    // A → B → A：旧 sheet 被 reset 关闭，新 sheet 重新打开（新请求仍在途）。
    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    const pendingNew = subscription.openRefluxSubscriptionSheet()
    expect(state.refluxSubscriptionSheetVisible.value).toBe(true)
    expect(state.refluxSubscriptionLoading.value).toBe(true)

    // 旧套餐请求晚到：不覆盖新 sheet 的数据与 loading。
    oldPlans.resolve({
      plans: [{
        code: 'plan-stale',
        scope: 'single_match',
        team_code: 'chengdurongcheng',
        title: '旧套餐',
        description: '',
        price_cents: 100,
      }],
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'stale@example.com' },
    })
    await pendingOld
    expect(state.refluxSubscriptionPlans.value).toEqual([])
    expect(state.refluxSubscriptionEmail.value).toBe('')
    expect(state.refluxSubscriptionLoading.value).toBe(true)

    newPlans.resolve({
      plans: [{
        code: 'plan-new',
        scope: 'single_match',
        team_code: 'chengdurongcheng',
        title: '新套餐',
        description: '',
        price_cents: 200,
      }],
      active_subscriptions: [],
      email_target: { channel: 'email', target: 'fresh@example.com' },
    })
    await pendingNew
    expect(state.refluxSubscriptionPlans.value.map((plan) => plan.code)).toEqual(['plan-new'])
    expect(state.refluxSubscriptionEmail.value).toBe('fresh@example.com')
    expect(state.refluxSubscriptionLoading.value).toBe(false)
    expect(toasts).toEqual([])
  })

  test('下单在途时 A→B→A：旧订单不能在上下文失效后拉起微信支付', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    state.canRequestWxPayment.value = true
    state.refluxSubscriptionEmail.value = 'a@example.com'
    state.selectedRefluxSubscriptionPlanCode.value = 'plan-1'
    state.refluxSubscriptionSheetVisible.value = true
    const orderRequest = deferred<{
      order_no: string
      params: { timeStamp: string; nonceStr: string; package: string; signType: string; paySign: string }
    }>()
    let paymentCalls = 0
    const subscription = createSubscriptionPair(state, session, {
      createRefluxSubscriptionOrder: () => orderRequest.promise,
      requestTicketWatchPayment: () => {
        paymentCalls += 1
        return Promise.resolve()
      },
      waitForPaidOrder: () => Promise.resolve(true),
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.submitRefluxSubscriptionOrder()
    expect(state.refluxSubscriptionSubmitting.value).toBe(true)

    // 下单在途时 A → B → A：旧订单返回后不得拉起微信支付、不得修改新 sheet。
    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    state.refluxSubscriptionSheetVisible.value = true

    orderRequest.resolve({
      order_no: 'order-stale',
      params: { timeStamp: '', nonceStr: '', package: '', signType: 'RSA', paySign: '' },
    })
    await pendingOld

    expect(paymentCalls).toBe(0)
    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    expect(toasts).toEqual([])
  })

  test('支付确认在途时 A→B→A：旧支付结果不修改新 sheet 状态', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    state.canRequestWxPayment.value = true
    state.refluxSubscriptionEmail.value = 'a@example.com'
    state.selectedRefluxSubscriptionPlanCode.value = 'plan-1'
    state.refluxSubscriptionSheetVisible.value = true
    const confirmation = deferred<boolean>()
    const subscription = createSubscriptionPair(state, session, {
      createRefluxSubscriptionOrder: () =>
        Promise.resolve({
          order_no: 'order-1',
          params: { timeStamp: '', nonceStr: '', package: '', signType: 'RSA', paySign: '' },
        }),
      requestTicketWatchPayment: () => Promise.resolve(),
      waitForPaidOrder: () => confirmation.promise,
    })
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = subscription.submitRefluxSubscriptionOrder()
    await Promise.resolve()
    await Promise.resolve()

    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)
    state.refluxSubscriptionSheetVisible.value = true

    confirmation.resolve(true)
    await pendingOld

    expect(state.refluxSubscriptionSubscribed.value).toBe(false)
    expect(state.refluxSubscriptionSheetVisible.value).toBe(true)
    expect(toasts).toEqual([])
  })

  test('比赛 ID 解锁下单在途时 A→B→A：旧订单不拉起微信支付', async () => {
    const toasts: string[] = []
    testGlobal.uni = {
      showToast: (options: { title: string }) => {
        toasts.push(options.title)
      },
    } as unknown as typeof uni
    const state = useTicketWatchState()
    const session = createSessionStub({ hasAccessToken: ref(true) })
    state.currentMatch.value = match(1)
    state.currentUser.value = { has_wechat_binding: true } as CurrentUser
    state.canRequestWxPayment.value = true
    state.matchIdSheetVisible.value = true
    const orderRequest = deferred<{
      order_no: string
      wx_pay_params: { timeStamp: string; nonceStr: string; package: string; signType: string; paySign: string }
    }>()
    let paymentCalls = 0
    const actions = useTicketWatchMatchId(state, () => {}, {
      createMatchIdOrder: () => orderRequest.promise,
      getMatchIdEntitlement: async () => ({
        unlocked: true,
        via: 'purchase',
        effective_tier: 'V1',
      }),
      requestTicketWatchPayment: () => {
        paymentCalls += 1
        return Promise.resolve()
      },
      waitForPaidOrder: async () => true,
    }, session)
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, session)

    const pendingOld = actions.payForMatchId()
    await Promise.resolve()
    expect(state.matchIdSheetState.value).toBe('paying')

    boards.resetCurrentBoard()
    boards.resetCurrentBoard()
    state.currentMatch.value = match(1)

    orderRequest.resolve({
      order_no: 'order-stale',
      wx_pay_params: { timeStamp: '', nonceStr: '', package: '', signType: 'RSA', paySign: '' },
    })
    await pendingOld

    expect(paymentCalls).toBe(0)
    expect(toasts).toEqual([])
  })

  test('同一比赛的普通库存轮询不失效在途的合法订阅请求', async () => {
    const state = useTicketWatchState()
    const session = createSessionStub()
    state.regions.value = [
      { block_name: 'A1', price: '180', usable_count: 10, estate: 0 },
    ]
    state.currentMatch.value = match(1)
    state.currentUser.value = { id: 'user-a' } as CurrentUser
    state.hasLoadedCurrentBoard.value = true
    state.currentLoading.value = false
    const statusRequest = deferred<RefluxSubscriptionStatusResponse>()
    const subscription = createSubscriptionPair(state, session, {
      getRefluxSubscriptionStatus: () => statusRequest.promise,
    })
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getCurrentTicketWatchBoard: async () => ({
        current_match: match(1),
        group_ticket_active: false,
        message: '',
        inventory: [],
        block_interests: [],
        tracked_interests: [],
      }),
    }, session)

    const pendingStatus = subscription.refreshRefluxSubscriptionStatus()
    // 同一场比赛的库存轮询：不递增 generation，在途订阅请求保持有效。
    await boards.loadCurrentBoard('poll')

    statusRequest.resolve(statusResponse('a@example.com', true))
    await pendingStatus
    expect(state.refluxSubscriptionEmail.value).toBe('a@example.com')
    expect(state.refluxSubscriptionSubscribed.value).toBe(true)
    expect(state.refluxSubscriptionStatusLoading.value).toBe(false)
  })

  test('generation 单调递增且 reset 不归零', () => {
    const state = useTicketWatchState()
    const boards = useTicketWatchBoards(state, async () => {}, { ...source }, createSessionStub())

    expect(state.currentBoardGeneration.value).toBe(0)
    boards.resetCurrentBoard()
    expect(state.currentBoardGeneration.value).toBe(1)
    boards.resetCurrentBoard()
    expect(state.currentBoardGeneration.value).toBe(2)
  })
})

describe('ticket watch 会话恢复看板重载（真实 boards + sessionSync + Vue watch）', () => {
  interface Deferred<T> {
    promise: Promise<T>
    resolve: (value: T) => void
    reject: (error: unknown) => void
  }

  interface InventoryCall {
    matchId: number
    request: Deferred<TicketWatchInventoryEntry[]>
    settled: boolean
  }

  function boardResponse(
    current: TicketWatchMatchSummary | null,
  ): TicketWatchCurrentBoardResponse {
    return {
      current_match: current,
      group_ticket_active: false,
      message: '',
      inventory: [],
      block_interests: [],
      tracked_interests: [],
    }
  }

  function inventoryEntry(occurrences: number): TicketWatchInventoryEntry[] {
    return [
      {
        block_name: 'A1',
        occurrences,
        latest_time: '2026-09-01T10:15:00+08:00',
      },
    ]
  }

  /** 与页面相同的真实接线：真实 boards/state/sessionSync + 真实 Vue watch，仅 API 层打桩。 */
  function createBoardSyncFixture(options?: {
    tab?: 'current' | 'history' | 'history-stats'
    team?: 'chengdurongcheng' | 'yunnanyukun'
    visible?: boolean
    ensureReady?: () => Promise<CurrentUser | null>
  }) {
    const state = useTicketWatchState()
    state.regions.value = [
      { block_name: 'A1', price: '180', usable_count: 10, estate: 0 },
    ]
    state.activeTab.value = options?.tab ?? 'current'
    state.selectedTeam.value = options?.team ?? 'chengdurongcheng'
    state.currentLoading.value = false
    state.historyLoading.value = false
    const sessionUser = ref<CurrentUser | null>(null)
    const session = createSessionStub({
      currentUser: sessionUser,
      ensureSessionReady:
        options?.ensureReady ?? (async () => sessionUser.value),
    })
    const currentBoardCalls: Deferred<TicketWatchCurrentBoardResponse>[] = []
    const yukunCurrentCalls: Deferred<TicketWatchCurrentMatchResponse>[] = []
    const historyMatchesCalls: Deferred<TicketWatchMatchSummary[]>[] = []
    const inventoryCalls: InventoryCall[] = []
    const boards = useTicketWatchBoards(state, async () => {}, {
      ...source,
      getCurrentTicketWatchBoard: () => {
        const request = deferred<TicketWatchCurrentBoardResponse>()
        currentBoardCalls.push(request)
        return request.promise
      },
      getYukunCurrentTicketWatchMatch: () => {
        const request = deferred<TicketWatchCurrentMatchResponse>()
        yukunCurrentCalls.push(request)
        return request.promise
      },
      getYukunTicketWatchInventory: async () => [],
      getYukunTicketWatchRegions: async () => [],
      getTicketWatchMatches: () => {
        const request = deferred<TicketWatchMatchSummary[]>()
        historyMatchesCalls.push(request)
        return request.promise
      },
      getTicketWatchInventorySince: (matchId: number) => {
        const request = deferred<TicketWatchInventoryEntry[]>()
        inventoryCalls.push({ matchId, request, settled: false })
        return request.promise
      },
      getTicketWatchBlockInterests: async () => [],
    }, session)
    let visible = options?.visible ?? true
    const sync = createTicketWatchSessionSync(state, session, {
      resetSessionScopedBoards: () => {
        boards.resetCurrentBoard()
        boards.resetHistoryBoard()
        state.currentUser.value = null
        state.viewerMembershipTier.value = 'V1'
      },
      loadViewerMembershipTier: boards.loadViewerMembershipTier,
      loadCurrentBoard: boards.loadCurrentBoard,
      loadHistoryMatches: boards.loadHistoryMatches,
      isPageVisible: () => visible,
    })
    const scope = effectScope()
    scope.run(() => {
      watch(session.revision, () => sync.handleSessionRevisionChange())
    })

    async function flush(): Promise<void> {
      await nextTick()
      for (let i = 0; i < 20; i += 1) {
        await Promise.resolve()
      }
      await nextTick()
    }

    function resolveInventory(
      matchId: number,
      entries: TicketWatchInventoryEntry[],
    ): void {
      for (const call of inventoryCalls) {
        if (!call.settled && call.matchId === matchId) {
          call.settled = true
          call.request.resolve(entries)
        }
      }
    }

    async function publishSessionUser(id: string): Promise<void> {
      sessionUser.value = { id, membership_tier: 'V3' } as CurrentUser
      session.revision.value += 1
      await flush()
    }

    return {
      state,
      session,
      sessionUser,
      boards,
      currentBoardCalls,
      yukunCurrentCalls,
      historyMatchesCalls,
      inventoryCalls,
      resolveInventory,
      flush,
      publishSessionUser,
      setVisible: (value: boolean) => {
        visible = value
      },
      stop: () => scope.stop(),
    }
  }

  test('游客 current 在途时静默登录成功：reset 允许新请求，旧结果与 finally 全部失效', async () => {
    const f = createBoardSyncFixture()
    const guest = f.boards.loadCurrentBoard()
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(1)
    expect(f.state.currentBoardRequestInFlight.value).toBe(true)

    // 后台静默登录成功并发布用户：revision watch 走真实 sessionSync。
    await f.publishSessionUser('user-b')
    // 新会话的 current 请求不被旧 inFlight 挡住。
    expect(f.currentBoardCalls.length).toBe(2)
    expect(f.state.currentLoading.value).toBe(true)

    // 旧游客响应晚到：不发布、不清新请求的 loading/inFlight。
    f.currentBoardCalls[0].resolve(boardResponse(match(1)))
    await guest
    await f.flush()
    expect(f.state.currentMatch.value).toBeNull()
    expect(f.state.hasLoadedCurrentBoard.value).toBe(false)
    expect(f.state.currentBoardRequestInFlight.value).toBe(true)
    expect(f.state.currentLoading.value).toBe(true)

    // 新会话响应正常发布。
    f.currentBoardCalls[1].resolve(boardResponse(match(2)))
    await f.flush()
    expect(f.state.currentMatch.value?.match_id).toBe(2)
    expect(f.state.hasLoadedCurrentBoard.value).toBe(true)
    expect(f.state.currentBoardRequestInFlight.value).toBe(false)
    expect(f.state.currentLoading.value).toBe(false)
    f.stop()
  })

  test('同会话同球队的在途 current 请求仍合并：poll 不重复发请求', async () => {
    const f = createBoardSyncFixture()
    const initial = f.boards.loadCurrentBoard('initial')
    await f.flush()
    await f.boards.loadCurrentBoard('poll')
    await f.boards.loadCurrentBoard('initial')
    expect(f.currentBoardCalls.length).toBe(1)

    f.currentBoardCalls[0].resolve(boardResponse(match(1)))
    await initial
    await f.flush()
    expect(f.state.hasLoadedCurrentBoard.value).toBe(true)

    // 请求完成后新的 poll 可以再发请求。
    const poll = f.boards.loadCurrentBoard('poll')
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(2)
    f.currentBoardCalls[1].resolve(boardResponse(match(1)))
    await poll
    await f.flush()
    expect(f.state.currentMatch.value?.match_id).toBe(1)
    expect(f.state.currentBoardRequestInFlight.value).toBe(false)
    f.stop()
  })

  test('恢复时可见 tab 为 history：清空的历史看板按新会话重载，不请求 current', async () => {
    const f = createBoardSyncFixture({ tab: 'history' })
    // 游客历史看板已加载。
    const guestHistory = f.boards.loadHistoryMatches('initial')
    await f.flush()
    expect(f.historyMatchesCalls.length).toBe(1)
    f.historyMatchesCalls[0].resolve([match(99)])
    await f.flush()
    f.resolveInventory(99, [])
    await guestHistory
    await f.flush()
    expect(f.state.historyMatches.value.map((item) => item.match_id)).toEqual([99])
    expect(f.state.displayedHistoryMatchId.value).toBe(99)
    expect(f.state.hasLoadedHistoryBoard.value).toBe(true)

    await f.publishSessionUser('user-b')
    // 历史看板被 reset 清空后按可见 tab 重载；current 看板不请求。
    expect(f.state.historyMatches.value).toEqual([])
    expect(f.state.hasLoadedHistoryBoard.value).toBe(false)
    expect(f.historyMatchesCalls.length).toBe(2)
    expect(f.currentBoardCalls.length).toBe(0)

    f.historyMatchesCalls[1].resolve([match(100)])
    await f.flush()
    f.resolveInventory(100, [])
    await f.flush()
    expect(f.state.historyMatches.value.map((item) => item.match_id)).toEqual([100])
    expect(f.state.selectedHistoryMatchId.value).toBe(100)
    expect(f.state.displayedHistoryMatchId.value).toBe(100)
    expect(f.state.hasLoadedHistoryBoard.value).toBe(true)
    expect(f.state.historyLoading.value).toBe(false)
    f.stop()
  })

  test('恢复时可见 tab 为 history-stats：重载历史看板而非 current', async () => {
    const f = createBoardSyncFixture({ tab: 'history-stats' })
    await f.publishSessionUser('user-b')
    expect(f.historyMatchesCalls.length).toBe(1)
    expect(f.currentBoardCalls.length).toBe(0)

    f.historyMatchesCalls[0].resolve([match(100)])
    await f.flush()
    f.resolveInventory(100, [])
    await f.flush()
    expect(f.state.historyMatches.value.map((item) => item.match_id)).toEqual([100])
    expect(f.state.hasLoadedHistoryBoard.value).toBe(true)
    f.stop()
  })

  test('球队切换作废旧上下文：旧球队晚到结果不发布、不清新 loading，新球队库存阶段不被订阅 generation 误判', async () => {
    const f = createBoardSyncFixture()
    const chengdu = f.boards.loadCurrentBoard()
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(1)

    // 页面 switchTeam 的 boards 层等价序列：reset 后新球队上下文重载。
    f.state.selectedTeam.value = 'yunnanyukun'
    f.boards.resetCurrentBoard()
    f.boards.resetHistoryBoard()
    const yukun = f.boards.loadCurrentBoard('initial')
    await f.flush()
    expect(f.yukunCurrentCalls.length).toBe(1)
    expect(f.state.currentLoading.value).toBe(true)

    // 旧成都响应晚到：不发布、不清新请求状态。
    f.currentBoardCalls[0].resolve(boardResponse(match(1)))
    await chengdu
    await f.flush()
    expect(f.state.currentMatch.value).toBeNull()
    expect(f.state.currentBoardRequestInFlight.value).toBe(true)
    expect(f.state.currentLoading.value).toBe(true)

    // 玉昆响应：publishCurrentMatch 递增订阅 generation 后，
    // 同一请求的库存/区域阶段仍属同一 owner，不被误判为旧。
    const generationBefore = f.state.currentBoardGeneration.value
    f.yukunCurrentCalls[0].resolve({
      current_match: match(7),
      group_ticket_active: false,
      message: '',
    })
    await yukun
    await f.flush()
    expect(f.state.currentBoardGeneration.value).toBe(generationBefore + 1)
    expect(f.state.currentMatch.value?.match_id).toBe(7)
    expect(f.state.hasLoadedCurrentBoard.value).toBe(true)
    expect(f.state.currentLoading.value).toBe(false)
    expect(f.state.currentBoardRequestInFlight.value).toBe(false)
    f.stop()
  })

  test('恢复链 tier 在途时会话再次变化：旧 reload 不再请求看板', async () => {
    const ensure = deferred<CurrentUser | null>()
    const f = createBoardSyncFixture({ ensureReady: () => ensure.promise })
    f.sessionUser.value = { id: 'user-b', membership_tier: 'V3' } as CurrentUser
    f.session.revision.value += 1
    await f.flush()
    // tier 在途，看板尚未请求。
    expect(f.currentBoardCalls.length).toBe(0)
    expect(f.historyMatchesCalls.length).toBe(0)

    // 恢复期间会话被退出/替换。
    f.sessionUser.value = null
    f.session.revision.value += 1
    await f.flush()
    ensure.resolve(null)
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(0)
    expect(f.historyMatchesCalls.length).toBe(0)
    expect(f.state.currentUser.value).toBeNull()
    f.stop()
  })

  test('恢复链 tier 在途时切换球队：旧 reload 不再请求看板', async () => {
    const ensure = deferred<CurrentUser | null>()
    const f = createBoardSyncFixture({ ensureReady: () => ensure.promise })
    f.sessionUser.value = { id: 'user-b', membership_tier: 'V3' } as CurrentUser
    f.session.revision.value += 1
    await f.flush()

    f.state.selectedTeam.value = 'yunnanyukun'
    ensure.resolve(f.sessionUser.value)
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(0)
    expect(f.yukunCurrentCalls.length).toBe(0)
    expect(f.historyMatchesCalls.length).toBe(0)
    f.stop()
  })

  test('恢复链在途时页面隐藏：不再继续 reload', async () => {
    const ensure = deferred<CurrentUser | null>()
    const f = createBoardSyncFixture({ ensureReady: () => ensure.promise })
    f.sessionUser.value = { id: 'user-b', membership_tier: 'V3' } as CurrentUser
    f.session.revision.value += 1
    await f.flush()

    f.setVisible(false)
    ensure.resolve(f.sessionUser.value)
    await f.flush()
    expect(f.currentBoardCalls.length).toBe(0)
    expect(f.historyMatchesCalls.length).toBe(0)
    f.stop()
  })

  test('隐藏页面的恢复：只清理私有数据，不发起 reload', async () => {
    const f = createBoardSyncFixture({ visible: false })
    f.state.currentMatch.value = match(1)
    await f.publishSessionUser('user-b')
    expect(f.state.currentMatch.value).toBeNull()
    expect(f.state.currentUser.value).toBeNull()
    expect(f.currentBoardCalls.length).toBe(0)
    expect(f.historyMatchesCalls.length).toBe(0)
    f.stop()
  })

  test('reset 作废旧历史选择请求：旧结果不覆盖新上下文、不写缓存、不清新 loading', async () => {
    const f = createBoardSyncFixture({ tab: 'history' })
    const initial = f.boards.loadHistoryMatches('initial')
    await f.flush()
    f.historyMatchesCalls[0].resolve([match(1), match(2)])
    await f.flush()
    f.resolveInventory(1, [])
    f.resolveInventory(2, [])
    await initial
    await f.flush()
    expect(f.state.displayedHistoryMatchId.value).toBe(1)

    // 用户选择比赛 2：库存请求在途。
    const staleSelection = f.boards.handleHistoryMatchSelect(2)
    await f.flush()
    expect(f.state.historySelectionLoading.value).toBe(true)
    const staleInventory = f.inventoryCalls.find(
      (call) => !call.settled && call.matchId === 2,
    )
    expect(staleInventory?.matchId).toBe(2)

    // 会话恢复：reset 作废旧选择并重载历史 [3, 4]。
    await f.publishSessionUser('user-b')
    expect(f.state.historySelectionLoading.value).toBe(false)
    expect(f.historyMatchesCalls.length).toBe(2)
    f.historyMatchesCalls[1].resolve([match(3), match(4)])
    await f.flush()
    f.resolveInventory(3, inventoryEntry(3))
    f.resolveInventory(4, [])
    await f.flush()
    expect(f.state.displayedHistoryMatchId.value).toBe(3)
    expect(f.state.historySections.value[0]?.total_occurrences).toBe(3)

    // 新会话选择比赛 4：loading 归新请求所有。
    const freshSelection = f.boards.handleHistoryMatchSelect(4)
    await f.flush()
    expect(f.state.historySelectionLoading.value).toBe(true)

    // 旧选择（比赛 2）晚到：不覆盖 displayed/sections、不写缓存、不清新 loading。
    staleInventory!.settled = true
    staleInventory!.request.resolve(inventoryEntry(22))
    await staleSelection
    await f.flush()
    expect(f.state.displayedHistoryMatchId.value).toBe(3)
    expect(f.state.historySections.value[0]?.total_occurrences).toBe(3)
    expect(f.state.historySectionsCache.value[2] ?? null).toBeNull()
    expect(f.state.historySelectionLoading.value).toBe(true)

    // 新选择正常完成。
    f.resolveInventory(4, inventoryEntry(4))
    await freshSelection
    await f.flush()
    expect(f.state.displayedHistoryMatchId.value).toBe(4)
    expect(f.state.historySections.value[0]?.total_occurrences).toBe(4)
    expect(f.state.historySelectionLoading.value).toBe(false)
    f.stop()
  })

  test('reset 作废旧 trend/看板请求：旧 finally 不清新 trend loading、不写旧上下文', async () => {
    const f = createBoardSyncFixture({ tab: 'history' })
    const initial = f.boards.loadHistoryMatches('initial')
    await f.flush()
    f.historyMatchesCalls[0].resolve([match(1)])
    await f.flush()
    // trend(1) 先发起，sections(1) 其次；都保持在途。
    expect(f.inventoryCalls.map((call) => call.matchId)).toEqual([1, 1])

    // 会话恢复：reset 作废旧 trend/看板请求，新会话重载历史 [3]。
    await f.publishSessionUser('user-b')
    expect(f.historyMatchesCalls.length).toBe(2)
    f.historyMatchesCalls[1].resolve([match(3)])
    await f.flush()
    expect(f.state.historyTrendLoading.value).toBe(true)

    // 旧 trend(1) 晚到：不清新 loading、不写旧 match 的 totals。
    const [staleTrend, staleSections] = f.inventoryCalls
    staleTrend.request.resolve(inventoryEntry(5))
    await f.flush()
    expect(f.state.historyTrendLoading.value).toBe(true)
    expect(f.state.historyTrendTotals.value[1] ?? null).toBeNull()

    // 旧 sections(1) 晚到：不覆盖新 displayed、不写缓存。
    staleSections.request.resolve(inventoryEntry(9))
    await initial
    await f.flush()
    expect(f.state.displayedHistoryMatchId.value).toBeNull()
    expect(f.state.historySectionsCache.value[1] ?? null).toBeNull()

    // 新会话请求完成：trend/sections 正常发布。
    f.resolveInventory(3, inventoryEntry(3))
    await f.flush()
    expect(f.state.historyTrendLoading.value).toBe(false)
    expect(f.state.historyTrendTotals.value[3]).toBe(3)
    expect(f.state.displayedHistoryMatchId.value).toBe(3)
    expect(f.state.hasLoadedHistoryBoard.value).toBe(true)
    f.stop()
  })
})
