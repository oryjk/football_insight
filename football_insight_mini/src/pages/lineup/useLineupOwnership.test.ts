import { afterEach, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import type { PublicLineup, SavedLineup } from '../../types/lineup'
import type { SessionStatus } from '../../stores/appSession'
import { useLineupOwnership } from './useLineupOwnership'

function user(id = 'A'): CurrentUser {
  return { id, display_name: '同名球迷', account_identifier: id, avatar_url: null,
    has_wechat_binding: false, membership_tier: 'none', created_at: '' }
}
function session(authenticated = true) {
  return { currentUser: ref<CurrentUser | null>(authenticated ? user() : null),
    hasAccessToken: ref(authenticated), revision: ref(0), status: ref<SessionStatus>(authenticated ? 'authenticated' : 'guest') }
}
function published(code = 'OwnCode1'): SavedLineup {
  return { share_code: code, formation: '4-3-3', slots: [], updated_at: '' }
}
function lineup(code = 'OwnCode1', teamId = 77680): PublicLineup {
  return { ...published(code), author: { display_name: '同名球迷', avatar_url: null },
    created_at: '', likes_count: 0, liked_by_me: false, players: [],
    team: { team_id: teamId, team_name: '成都蓉城', avatar_url: null } }
}
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(yes => { resolve = yes })
  return { promise, resolve }
}
const cleanups: Array<() => void> = []
afterEach(() => { while (cleanups.length) cleanups.pop()!() })
async function flush() { await nextTick(); await Promise.resolve(); await nextTick() }

test('matching the authenticated private published code identifies the owner', async () => {
  const ids: number[] = []
  const state = useLineupOwnership(ref(lineup()), session(), async id => { ids.push(id); return { lineup: published() } })
  cleanups.push(state.dispose)
  expect(state.loading.value).toBe(true)
  await flush()
  expect(ids).toEqual([77680]); expect(state.isOwner.value).toBe(true); expect(state.loading.value).toBe(false)
})

test('matching display names do not identify someone else’s lineup as owned', async () => {
  const state = useLineupOwnership(ref(lineup('Other001')), session(), async () => ({ lineup: published() }))
  cleanups.push(state.dispose)
  await flush()
  expect(state.isOwner.value).toBe(false)
})

test('guests make no private request; startup login rechecks ownership on the open page', async () => {
  const auth = session(false); let calls = 0
  const state = useLineupOwnership(ref(lineup()), auth, async () => { calls++; return { lineup: published() } })
  cleanups.push(state.dispose)
  await flush(); expect(calls).toBe(0); expect(state.isOwner.value).toBe(false)
  auth.currentUser.value = user(); auth.hasAccessToken.value = true; auth.status.value = 'authenticated'; auth.revision.value++
  await flush(); expect(calls).toBe(1); expect(state.isOwner.value).toBe(true)
})

test('logout invalidates a pending ownership response before it can restore Edit', async () => {
  const auth = session(), pending = deferred<{ lineup: SavedLineup | null }>()
  const state = useLineupOwnership(ref(lineup()), auth, () => pending.promise)
  cleanups.push(state.dispose)
  auth.currentUser.value = null; auth.hasAccessToken.value = false; auth.status.value = 'guest'; auth.revision.value++
  pending.resolve({ lineup: published() })
  await flush()
  expect(state.isOwner.value).toBe(false); expect(state.loading.value).toBe(false)
})

test('an old account response cannot mark a new account’s view as owned', async () => {
  const auth = session(), pending = deferred<{ lineup: SavedLineup | null }>(); let calls = 0
  const state = useLineupOwnership(ref(lineup()), auth, async () => ++calls === 1 ? pending.promise : { lineup: published('Other001') })
  cleanups.push(state.dispose)
  auth.currentUser.value = user('B'); auth.revision.value++
  await flush(); pending.resolve({ lineup: published() }); await flush()
  expect(calls).toBe(2); expect(state.isOwner.value).toBe(false)
})

test('switching public lineup/team invalidates the previous owner lookup', async () => {
  const detail = ref<PublicLineup | null>(lineup()), pending = deferred<{ lineup: SavedLineup | null }>()
  const ids: number[] = []
  const state = useLineupOwnership(detail, session(), async id => { ids.push(id); return id === 77680 ? pending.promise : { lineup: null } })
  cleanups.push(state.dispose)
  detail.value = lineup('Other001', 2)
  await flush(); pending.resolve({ lineup: published() }); await flush()
  expect(ids).toEqual([77680, 2]); expect(state.isOwner.value).toBe(false)
})

test('a private read failure keeps the public lineup available without claiming ownership', async () => {
  const detail = ref<PublicLineup | null>(lineup())
  const state = useLineupOwnership(detail, session(), async () => { throw new Error('offline') })
  cleanups.push(state.dispose)
  await flush()
  expect(detail.value?.share_code).toBe('OwnCode1'); expect(state.isOwner.value).toBe(false); expect(state.loading.value).toBe(false)
})

test('unmount cancels late ownership publication', async () => {
  const pending = deferred<{ lineup: SavedLineup | null }>()
  const state = useLineupOwnership(ref(lineup()), session(), () => pending.promise)
  state.dispose(); pending.resolve({ lineup: published() }); await flush()
  expect(state.isOwner.value).toBe(false); expect(state.loading.value).toBe(false)
})
