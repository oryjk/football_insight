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

  test('loads the saved lineup when there is one, otherwise a default 4-3-3 starting eleven', async () => {
    const { page } = build({ getMyLineup: async () => ({ lineup: saved }) })
    await page.load(1)
    expect(page.editor.formation.value).toBe('4-4-2')
    const first = build()
    await first.page.load(1)
    expect(first.page.editor.formation.value).toBe('4-3-3')
    // 测试名单只有 1 名门将和 1 名后卫，初始首发就只能填这两个
    expect(first.page.editor.filled.value).toBe(2)
    expect(first.page.editor.dirty.value).toBe(false)
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

  test('edits are written to the draft after the debounce even when logged out', async () => {
    const { page, storage } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.editor.pick(0, 1)
    await new Promise((resolve) => setTimeout(resolve, 30))
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
  })

  test('dispose writes a still-pending edit to the draft instead of losing it (system back right after an edit)', async () => {
    const { page, storage } = build({ isLoggedIn: () => false, draftDelayMs: 1000 })
    await page.load(1)
    page.editor.pick(0, 1)
    page.dispose()
    expect(readDraft(1, storage)?.slots[0].player_id).toBe(1)
  })

  test('dispose with nothing dirty writes no draft and later edits are not persisted', async () => {
    const { page, storage } = build({ isLoggedIn: () => false })
    await page.load(1)
    page.dispose()
    expect(readDraft(1, storage)).toBeNull()
    page.editor.pick(0, 1)
    await new Promise((resolve) => setTimeout(resolve, 30))
    expect(readDraft(1, storage)).toBeNull()
  })

  test('loading another team does not carry the previous team draft or state over', async () => {
    const { page } = build()
    await page.load(1)
    page.editor.pick(0, 1)
    await page.load(2)
    expect(page.teamId.value).toBe(2)
    // 新球队没有上一队的改动：回到该队自己的初始首发，且不算"未保存"
    expect(page.editor.filled.value).toBe(2)
    expect(page.editor.dirty.value).toBe(false)
  })

  test('a slow response for a previous team never overwrites the team the user switched to (A→B→C)', async () => {
    const gates: Record<number, () => void> = {}
    const lineups: Record<number, SavedLineup> = {
      2: { ...saved, formation: '3-5-2', slots: assignPlayer(emptyLineup('3-5-2'), 0, 1) },
      3: { ...saved, formation: '4-2-3-1', slots: emptyLineup('4-2-3-1') },
    }
    const { page } = build({
      getSquad: async (teamId) => {
        await new Promise<void>((resolve) => { gates[teamId] = resolve })
        return { ...squad, team: { ...squad.team, team_id: teamId } }
      },
      getMyLineup: async (teamId) => ({ lineup: lineups[teamId] ?? null }),
    })
    const toB = page.load(2)
    const toC = page.load(3)
    gates[3]()
    await toC
    gates[2]() // B 的响应最后才到
    await toB
    expect(page.teamId.value).toBe(3)
    expect(page.squad.value?.team.team_id).toBe(3)
    expect(page.editor.formation.value).toBe('4-2-3-1')
    expect(page.loading.value).toBe(false)
  })

  test('edits made while a save is in flight keep the lineup dirty and the draft', async () => {
    let finish: () => void = () => {}
    const { page, storage } = build({
      saveMyLineup: (_teamId, payload) =>
        new Promise((resolve) => {
          finish = () => resolve({ lineup: { ...saved, formation: payload.formation, slots: payload.slots } })
        }),
    })
    await page.load(1)
    page.editor.pick(0, 1)
    const pending = page.save()
    page.editor.pick(1, 2) // 保存请求在途时又改了一个位置
    finish()
    expect(await pending).toBe('saved')
    expect(page.editor.dirty.value).toBe(true)
    expect(page.saveState.value).toBe('dirty')
    expect(readDraft(1, storage)?.slots[1].player_id).toBe(2)
  })

  test('a save without edits in between still clears the dirty flag and the draft', async () => {
    const { page, storage } = build()
    await page.load(1)
    page.editor.pick(0, 1)
    await page.save()
    expect(page.editor.dirty.value).toBe(false)
    expect(readDraft(1, storage)).toBeNull()
  })

  test('a saved lineup that names players no longer in the squad is cleaned, marked dirty and explained', async () => {
    const stale: SavedLineup = {
      ...saved,
      formation: '4-3-3',
      slots: assignPlayer(assignPlayer(emptyLineup('4-3-3'), 0, 1), 1, 999),
    }
    const { page, calls } = build({ getMyLineup: async () => ({ lineup: stale }) })
    await page.load(1)
    expect(page.editor.slots.value[0].player_id).toBe(1)
    expect(page.editor.slots.value[1].player_id).toBeNull()
    expect(page.editor.dirty.value).toBe(true)
    expect(page.notice.value.length > 0).toBe(true)
    expect(await page.save()).toBe('saved') // 清理后可以正常保存，不再被后端拒绝
    expect(calls).toEqual(['save'])
  })

  test('no notice for a clean saved lineup', async () => {
    const { page } = build({ getMyLineup: async () => ({ lineup: saved }) })
    await page.load(1)
    expect(page.notice.value).toBe('')
    expect(page.editor.dirty.value).toBe(false)
  })

  const fullSquad: SquadResponse = {
    team: { team_id: 1, team_name: '成都蓉城', avatar_url: null },
    active: [
      ...[1, 15].map((n, i) => ({ player_id: 10 + i, name: `G${n}`, shirt_number: n, position: 'GK' as const, avatar_url: null, last_season: 2026 })),
      ...[2, 3, 4, 5, 22].map((n, i) => ({ player_id: 20 + i, name: `D${n}`, shirt_number: n, position: 'DF' as const, avatar_url: null, last_season: 2026 })),
      ...[6, 8, 10, 14].map((n, i) => ({ player_id: 30 + i, name: `M${n}`, shirt_number: n, position: 'MF' as const, avatar_url: null, last_season: 2026 })),
      ...[9, 11, 19].map((n, i) => ({ player_id: 40 + i, name: `F${n}`, shirt_number: n, position: 'FW' as const, avatar_url: null, last_season: 2026 })),
    ],
    former: [{ player_id: 99, name: 'Old', shirt_number: 7, position: 'FW', avatar_url: null, last_season: 2024 }],
  }

  test('a first visit with no saved lineup and no draft starts from a filled starting eleven (not dirty, not saved)', async () => {
    const { page } = build({ getSquad: async () => fullSquad })
    await page.load(1)
    expect(page.editor.filled.value).toBe(11)
    expect(page.editor.slots.value.find((s) => s.slot === 0)!.player_id).toBe(10) // 1 号门将
    expect(page.editor.slots.value.every((s) => s.player_id !== 99)).toBe(true) // 前队员不会自动入选
    expect(page.editor.dirty.value).toBe(false)
    expect(page.saveState.value).toBe('idle')
  })

  test('an automatically filled lineup is saved like any other when the user taps save', async () => {
    const { page, calls } = build({ getSquad: async () => fullSquad })
    await page.load(1)
    expect(await page.save()).toBe('saved') // 没有改动，不重复请求
    expect(calls).toEqual([])
    page.editor.pick(0, 11)
    expect(await page.save()).toBe('saved')
    expect(calls).toEqual(['save'])
  })

  test('a saved lineup or a draft is never replaced by the automatic starting eleven', async () => {
    const mine: SavedLineup = { ...saved, formation: '4-4-2', slots: assignPlayer(emptyLineup('4-4-2'), 0, 11) }
    const { page } = build({ getSquad: async () => fullSquad, getMyLineup: async () => ({ lineup: mine }) })
    await page.load(1)
    expect(page.editor.filled.value).toBe(1)
    const drafted = build({ getSquad: async () => fullSquad })
    drafted.storage.setStorageSync('fi-lineup-draft-1', { teamId: 1, formation: '3-5-2', slots: assignPlayer(emptyLineup('3-5-2'), 0, 11), savedAt: 1 })
    await drafted.page.load(1)
    expect(drafted.page.editor.filled.value).toBe(1)
  })

  test('fillStartingEleven fills the empty slots and marks the lineup dirty; it never moves players already placed', async () => {
    const { page } = build({ getSquad: async () => fullSquad })
    await page.load(1)
    page.editor.pick(0, null)
    page.editor.pick(1, null)
    page.editor.pick(2, 40)
    page.fillStartingEleven()
    expect(page.editor.slots.value.find((s) => s.slot === 2)!.player_id).toBe(40)
    expect(page.editor.filled.value).toBe(11)
    expect(page.editor.dirty.value).toBe(true)
  })

  test('fillStartingEleven before the squad is loaded does nothing', () => {
    const { page } = build()
    page.fillStartingEleven()
    expect(page.editor.filled.value).toBe(0)
    expect(page.editor.dirty.value).toBe(false)
  })
})

