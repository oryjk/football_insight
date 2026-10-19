import { describe, expect, test } from 'bun:test'
import type { SavedLineup, SquadResponse } from '../../types/lineup'
import { assignPlayer, emptyLineup } from './helpers'
import { readDraft } from './lineupDraft'
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
    getStorageSync: (k: string) => map.get(k) ?? '',
    setStorageSync: (k: string, v: unknown) => { map.set(k, v) },
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

describe('useLineupPage', () => {
  function build(overrides: Partial<NonNullable<Parameters<typeof useLineupPage>[0]>> = {}) {
    const storage = memoryStorage()
    const calls: string[] = []
    const page = useLineupPage({
      getSquad: async () => squad,
      getMyLineup: async () => ({ lineup: null }),
      saveMyLineup: async (_teamId, payload) => {
        calls.push('save')
        return { lineup: { ...saved, formation: payload.formation, slots: payload.slots } }
      },
      isLoggedIn: () => true,
      storage,
      now: () => 1,
      draftDelayMs: 5,
      ...overrides,
    })
    return { page, storage, calls }
  }

  test('loads the saved lineup when there is one, otherwise an empty 4-3-3', async () => {
    const { page } = build({ getMyLineup: async () => ({ lineup: saved }) })
    await page.load(1)
    expect(page.editor.formation.value).toBe('4-4-2')
    const empty = build()
    await empty.page.load(1)
    expect(empty.page.editor.formation.value).toBe('4-3-3')
    expect(empty.page.editor.filled.value).toBe(0)
  })

  test('anonymous visitors can still load and edit without calling my-lineup', async () => {
    let myCalls = 0
    const { page } = build({ isLoggedIn: () => false, getMyLineup: async () => { myCalls += 1; return { lineup: null } } })
    await page.load(1)
    expect(myCalls).toBe(0)
    expect(page.errorMessage.value).toBe('')
  })

  test('saving while logged out stores a draft and reports need-login', async () => {
    const { page, storage, calls } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.editor.pick(0, 1)
    expect(await page.save()).toBe('need-login')
    expect(calls).toEqual([])
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
  })

  test('a stored draft wins over the saved lineup, is sanitized, and is saved once then cleared', async () => {
    const { page, storage, calls } = build({ getMyLineup: async () => ({ lineup: saved }) })
    const bad = assignPlayer(assignPlayer(emptyLineup('3-5-2'), 0, 1), 1, 999)
    storage.setStorageSync('fi-lineup-draft-1', { teamId: 1, formation: '3-5-2', slots: bad, savedAt: 1 })
    await page.load(1)
    expect(page.editor.formation.value).toBe('3-5-2')
    expect(page.editor.slots.value[0].player_id).toBe(1)
    expect(page.editor.slots.value[1].player_id).toBeNull()
    expect(page.editor.dirty.value).toBe(true)
    expect(await page.save()).toBe('saved')
    expect(await page.save()).toBe('saved')
    expect(calls).toEqual(['save'])
    expect(readDraft(1, storage)).toBeNull()
  })

  test('save failure keeps the draft and reports failed', async () => {
    const { page, storage } = build({ saveMyLineup: async () => { throw new Error('offline') } })
    await page.load(1)
    page.editor.pick(0, 1)
    expect(await page.save()).toBe('failed')
    expect(page.saveState.value).toBe('error')
    expect(readDraft(1, storage) === null).toBe(false)
  })

  test('loading failure surfaces an error message', async () => {
    const { page } = build({ getSquad: async () => { throw new Error('boom') } })
    await page.load(1)
    expect(page.errorMessage.value).toBe('boom')
    expect(page.loading.value).toBe(false)
  })

  test('save state follows the editor: dirty after an edit, saved after a successful save', async () => {
    const { page } = build()
    await page.load(1)
    expect(page.saveState.value).toBe('idle')
    page.editor.pick(0, 1)
    expect(page.saveState.value).toBe('dirty')
    await page.save()
    expect(page.saveState.value).toBe('saved')
  })

  test('edits are written to the draft after the debounce even when logged out, and dispose stops the timer', async () => {
    const { page, storage } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.editor.pick(0, 1)
    await new Promise((resolve) => setTimeout(resolve, 30))
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
    page.editor.pick(1, 2)
    page.dispose()
    await new Promise((resolve) => setTimeout(resolve, 30))
    expect(readDraft(1, storage)?.slots[1].player_id).toBeNull()
  })

  test('loading another team does not carry the previous team draft or state over', async () => {
    const { page } = build()
    await page.load(1)
    page.editor.pick(0, 1)
    await page.load(2)
    expect(page.teamId.value).toBe(2)
    expect(page.editor.filled.value).toBe(0)
    expect(page.editor.dirty.value).toBe(false)
  })
})
