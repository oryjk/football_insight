import { afterEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick } from 'vue'
import * as authApi from '../../api/auth'
import * as ticketApi from '../../api/ticketWatch'
import type { CurrentUser } from '../../types/auth'
import type {
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
} from '../../types/ticketWatch'
import { useTicketWatchBoards } from './useTicketWatchBoards'
import { useTicketWatchMatchId } from './useTicketWatchMatchId'
import { useTicketWatchPolling } from './useTicketWatchPolling'
import { useTicketWatchState } from './useTicketWatchState'

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((accept) => {
    resolve = accept
  })
  return { promise, resolve }
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

const source = { ...ticketApi, getCurrentUser: authApi.getCurrentUser }
const testGlobal = globalThis as typeof globalThis & { uni: typeof uni }
const originalUni = testGlobal.uni
const originalSetInterval = globalThis.setInterval
const originalClearInterval = globalThis.clearInterval

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
    })
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
    })
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
    })
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
    })
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
      useTicketWatchPolling(state, async () => {}),
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
      useTicketWatchPolling(state, () => request.promise),
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
