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
export function draftKey(teamId: number, owner: string = 'guest'): string {
  return `fi-lineup-draft-v2-${encodeURIComponent(owner)}-${teamId}`
}

function isSlots(value: unknown): value is LineupSlots {
  return (
    Array.isArray(value)
    && value.length === 11
    && new Set(value.map(item => item?.slot)).size === 11
    && new Set(value.filter(item => item?.player_id !== null).map(item => item?.player_id)).size === value.filter(item => item?.player_id !== null).length
    && value.every(
      (item) =>
        item
        && typeof item === 'object'
        && Number.isInteger((item as { slot: unknown }).slot)
        && (item as { slot: number }).slot >= 0 && (item as { slot: number }).slot < 11
        && ((item as { player_id: unknown }).player_id === null || Number.isInteger((item as { player_id: unknown }).player_id))
        && Number.isFinite((item as { x: unknown }).x)
        && Number.isFinite((item as { y: unknown }).y),
    )
  )
}

export function readDraft(teamId: number, storage: DraftStorage = uni, owner: string = 'guest'): LineupDraft | null {
  const raw = storage.getStorageSync(draftKey(teamId, owner))
  if (!raw || typeof raw !== 'object') return null
  const draft = raw as Record<string, unknown>
  if (draft.teamId !== teamId) return null
  if (typeof draft.formation !== 'string' || !(FORMATION_CODES as readonly string[]).includes(draft.formation)) return null
  if (!isSlots(draft.slots)) return null
  return { teamId, formation: draft.formation, slots: draft.slots, savedAt: Number(draft.savedAt) || 0 }
}

export function writeDraft(draft: LineupDraft, storage: DraftStorage = uni, owner: string = 'guest'): void {
  storage.setStorageSync(draftKey(draft.teamId, owner), draft)
}

export function clearDraft(teamId: number, storage: DraftStorage = uni, owner: string = 'guest'): void {
  storage.removeStorageSync(draftKey(teamId, owner))
}

/** 草稿里名单已不存在的球员置空，避免保存时被后端拒绝。 */
export function sanitizeDraftSlots(slots: LineupSlots, knownPlayerIds: Set<number>): LineupSlots {
  return slots.map((s) => (s.player_id !== null && !knownPlayerIds.has(s.player_id) ? { ...s, player_id: null } : s))
}

/** 一次性显式游客操作凭据；普通登录不能导入游客稿。 */
export function markGuestDraftIntent(teamId: number, storage: DraftStorage): void {
  storage.setStorageSync(`fi-lineup-guest-intent-${teamId}`, true)
}
export function consumeGuestDraft(teamId: number, storage: DraftStorage): LineupDraft | null {
  if (storage.getStorageSync(`fi-lineup-guest-intent-${teamId}`) !== true) return null
  const draft = readDraft(teamId, storage)
  storage.removeStorageSync(`fi-lineup-guest-intent-${teamId}`)
  clearDraft(teamId, storage)
  return draft
}

/** 比较实际内容，不依赖编辑器的本地 dirty 标记或服务端时间戳。 */
export function sameLineup(a: { formation: string; slots: LineupSlots } | null, b: { formation: string; slots: LineupSlots } | null): boolean {
  if (!a || !b || a.formation !== b.formation || a.slots.length !== b.slots.length) return false
  const sorted = [...b.slots].sort((x, y) => x.slot - y.slot)
  return [...a.slots].sort((x, y) => x.slot - y.slot).every((s, i) => {
    const other = sorted[i]
    return s.slot === other.slot && s.player_id === other.player_id && s.x === other.x && s.y === other.y
  })
}
