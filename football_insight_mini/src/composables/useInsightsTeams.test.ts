import { describe, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { AppSession, SessionStatus } from '../stores/appSession'
import type { CurrentUser } from '../types/auth'
import type { SupportProfile, SupportTeam } from '../types/support'
import { useInsightsTeams } from './useInsightsTeams'

const TEAM_KEY = 'fi-insights-selected-team'
const clubs: SupportTeam[] = [
  { team_id: 1, team_name: '北京国安', rank_no: 2, avatar_storage_url: null },
  { team_id: 9, team_name: '成都蓉城', rank_no: 1, avatar_storage_url: null },
  { team_id: 3, team_name: '无排名球队', rank_no: null, avatar_storage_url: null },
]
function user(id: string): CurrentUser {
  return { id, display_name: id, account_identifier: id, avatar_url: null,
    has_wechat_binding: false, membership_tier: 'none', membership_benefits_enabled: false, created_at: '' }
}
function auth(authenticated = false) {
  const currentUser = ref<CurrentUser | null>(authenticated ? user('A') : null)
  const status = ref<SessionStatus>(authenticated ? 'authenticated' : 'guest'), revision = ref(0)
  const session = { currentUser, status, revision } as unknown as AppSession
  return { currentUser, status, revision, session }
}
function storage(value?: unknown) {
  const values = new Map<string, unknown>(value === undefined ? [] : [[TEAM_KEY, value]])
  return { values, getStorageSync: (key: string) => values.get(key), setStorageSync: (key: string, next: unknown) => { values.set(key, next) } }
}
function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (error: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}
const favorite = (id: number): SupportProfile => ({ favorite_team: clubs.find(team => team.team_id === id) ?? null, next_match: null })
async function flush() { await nextTick(); await Promise.resolve(); await Promise.resolve() }

describe('public insights team selection', () => {
  test('guest teams load once, include unranked clubs, and do not request a private favorite', async () => {
    let calls = 0, profiles = 0
    const list = deferred<SupportTeam[]>()
    const state = useInsightsTeams(auth().session, { storage: storage(), list: () => { calls++; return list.promise }, profile: async () => { profiles++; return favorite(1) } })
    const first = state.load(), second = state.load()
    expect(first).toBe(second); expect(state.loading.value).toBe(true)
    list.resolve(clubs); await first
    expect(calls).toBe(1); expect(profiles).toBe(0)
    expect(state.teams.value.map(team => team.team_id)).toEqual([1, 9, 3])
    expect(state.selectedTeamId.value).toBe(9); expect(state.loading.value).toBe(false)
    await state.load(); expect(calls).toBe(1)
    state.dispose()
  })

  test('an account without membership sees public teams before its slow favorite is available', async () => {
    const profile = deferred<SupportProfile>(), session = auth(true)
    const state = useInsightsTeams(session.session, { storage: storage(), list: async () => clubs, profile: () => profile.promise })
    await state.load()
    expect(state.loading.value).toBe(false); expect(state.teams.value.length).toBe(3)
    expect(state.selectedTeamId.value).toBe(9)
    profile.resolve(favorite(1)); await flush()
    expect(state.selectedTeamId.value).toBe(1)
    state.dispose()
  })

  test('explicit selection wins over a late favorite and is stored for the next visit', async () => {
    const profile = deferred<SupportProfile>(), saved = storage()
    const state = useInsightsTeams(auth(true).session, { storage: saved, list: async () => clubs, profile: () => profile.promise })
    await state.load(); state.selectTeam(3)
    expect(saved.values.get(TEAM_KEY)).toBe(3)
    profile.resolve(favorite(1)); await flush()
    expect(state.selectedTeamId.value).toBe(3)
    state.dispose()
    let profiles = 0
    const restored = useInsightsTeams(auth(true).session, { storage: saved, list: async () => clubs, profile: async () => { profiles++; return favorite(1) } })
    await restored.load()
    expect(restored.selectedTeamId.value).toBe(3); expect(profiles).toBe(0)
    restored.dispose()
  })

  test('a late previous-account favorite cannot replace the new account selection', async () => {
    const session = auth(true), old = deferred<SupportProfile>(), current = deferred<SupportProfile>()
    let profiles = 0
    const state = useInsightsTeams(session.session, { storage: storage(), list: async () => clubs, profile: () => ++profiles === 1 ? old.promise : current.promise })
    await state.load()
    session.revision.value++; session.currentUser.value = user('B'); await flush()
    expect(profiles).toBe(2)
    current.resolve(favorite(3)); await flush()
    expect(state.selectedTeamId.value).toBe(3)
    old.resolve(favorite(1)); await flush()
    expect(state.selectedTeamId.value).toBe(3); expect(state.teams.value.length).toBe(3)
    state.dispose()
  })

  test('explicit public choice survives logout and login to a different account', async () => {
    const session = auth(true), old = deferred<SupportProfile>()
    let profiles = 0
    const state = useInsightsTeams(session.session, { storage: storage(), list: async () => clubs, profile: () => { profiles++; return old.promise } })
    await state.load(); state.selectTeam(3)
    session.currentUser.value = null; session.status.value = 'guest'; session.revision.value++; await flush()
    old.resolve(favorite(1)); await flush()
    expect(state.selectedTeamId.value).toBe(3); expect(state.teams.value.length).toBe(3)
    session.currentUser.value = user('B'); session.status.value = 'authenticated'; session.revision.value++; await flush()
    expect(state.selectedTeamId.value).toBe(3); expect(profiles).toBe(1)
    state.dispose()
  })

  test('a revision change invalidates a late favorite even before replacement user publication', async () => {
    const session = auth(true), profile = deferred<SupportProfile>()
    const state = useInsightsTeams(session.session, { storage: storage(), list: async () => clubs, profile: () => profile.promise })
    await state.load(); session.revision.value++
    profile.resolve(favorite(1)); await flush()
    expect(state.selectedTeamId.value).toBe(9)
    state.dispose()
  })

  test('invalid remembered teams are ignored and unknown explicit choices are not stored', async () => {
    for (const remembered of [999, '3', null]) {
      const saved = storage(remembered)
      const state = useInsightsTeams(auth().session, { storage: saved, list: async () => clubs })
      await state.load(); expect(state.selectedTeamId.value).toBe(9)
      state.selectTeam(999)
      expect(state.selectedTeamId.value).toBe(9); expect(saved.values.get(TEAM_KEY)).toBe(remembered)
      state.dispose()
    }
  })

  test('storage failures do not block public teams or an explicit in-memory choice', async () => {
    const state = useInsightsTeams(auth().session, {
      storage: { getStorageSync: () => { throw Error('unavailable') }, setStorageSync: () => { throw Error('unavailable') } },
      list: async () => clubs,
    })
    await state.load(); state.selectTeam(3)
    expect(state.selectedTeamId.value).toBe(3); expect(state.error.value).toBe('')
    state.dispose()
  })

  test('failed public reads can retry, and disposal discards a late list', async () => {
    let calls = 0
    const state = useInsightsTeams(auth().session, { storage: storage(), list: async () => { if (++calls === 1) throw Error('offline'); return clubs } })
    await state.load(); expect(state.error.value).toBe('offline'); expect(state.loading.value).toBe(false)
    await state.load(); expect(state.teams.value.length).toBe(3); expect(state.error.value).toBe('')
    state.dispose()
    const gate = deferred<SupportTeam[]>()
    const abandoned = useInsightsTeams(auth().session, { storage: storage(), list: () => gate.promise })
    const pending = abandoned.load(); abandoned.dispose(); gate.resolve(clubs); await pending
    expect(abandoned.teams.value).toEqual([]); expect(abandoned.selectedTeamId.value).toBe(null)
  })
})
