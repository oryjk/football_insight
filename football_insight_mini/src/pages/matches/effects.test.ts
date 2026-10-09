import { afterEach, describe, expect, test } from 'bun:test'
import type { MatchCard, MatchListResponse, RoundReference } from '../../types/insight'
import { buildSeasonProgressRows, buildUpcomingSections, groupFinishedMatches } from './presentation'
import { useMatchesPage } from './useMatchesPage'

function match(id: number, roundNumber = id, overrides: Partial<MatchCard> = {}): MatchCard {
  return {
    match_id: id, round_number: roundNumber, match_date: '2026-10-10', match_time: '19:35',
    status: 'scheduled', home_team_id: 1, home_team_name: '成都蓉城', home_score: '',
    away_team_id: 2, away_team_name: '客队', away_score: '', home_team_avatar: null,
    away_team_avatar: null, leisu_match_id: null, home_corners: null, away_corners: null,
    corner_source: null, technical_stats: [], ...overrides,
  }
}

function round(roundNumber: number, status: RoundReference['status']): RoundReference {
  return { season: 2026, round_number: roundNumber, status, finalized_at: null, total_matches: 8, completed_matches: 0 }
}

function response(matches: MatchCard[]): MatchListResponse {
  return { view_kind: 'live', round_number: null, current_season: 2026, matches }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((accept) => { resolve = accept })
  return { promise, resolve }
}

const now = '2026-10-09T12:00:00+08:00'
const testGlobal = globalThis as typeof globalThis & { uni: typeof uni }
const originalUni = testGlobal.uni
afterEach(() => { testGlobal.uni = originalUni })

describe('matches presentation', () => {
  test('keeps current-round pending matches ahead of the next round', () => {
    const sections = buildUpcomingSections(
      [round(1, 'current'), round(2, 'upcoming')],
      [match(1, 1, { status: 'finished' }), match(2, 1, { status: 'live' }), match(3, 2)], now,
    )
    expect(sections).toEqual([{ title: '本轮进行中', roundNumber: 1, matches: [match(2, 1, { status: 'live' })] }])
  })

  test('falls forward to the next round when the current round is complete', () => {
    const sections = buildUpcomingSections(
      [round(1, 'current'), round(2, 'upcoming')], [match(1, 1, { status: 'finished' }), match(2, 2)], now,
    )
    expect(sections.map((section) => [section.title, section.roundNumber])).toEqual([['下一轮', 2]])
    expect(buildUpcomingSections([round(2, 'upcoming')], [match(2, 2)], now)[0]?.roundNumber).toBe(2)
    expect(buildUpcomingSections([round(1, 'completed')], [match(1, 1, { status: 'finished' })], now)).toEqual([])
  })

  test('groups completed results by descending round while retaining their order', () => {
    const groups = groupFinishedMatches([
      match(1, 1, { status: 'finished' }), match(2, 2, { status: 'finished' }),
      match(3, 2, { status: 'finished' }), match(4, 3),
    ], now)
    expect(groups.map((group) => [group.roundNumber, group.items.map((item) => item.match_id)])).toEqual([[2, [2, 3]], [1, [1]]])
  })

  test('keeps the season timeline in rows of fifteen including the current-round fill', () => {
    const rows = buildSeasonProgressRows(Array.from({ length: 17 }, (_, index) =>
      round(index + 1, index < 15 ? 'completed' : index === 15 ? 'current' : 'upcoming')))
    expect(rows.map((row) => [row.rounds.length, row.fillWidth])).toEqual([[15, '100%'], [2, '50%']])
    expect(buildSeasonProgressRows([])).toEqual([])
  })
})

describe('matches data and dialogs', () => {
  test('renders live matches while future rounds load, merges replacements, and tolerates partial failures', async () => {
    const future = deferred<MatchListResponse>()
    const requested = deferred<void>()
    const calls: Array<number | null> = []
    const state = useMatchesPage({}, {
      getAvailableRounds: async () => [round(1, 'current'), round(2, 'upcoming'), round(3, 'upcoming')],
      getMatches: async (request) => {
        calls.push(request.roundNumber)
        if (request.mode === 'live') return response([match(1)])
        if (request.roundNumber === 1) { requested.resolve(); return future.promise }
        throw new Error('future round unavailable')
      },
    })
    const loading = state.loadPage()
    await requested.promise
    expect(state.loading.value).toBe(false)
    expect(state.matches.value.map((item) => item.match_id)).toEqual([1])
    future.resolve(response([match(1, 1, { home_score: '2', status: 'finished' }), match(2)]))
    await loading
    expect(calls).toEqual([null, 1, 2])
    expect(state.matches.value.map((item) => item.match_id)).toEqual([1, 2])
    expect(state.matches.value[0]?.home_score).toBe('2')
    expect(state.errorMessage.value).toBe('')
  })

  test('retains the standalone entry loading policy and its future-round failure message', async () => {
    const future = deferred<MatchListResponse>()
    const requested = deferred<void>()
    const state = useMatchesPage({ progressiveLoad: false }, {
      getAvailableRounds: async () => [round(1, 'current')],
      getMatches: async (request) => {
        if (request.mode === 'live') return response([match(1)])
        requested.resolve()
        await future.promise
        throw new Error('future round unavailable')
      },
    })
    const loading = state.loadPage()
    await requested.promise
    expect(state.loading.value).toBe(true)
    future.resolve(response([]))
    await loading
    expect(state.loading.value).toBe(false)
    expect(state.errorMessage.value).toBe('future round unavailable')
  })

  test('reports the initial API error and ends the loading state', async () => {
    const state = useMatchesPage({}, {
      getAvailableRounds: async () => { throw new Error('rounds unavailable') },
      getMatches: async () => response([]),
    })
    await state.loadPage()
    expect(state.loading.value).toBe(false)
    expect(state.errorMessage.value).toBe('rounds unavailable')
  })

  test('loads the chosen round, clears it on close, and exposes dialog errors', async () => {
    const state = useMatchesPage({}, {
      getAvailableRounds: async () => [],
      getMatches: async (request) => {
        if (request.roundNumber === 2) throw new Error('round unavailable')
        return response([match(1)])
      },
    })
    await state.openRoundDialog(1)
    expect(state.selectedRoundNumber.value).toBe(1)
    expect(state.selectedRoundMatches.value.map((item) => item.match_id)).toEqual([1])
    expect(state.roundDialogLoading.value).toBe(false)
    state.closeRoundDialog()
    expect(state.selectedRoundNumber.value).toBeNull()
    expect(state.selectedRoundMatches.value).toEqual([])
    await state.openRoundDialog(2)
    expect(state.roundDialogErrorMessage.value).toBe('round unavailable')
    state.closeRoundDialog()
    expect(state.roundDialogErrorMessage.value).toBe('')
  })

  test('opens technical statistics only when data exists and leaves the round sheet selected', () => {
    const messages: string[] = []
    testGlobal.uni = { showToast: (options: { title: string }) => messages.push(options.title) } as unknown as typeof uni
    const state = useMatchesPage()
    state.selectedRoundNumber.value = 1
    state.openMatchTechStats(match(1))
    expect(state.selectedTechStatsMatch.value).toBeNull()
    expect(messages).toEqual(['这场比赛暂时还没有技术统计'])
    state.openMatchTechStats(match(1, 1, { home_corners: 3, away_corners: 0 }))
    expect(state.selectedTechStatsMatch.value?.match_id).toBe(1)
    expect(state.selectedRoundNumber.value).toBe(1)
    state.closeMatchTechStats()
    expect(state.selectedTechStatsMatch.value).toBeNull()
    expect(state.selectedRoundNumber.value).toBe(1)
  })
})
