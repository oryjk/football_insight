import { describe, expect, test } from 'bun:test'
import type { CurrentUser } from '../../types/auth'
import type { TicketWatchMatchSummary } from '../../types/ticketWatch'
import { syncVisibleHistorySelection } from './ticketWatchHistorySelection'
import { useTicketWatchState } from './useTicketWatchState'

const match = (id: number): TicketWatchMatchSummary => ({
  match_id: id, external_match_id: String(id), round_number: 1,
  match_date: '2026-09-01', match_time: '19:30', kickoff_at: '2026-09-01T19:30:00+08:00',
  home_team_name: '成都蓉城', away_team_name: '浙江队', is_current: false,
})

function fixture() {
  const state = useTicketWatchState()
  state.isPageVisible.value = true
  state.activeTab.value = 'history'
  state.historyLoading.value = false
  state.hasLoadedHistoryBoard.value = true
  state.historyMatches.value = [match(1), match(2), match(3)]
  state.historyTrendTotals.value = { 1: 0, 2: 10, 3: 20 }
  state.currentUser.value = { membership_benefits_enabled: true } as CurrentUser
  return state
}

describe('visible history selection', () => {
  test('moves an empty or excluded selection to the first visible match', async () => {
    const state = fixture()
    state.selectedHistoryMatchId.value = 1
    const selected: number[] = []
    await syncVisibleHistorySelection(state, async id => { selected.push(id) })
    expect(selected).toEqual([2])
  })

  test('keeps a valid user selection without loading another match', async () => {
    const state = fixture()
    state.selectedHistoryMatchId.value = 3
    const selected: number[] = []
    await syncVisibleHistorySelection(state, async id => { selected.push(id) })
    expect(selected).toEqual([])
  })

  for (const guard of ['hidden', 'current', 'stats', 'loading', 'prefetching', 'not-loaded', 'reviewing', 'locked', 'empty'] as const) {
    test(`does not auto-select while ${guard}`, async () => {
      const state = fixture()
      if (guard === 'hidden') state.isPageVisible.value = false
      if (guard === 'current') state.activeTab.value = 'current'
      if (guard === 'stats') state.activeTab.value = 'history-stats'
      if (guard === 'loading') state.historyLoading.value = true
      if (guard === 'prefetching') state.historyTrendLoading.value = true
      if (guard === 'not-loaded') state.hasLoadedHistoryBoard.value = false
      if (guard === 'reviewing') state.systemConfigUnderReview.value = true
      if (guard === 'locked') state.currentUser.value = { membership_benefits_enabled: false } as CurrentUser
      if (guard === 'empty') state.historyTrendTotals.value = { 1: 0, 2: 0, 3: 0 }
      const selected: number[] = []
      await syncVisibleHistorySelection(state, async id => { selected.push(id) })
      expect(selected).toEqual([])
    })
  }
})
