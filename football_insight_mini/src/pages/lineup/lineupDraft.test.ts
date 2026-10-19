import { describe, expect, test } from 'bun:test'
import { assignPlayer, emptyLineup } from './helpers'
import { clearDraft, draftKey, readDraft, sanitizeDraftSlots, writeDraft } from './lineupDraft'

function memoryStorage() {
  const map = new Map<string, unknown>()
  return {
    getStorageSync: (k: string) => map.get(k) ?? '',
    setStorageSync: (k: string, v: unknown) => { map.set(k, v) },
    removeStorageSync: (k: string) => { map.delete(k) },
  }
}

describe('lineup draft', () => {
  test('is stored per team', () => {
    expect(draftKey(1) === draftKey(2)).toBe(false)
    expect(draftKey(1)).toBe('fi-lineup-draft-v2-guest-1')
    expect(draftKey(1, 'user:A') === draftKey(1, 'user:B')).toBe(false)
    expect(draftKey(1, 'user:A') === draftKey(1)).toBe(false)
  })

  test('round trips and clears', () => {
    const storage = memoryStorage()
    const slots = assignPlayer(emptyLineup('4-4-2'), 0, 5)
    writeDraft({ teamId: 1, formation: '4-4-2', slots, savedAt: 1 }, storage)
    expect(readDraft(1, storage)).toEqual({ teamId: 1, formation: '4-4-2', slots, savedAt: 1 })
    expect(readDraft(2, storage)).toBeNull()
    clearDraft(1, storage)
    expect(readDraft(1, storage)).toBeNull()
  })

  test('rejects corrupted drafts instead of crashing', () => {
    const storage = memoryStorage()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '4-4-2', slots: 'nope', savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '9-9-9', slots: emptyLineup('4-4-2'), savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '4-4-2', slots: emptyLineup('4-4-2').slice(0, 5), savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), { teamId: 2, formation: '4-4-2', slots: emptyLineup('4-4-2'), savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
    storage.setStorageSync(draftKey(1), 'not an object')
    expect(readDraft(1, storage)).toBeNull()
    const badSlot = emptyLineup('4-4-2').map((s, i) => (i === 0 ? { ...s, x: 'left' } : s))
    storage.setStorageSync(draftKey(1), { teamId: 1, formation: '4-4-2', slots: badSlot, savedAt: 1 })
    expect(readDraft(1, storage)).toBeNull()
  })

  test('sanitize blanks players the squad no longer contains', () => {
    const slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 0, 1), 1, 999)
    const clean = sanitizeDraftSlots(slots, new Set([1]))
    expect(clean[0].player_id).toBe(1)
    expect(clean[1].player_id).toBeNull()
  })
})
