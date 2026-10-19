import { describe, expect, test } from 'bun:test'
import { nextTick } from 'vue'
import type { SavedLineup, SquadResponse } from '../../types/lineup'
import { assignPlayer, emptyLineup } from './helpers'
import { ApiRequestError } from '../../utils/apiError'
import { readDraft, writeDraft } from './lineupDraft'
import { useLineupEditor } from './useLineupEditor'
import { useLineupPage } from './useLineupPage'

const squad: SquadResponse = {
  team: { team_id: 1, team_name: '成都蓉城', avatar_url: null },
  active: [
    { player_id: 1, name: 'A', shirt_number: 1, position: 'GK', avatar_url: null, last_season: 2026 },
    { player_id: 2, name: 'B', shirt_number: 2, position: 'DF', avatar_url: null, last_season: 2026 },
  ],
  former: [],
}
const saved: SavedLineup = { formation: '4-4-2', slots: emptyLineup('4-4-2'), share_code: 'ABCDEFGH', updated_at: '2026-10-19T00:00:00Z' }

function memoryStorage() {
  const map = new Map<string, unknown>()
  return {
    getStorageSync: (k: string) => JSON.parse(JSON.stringify(map.get(k) ?? null)),
    setStorageSync: (k: string, v: unknown) => { map.set(k, JSON.parse(JSON.stringify(v))) },
    removeStorageSync: (k: string) => { map.delete(k) },
  }
}

describe('useLineupEditor', () => {
  test('starts clean; changes mark dirty; markSaved clears it', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', emptyLineup('4-3-3'))
    expect(editor.dirty.value).toBe(false)
    editor.pick(0, 1)
    expect(editor.dirty.value).toBe(true)
    expect(editor.filled.value).toBe(1)
    editor.markSaved()
    expect(editor.dirty.value).toBe(false)
  })

  test('load can start dirty (restored drafts)', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', emptyLineup('4-3-3'), true)
    expect(editor.dirty.value).toBe(true)
  })

  test('switching formation keeps players and resets coordinates; reset only resets coordinates', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', assignPlayer(emptyLineup('4-3-3'), 0, 1))
    editor.setFormation('4-4-2', () => 'GK')
    expect(editor.formation.value).toBe('4-4-2')
    expect(editor.slots.value.filter((s) => s.player_id === 1).length).toBe(1)
    editor.applyDrag(5, { kind: 'move', x: 0.2, y: 0.2 })
    expect(editor.slots.value.find((s) => s.slot === 5)!.x).toBe(0.2)
    editor.reset()
    expect(editor.slots.value.find((s) => s.slot === 5)!.x === 0.2).toBe(false)
    expect(editor.slots.value.filter((s) => s.player_id === 1).length).toBe(1)
  })

  test('selecting the same formation again changes nothing and stays clean', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', emptyLineup('4-3-3'))
    editor.setFormation('4-3-3', () => null)
    expect(editor.dirty.value).toBe(false)
  })

  test('a tap does not make the lineup dirty', () => {
    const editor = useLineupEditor()
    editor.load('4-3-3', emptyLineup('4-3-3'))
    editor.applyDrag(1, { kind: 'tap' })
    expect(editor.dirty.value).toBe(false)
  })
})

describe('useLineupPage private drafts and publication', () => {
  const fullSlots = emptyLineup('4-3-3').map((s, i) => ({ ...s, player_id: i + 1 }))
  const publicSnapshot: SavedLineup = { ...saved, formation: '4-3-3', slots: fullSlots }
  const fullSquad: SquadResponse = { ...squad, active: fullSlots.map(s => ({ ...squad.active[0], player_id: s.player_id, position: 'MF' })) }
  function build(overrides: Partial<NonNullable<Parameters<typeof useLineupPage>[0]>> = {}, storage = memoryStorage()) {
    const calls: string[] = []
    const page = useLineupPage({
      getSquad: async () => fullSquad,
      getMyLineupEditor: async () => ({ lineup: null, draft: null }),
      saveMyLineupDraft: async (_teamId, payload) => { calls.push('draft'); return { draft: { ...payload, updated_at: '2026-10-19T01:00:00Z' } } },
      publishMyLineup: async (_teamId, payload) => { calls.push('publish'); return { lineup: { ...publicSnapshot, ...payload }, draft: { ...payload, updated_at: '2026-10-19T01:00:00Z' } } },
      isLoggedIn: () => true, userId: () => 'A', storage,
      now: () => Date.parse('2026-10-19T02:00:00Z'), draftDelayMs: 5,
      ...overrides,
    })
    return { page, storage, calls }
  }
  test('loads cloud private draft before public snapshot without marking it published', async () => {
    const draft = { formation: '4-4-2', slots: emptyLineup('4-4-2'), updated_at: saved.updated_at }
    const { page } = build({ getMyLineupEditor: async () => ({ lineup: publicSnapshot, draft }) })
    await page.load(1)
    expect(page.editor.formation.value).toBe('4-4-2')
    expect(page.editor.dirty.value).toBe(false)
    expect(page.hasUnpublishedChanges.value).toBe(true)
    expect(page.saveState.value).toBe('draft')
    expect(page.savedLineup.value?.share_code).toBe(publicSnapshot.share_code)
  })
  test('local unsynchronized draft wins only when newer than cloud', async () => {
    const draft = { ...publicSnapshot, formation: '4-4-2', slots: emptyLineup('4-4-2') }
    const { page, storage } = build({ getMyLineupEditor: async () => ({ lineup: publicSnapshot, draft }) })
    writeDraft({ teamId: 1, formation: '3-5-2', slots: emptyLineup('3-5-2'), savedAt: Date.parse(saved.updated_at) - 1 }, storage, 'user:A')
    await page.load(1)
    expect(page.editor.formation.value).toBe('4-4-2')
    writeDraft({ teamId: 1, formation: '3-5-2', slots: emptyLineup('3-5-2'), savedAt: Date.parse(saved.updated_at) + 1 }, storage, 'user:A')
    await page.load(1)
    expect(page.editor.formation.value).toBe('3-5-2')
    expect(page.editor.dirty.value).toBe(true)
  })
  test('draft save permits incomplete eleven and leaves published code and content unchanged', async () => {
    const { page, calls } = build({ getMyLineupEditor: async () => ({ lineup: publicSnapshot, draft: null }) })
    await page.load(1); page.editor.pick(0, null)
    expect(await page.saveDraft()).toBe('saved')
    expect(calls).toEqual(['draft'])
    expect(page.savedLineup.value).toEqual(publicSnapshot)
    expect(page.editor.dirty.value).toBe(false)
    expect(page.hasUnpublishedChanges.value).toBe(true)
    expect(page.saveState.value).toBe('draft')
    expect(await page.publish()).toBe('incomplete')
    expect(calls).toEqual(['draft'])
  })
  test('explicit publication updates snapshot and cloud draft; code is retained', async () => {
    const { page, calls } = build()
    await page.load(1)
    expect(await page.publish()).toBe('saved')
    expect(calls).toEqual(['publish'])
    expect(page.hasUnpublishedChanges.value).toBe(false)
    expect(page.saveState.value).toBe('published')
    expect(page.savedLineup.value?.share_code).toBe(saved.share_code)
  })
  test('reverting an edit to published content is shareable even when editor is dirty', async () => {
    const { page } = build({ getMyLineupEditor: async () => ({ lineup: publicSnapshot, draft: null }) })
    await page.load(1); page.editor.pick(0, null); page.editor.pick(0, 1)
    expect(page.editor.dirty.value).toBe(true)
    expect(page.hasUnpublishedChanges.value).toBe(false)
  })
  test('saving an unchanged cloud draft avoids duplicate requests', async () => {
    const { page, calls } = build()
    await page.load(1); await page.saveDraft(); await page.saveDraft()
    expect(calls).toEqual(['draft'])
    expect(page.savedLineup.value).toBeNull()
  })
  test('new edits during either request retain dirty state and local draft', async () => {
    for (const kind of ['draft', 'publish'] as const) {
      let finish: () => void = () => {}
      const { page, storage } = build({
        saveMyLineupDraft: async (_id, payload) => { await new Promise<void>(resolve => { finish = resolve }); return { draft: { ...payload, updated_at: saved.updated_at } } },
        publishMyLineup: async (_id, payload) => { await new Promise<void>(resolve => { finish = resolve }); return { lineup: { ...publicSnapshot, ...payload }, draft: { ...payload, updated_at: saved.updated_at } } },
      })
      await page.load(1)
      const pending = kind === 'draft' ? page.saveDraft() : page.publish()
      page.editor.pick(0, null); finish()
      expect(await pending).toBe('saved')
      expect(page.editor.dirty.value).toBe(true)
      expect(readDraft(1, storage, 'user:A')?.slots[0].player_id).toBeNull()
      expect(page.hasUnpublishedChanges.value).toBe(true)
      page.dispose()
    }
  })
  test('team switch ignores old responses and does not clear the new request lock', async () => {
    let finishA: () => void = () => {}, finishB: () => void = () => {}
    const { page } = build({ saveMyLineupDraft: async (id, payload) => { await new Promise<void>(resolve => { if (id === 1) finishA = resolve; else finishB = resolve }); return { draft: { ...payload, updated_at: saved.updated_at } } } })
    await page.load(1); const a = page.saveDraft(); await page.load(2); const b = page.saveDraft()
    finishA(); expect(await a).toBe('cancelled')
    expect(page.savingDraft.value).toBe(true); expect(page.cloudDraft.value).toBeNull()
    finishB(); await b; expect(page.saving.value).toBe(false)
  })
  test('prior account response cannot update metadata and in-memory prior edits cannot save to new account', async () => {
    let revision = 1, user = 'A', finish: () => void = () => {}
    const { page, storage } = build({ userId: () => user, sessionRevision: () => revision, saveMyLineupDraft: async (_id, payload) => { await new Promise<void>(resolve => { finish = resolve }); return { draft: { ...payload, updated_at: saved.updated_at } } } })
    await page.load(1); page.editor.pick(0, null); const pending = page.saveDraft()
    revision++; user = 'B'; finish()
    expect(await pending).toBe('cancelled'); expect(page.cloudDraft.value).toBeNull(); expect(page.saving.value).toBe(false)
    expect(await page.saveDraft()).toBe('cancelled')
    await page.load(1)
    expect(page.editor.filled.value).toBe(11)
    expect(readDraft(1, storage, 'user:B')).toBeNull()
    expect(readDraft(1, storage, 'user:A')?.slots[0].player_id).toBeNull()
  })
  test('guest data is imported once only after a deliberate save intent and explicit resume', async () => {
    let loggedIn = false, user: string | null = null, revision = 1
    const { page, storage } = build({ isLoggedIn: () => loggedIn, userId: () => user, sessionRevision: () => revision })
    await page.load(1); page.editor.pick(0, null)
    expect(await page.saveDraft()).toBe('need-login')
    loggedIn = true; user = 'A'; revision++
    await page.load(1, { restoreGuestDraft: true })
    expect(page.editor.filled.value).toBe(10)
    expect(readDraft(1, storage)).toBeNull()
    expect(await page.saveDraft()).toBe('saved')
    user = 'B'; revision++
    await page.load(1, { restoreGuestDraft: true })
    expect(page.editor.filled.value).toBe(11)
  })
  test('ordinary login and old unscoped cache never import guest or another account', async () => {
    const { page, storage } = build()
    writeDraft({ teamId: 1, formation: '4-4-2', slots: emptyLineup('4-4-2'), savedAt: Date.now() }, storage)
    storage.setStorageSync('fi-lineup-draft-1', { teamId: 1, formation: '4-4-2', slots: emptyLineup('4-4-2'), savedAt: Date.now() })
    await page.load(1); expect(page.editor.filled.value).toBe(11)
    await page.load(1, { restoreGuestDraft: true }); expect(page.editor.filled.value).toBe(11)
  })
  test('unresolved authenticated identity neither reads nor writes shared account cache', async () => {
    const { page, storage } = build({ userId: () => null })
    writeDraft({ teamId: 1, formation: '4-4-2', slots: emptyLineup('4-4-2'), savedAt: Date.now() }, storage)
    await page.load(1); page.editor.pick(0, null); await nextTick(); page.dispose()
    expect(readDraft(1, storage)?.formation).toBe('4-4-2')
    expect(readDraft(1, storage, 'user:null')).toBeNull()
  })
  test('401 saves authenticated draft under its original owner before invalidating session', async () => {
    let revision = 1, loggedIn = true
    const { page, storage } = build({ sessionRevision: () => revision, isLoggedIn: () => loggedIn, invalidateSession: () => { revision++; loggedIn = false }, saveMyLineupDraft: async () => { throw new ApiRequestError('expired', 401) } })
    await page.load(1); page.editor.pick(0, null)
    expect(await page.saveDraft()).toBe('need-login')
    expect(readDraft(1, storage, 'user:A')?.slots[0].player_id).toBeNull()
    expect(readDraft(1, storage)).toBeNull()
    expect(page.saving.value).toBe(false)
  })
  test('failed saves retain local edits and never mutate published snapshot', async () => {
    const { page, storage } = build({ getMyLineupEditor: async () => ({ lineup: publicSnapshot, draft: null }), saveMyLineupDraft: async () => { throw new Error('network') } })
    await page.load(1); page.editor.pick(0, null)
    expect(await page.saveDraft()).toBe('failed')
    expect(page.saveState.value).toBe('error')
    expect(page.savedLineup.value).toEqual(publicSnapshot)
    expect(readDraft(1, storage, 'user:A') === null).toBe(false)
  })
  test('out of order loads and disposal cannot publish obsolete data', async () => {
    const gates: Record<number, () => void> = {}
    const { page } = build({ getSquad: async id => { await new Promise<void>(resolve => { gates[id] = resolve }); return { ...fullSquad, team: { ...fullSquad.team, team_id: id } } } })
    const a = page.load(1), b = page.load(2); gates[2](); await b; gates[1](); await a
    expect(page.squad.value?.team.team_id).toBe(2)
    const c = page.load(3); page.dispose(); gates[3](); await c
    expect(page.squad.value).toBeNull()
  })
  test('missing squad players are cleared from editor with notice without mutating public snapshot', async () => {
    const stale = { ...publicSnapshot, slots: assignPlayer(publicSnapshot.slots, 0, 999) }
    const { page } = build({ getMyLineupEditor: async () => ({ lineup: stale, draft: null }) })
    await page.load(1)
    expect(page.editor.slots.value[0].player_id).toBeNull()
    expect(page.notice.value === '').toBe(false)
    expect(page.savedLineup.value).toEqual(stale)
    expect(page.hasUnpublishedChanges.value).toBe(true)
  })
  test('first automatically filled eleven is clean and may be saved to the private cloud', async () => {
    const { page, calls } = build()
    await page.load(1)
    expect(page.editor.filled.value).toBe(11)
    expect(page.editor.dirty.value).toBe(false)
    expect(page.saveState.value).toBe('idle')
    expect(await page.saveDraft()).toBe('saved')
    expect(calls).toEqual(['draft'])
    expect(page.savedLineup.value).toBeNull()
  })
  test('switching teams flushes the prior dirty lineup before resetting editor', async () => {
    const { page, storage } = build()
    await page.load(1); page.editor.pick(0, null)
    await page.load(2)
    expect(readDraft(1, storage, 'user:A')?.slots[0].player_id).toBeNull()
    expect(page.editor.filled.value).toBe(11)
    expect(readDraft(2, storage, 'user:A')).toBeNull()
    page.dispose()
  })
  test('dispose flushes final changes and pending watchers cannot recreate timers or drafts', async () => {
    const { page, storage } = build()
    await page.load(1); page.editor.pick(0, null); page.dispose()
    expect(readDraft(1, storage, 'user:A')?.slots[0].player_id).toBeNull()
    storage.removeStorageSync('fi-lineup-draft-v2-user%3AA-1')
    await nextTick(); await new Promise(resolve => setTimeout(resolve, 15))
    expect(readDraft(1, storage, 'user:A')).toBeNull()
  })
  test('fill starting eleven only fills empty places and does nothing before loading', async () => {
    const { page } = build()
    page.fillStartingEleven(); expect(page.editor.filled.value).toBe(0)
    await page.load(1); page.editor.pick(0, null); const retained = page.editor.slots.value[1].player_id
    page.fillStartingEleven()
    expect(page.editor.filled.value).toBe(11); expect(page.editor.slots.value[1].player_id).toBe(retained)
    expect(page.editor.dirty.value).toBe(true)
  })
  test('optional editor 401 still loads squad anonymously', async () => {
    let revision = 1, loggedIn = true
    const { page } = build({ sessionRevision: () => revision, isLoggedIn: () => loggedIn, invalidateSession: expected => { if (expected === revision) { revision++; loggedIn = false } }, getMyLineupEditor: async () => { throw new ApiRequestError('expired', 401) } })
    await page.load(1)
    expect(page.squad.value?.team.team_id).toBe(1); expect(page.errorMessage.value).toBe('')
    expect(page.loading.value).toBe(false); expect(revision).toBe(2)
  })
  test('late old-account 401 cannot invalidate a newer session', async () => {
    let revision = 1, invalidations = 0, reject: (error: Error) => void = () => {}
    const { page } = build({ sessionRevision: () => revision, invalidateSession: () => { invalidations++; revision++ }, saveMyLineupDraft: async () => new Promise((_resolve, fail) => { reject = fail }) })
    await page.load(1); const pending = page.saveDraft(); revision++
    reject(new ApiRequestError('expired', 401))
    expect(await pending).toBe('cancelled'); expect(invalidations).toBe(0); expect(revision).toBe(2)
  })

})
