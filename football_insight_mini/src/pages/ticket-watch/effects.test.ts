import { afterEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick, ref, watch, type Ref } from 'vue'
import * as ticketApi from '../../api/ticketWatch'
import type { CurrentUser } from '../../types/auth'
import type {
  RefluxSubscriptionPlansResponse,
  RefluxSubscriptionStatusResponse,
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
    stop: () => void
  }

  /** 与页面相同的接线，走真实 Vue watch：revision 变化由会话 stub 真实发布触发。 */
  function createSyncFixture(options?: { underReview?: boolean }): SyncFixture {
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
      stop: () => {},
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

  test('隐藏页面的后台恢复不启动轮询', async () => {
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

    const fixture = createSyncFixture()
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

    expect(fixture.boardLoads).toEqual(['initial'])
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
