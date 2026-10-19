import { FORMATION_CODES } from './formations'
import type { LineupSlots } from './helpers'

export interface LineupDraft {
  teamId: number
  formation: string
  slots: LineupSlots
  savedAt: number
}

/** 只要求草稿用到的三个存储方法，便于测试注入内存实现；`uni` 本身满足该接口。 */
export interface DraftStorage {
  getStorageSync(key: string): unknown
  setStorageSync(key: string, value: unknown): void
  removeStorageSync(key: string): void
}

/** 每支球队一份草稿：没登录先排、保存失败、系统返回时改动都不会丢。 */
export function draftKey(teamId: number): string {
  return `fi-lineup-draft-${teamId}`
}

function isSlots(value: unknown): value is LineupSlots {
  return (
    Array.isArray(value)
    && value.length === 11
    && value.every(
      (item) =>
        item
        && typeof item === 'object'
        && Number.isInteger((item as { slot: unknown }).slot)
        && ((item as { player_id: unknown }).player_id === null || Number.isInteger((item as { player_id: unknown }).player_id))
        && Number.isFinite((item as { x: unknown }).x)
        && Number.isFinite((item as { y: unknown }).y),
    )
  )
}

export function readDraft(teamId: number, storage: DraftStorage = uni): LineupDraft | null {
  const raw = storage.getStorageSync(draftKey(teamId))
  if (!raw || typeof raw !== 'object') return null
  const draft = raw as Record<string, unknown>
  if (draft.teamId !== teamId) return null
  if (typeof draft.formation !== 'string' || !(FORMATION_CODES as readonly string[]).includes(draft.formation)) return null
  if (!isSlots(draft.slots)) return null
  return { teamId, formation: draft.formation, slots: draft.slots, savedAt: Number(draft.savedAt) || 0 }
}

export function writeDraft(draft: LineupDraft, storage: DraftStorage = uni): void {
  storage.setStorageSync(draftKey(draft.teamId), draft)
}

export function clearDraft(teamId: number, storage: DraftStorage = uni): void {
  storage.removeStorageSync(draftKey(teamId))
}

/** 草稿里名单已不存在的球员置空，避免保存时被后端拒绝。 */
export function sanitizeDraftSlots(slots: LineupSlots, knownPlayerIds: Set<number>): LineupSlots {
  return slots.map((s) => (s.player_id !== null && !knownPlayerIds.has(s.player_id) ? { ...s, player_id: null } : s))
}
