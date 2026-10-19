import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { AppSession } from '../../stores/appSession'
import type { CurrentUser } from '../../types/auth'
import type { TeamInsightsResponse } from '../../types/insight'
import { useInsightsData } from './useInsightsData'

function auth() {
  const currentUser = ref<CurrentUser | null>({ id: 'A' } as CurrentUser), revision = ref(0)
  const session = { currentUser, revision, ensureSessionReady: async () => currentUser.value } as unknown as AppSession
  return { currentUser, revision, session }
}
function response(id: number): TeamInsightsResponse {
  return { view_kind: 'live', round_number: null, current_season: 2026,
    teams: [{ team_id: id, team_name: `球队${id}`, rank_no: 1, avatar_storage_url: null }], insights: [] }
}
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(yes => { resolve = yes })
  return { promise, resolve }
}

describe('member insights with a shared public team selection', () => {
  test('a late private roster cannot default or replace a public choice outside that roster', async () => {
    const selectedTeamId = ref<number | null>(9), gate = deferred<TeamInsightsResponse>()
    const data = useInsightsData({ session: auth().session, selectedTeamId, isMembershipLocked: () => false, getLiveTeamInsights: () => gate.promise })
    expect(data.selectedTeamId).toBe(selectedTeamId)
    const pending = data.loadInsights(); selectedTeamId.value = 3
    gate.resolve(response(1)); await pending
    expect(data.teams.value[0]?.team_id).toBe(1); expect(selectedTeamId.value).toBe(3)
    data.resetSessionScopedState()
    expect(data.teams.value).toEqual([]); expect(selectedTeamId.value).toBe(3)
  })

  test('private reset preserves a shared null selection and never initializes it from member teams', async () => {
    const selectedTeamId = ref<number | null>(null)
    const data = useInsightsData({ session: auth().session, selectedTeamId, isMembershipLocked: () => false, getLiveTeamInsights: async () => response(1) })
    await data.loadInsights(); expect(selectedTeamId.value).toBe(null)
    data.goalsForPlayerExpanded.value = true; data.resetSessionScopedState()
    expect(selectedTeamId.value).toBe(null); expect(data.goalsForPlayerExpanded.value).toBe(false)
  })

  test('cross-account late private data cannot restore caches or clear the current loading and shared choice', async () => {
    const session = auth(), selectedTeamId = ref<number | null>(3)
    const old = deferred<TeamInsightsResponse>(), current = deferred<TeamInsightsResponse>(); let calls = 0
    const data = useInsightsData({ session: session.session, selectedTeamId, isMembershipLocked: () => false,
      getLiveTeamInsights: () => ++calls === 1 ? old.promise : current.promise })
    const stale = data.loadInsights()
    session.revision.value++; session.currentUser.value = { id: 'B' } as CurrentUser
    data.resetSessionScopedState()
    const latest = data.loadInsights()
    old.resolve(response(1)); await stale
    expect(data.teams.value).toEqual([]); expect(data.loading.value).toBe(true); expect(selectedTeamId.value).toBe(3)
    current.resolve(response(9)); await latest
    expect(data.teams.value[0]?.team_id).toBe(9); expect(data.loading.value).toBe(false); expect(selectedTeamId.value).toBe(3)
    session.currentUser.value = null; session.revision.value++; data.resetSessionScopedState()
    expect(selectedTeamId.value).toBe(3); expect(data.teams.value).toEqual([])
  })

  test('membership locking prevents private requests while retaining the shared public choice', async () => {
    const selectedTeamId = ref<number | null>(3); let calls = 0
    const data = useInsightsData({ session: auth().session, selectedTeamId, isMembershipLocked: () => true,
      getLiveTeamInsights: async () => { calls++; return response(1) } })
    await data.loadInsights(); data.resetSessionScopedState()
    expect(calls).toBe(0); expect(selectedTeamId.value).toBe(3)
  })
})
