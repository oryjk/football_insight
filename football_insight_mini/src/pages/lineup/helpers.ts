import type { LineupPosition, SquadPlayer } from '../../types/lineup'
import { getFormation } from './formations'

/** 位移小于它视为点击（rpx）。 */
export const TAP_THRESHOLD_RPX = 10
/** 松手点与另一名球员中心的距离小于圆点直径的这个比例，视为互换。 */
export const SWAP_DISTANCE_RATIO = 0.6
export const COORD_MIN = 0.04
export const COORD_MAX = 0.96

export type LineupSlots = Array<{ slot: number; player_id: number | null; x: number; y: number }>

export function clampCoord(value: number): number {
  if (!Number.isFinite(value)) return 0.5
  return Math.min(COORD_MAX, Math.max(COORD_MIN, value))
}

export function emptyLineup(formation: string): LineupSlots {
  return getFormation(formation).map((s) => ({ slot: s.slot, player_id: null, x: s.x, y: s.y }))
}

/** 坐标恢复为阵型默认值，球员不动。 */
export function resetPositions(slots: LineupSlots, formation: string): LineupSlots {
  const defaults = new Map(getFormation(formation).map((s) => [s.slot, s]))
  return slots.map((s) => ({ ...s, x: defaults.get(s.slot)?.x ?? s.x, y: defaults.get(s.slot)?.y ?? s.y }))
}

export function countFilled(slots: LineupSlots): number {
  return slots.filter((s) => s.player_id !== null).length
}

/**
 * 把球员放进某个位置。该球员已在场时，目标位置原来的人换到他原来的位置（目标为空则原位置清空）；
 * playerId 为 null 表示移出。
 */
export function assignPlayer(slots: LineupSlots, slot: number, playerId: number | null): LineupSlots {
  const target = slots.find((s) => s.slot === slot)
  if (!target) return slots
  if (playerId === null) return slots.map((s) => (s.slot === slot ? { ...s, player_id: null } : s))
  const origin = slots.find((s) => s.player_id === playerId)
  return slots.map((s) => {
    if (s.slot === slot) return { ...s, player_id: playerId }
    if (origin && s.slot === origin.slot) return { ...s, player_id: target.player_id }
    return s
  })
}

const POSITION_ORDER: LineupPosition[] = ['GK', 'DF', 'MF', 'FW']

/**
 * 换阵型：已排球员按位置类别放进新阵型同类别的空位；同类别放不下的（以及位置未知的）补进剩余空位，
 * 不丢人、不重复；门将位只在其他空位都满后才会被非门将占用。所有坐标取新阵型默认值。
 */
export function reassignPlayersToFormation(
  slots: LineupSlots,
  nextFormation: string,
  positionOf: (playerId: number) => LineupPosition | null,
): LineupSlots {
  const next = getFormation(nextFormation)
  const result: LineupSlots = next.map((f) => ({ slot: f.slot, player_id: null, x: f.x, y: f.y }))
  const free = (position: LineupPosition) =>
    next.filter((f) => f.position === position).map((f) => f.slot).filter((s) => result[s].player_id === null)

  const leftover: number[] = []
  const ordered = slots
    .filter((s): s is typeof s & { player_id: number } => s.player_id !== null)
    .slice()
    .sort((a, b) => a.slot - b.slot)
  for (const entry of ordered) {
    const position = positionOf(entry.player_id)
    const room = position ? free(position) : []
    if (room.length) {
      result[room[0]].player_id = entry.player_id
    } else {
      leftover.push(entry.player_id)
    }
  }
  // 溢出与位置未知的球员：先填非门将空位，最后才用门将位
  const rest = [
    ...POSITION_ORDER.filter((p) => p !== 'GK').flatMap((p) => free(p)),
    ...free('GK'),
  ]
  leftover.forEach((playerId, index) => {
    const target = rest[index]
    if (target !== undefined) result[target].player_id = playerId
  })
  return result
}

export type DragOutcome =
  | { kind: 'tap' }
  | { kind: 'swap'; withSlot: number }
  | { kind: 'move'; x: number; y: number }

/**
 * 松手判定。所有坐标为 rpx，相对球场区域左上角，且是圆点中心：
 * 位移小于 TAP_THRESHOLD_RPX → 点击；落点离另一个位置中心小于圆点直径的 SWAP_DISTANCE_RATIO → 互换最近的那个；
 * 否则移动到落点（比例坐标，钳制在 COORD_MIN–COORD_MAX）。
 */
export function resolveDragRelease(input: {
  slot: number
  fromX: number
  fromY: number
  toX: number
  toY: number
  areaWidthRpx: number
  areaHeightRpx: number
  dotDiameterRpx: number
  slots: LineupSlots
}): DragOutcome {
  const { slot, fromX, fromY, toX, toY, areaWidthRpx, areaHeightRpx, dotDiameterRpx, slots } = input
  if (Math.hypot(toX - fromX, toY - fromY) < TAP_THRESHOLD_RPX) return { kind: 'tap' }

  const reach = dotDiameterRpx * SWAP_DISTANCE_RATIO
  let nearest: { slot: number; distance: number } | null = null
  for (const other of slots) {
    if (other.slot === slot) continue
    const distance = Math.hypot(toX - other.x * areaWidthRpx, toY - other.y * areaHeightRpx)
    if (distance < reach && (!nearest || distance < nearest.distance)) nearest = { slot: other.slot, distance }
  }
  if (nearest) return { kind: 'swap', withSlot: nearest.slot }
  return { kind: 'move', x: clampCoord(toX / areaWidthRpx), y: clampCoord(toY / areaHeightRpx) }
}

/** 互换只交换球员，各位置保留自己的坐标；移动只改该位置的坐标。 */
export function applyDragOutcome(slots: LineupSlots, slot: number, outcome: DragOutcome): LineupSlots {
  if (outcome.kind === 'tap') return slots
  if (outcome.kind === 'move') {
    return slots.map((s) => (s.slot === slot ? { ...s, x: outcome.x, y: outcome.y } : s))
  }
  const a = slots.find((s) => s.slot === slot)
  const b = slots.find((s) => s.slot === outcome.withSlot)
  if (!a || !b) return slots
  return slots.map((s) => {
    if (s.slot === a.slot) return { ...s, player_id: b.player_id }
    if (s.slot === b.slot) return { ...s, player_id: a.player_id }
    return s
  })
}

/**
 * 初始首发：把现役球员按位置分组，每组取号码最靠前的几人，依次填进阵型里同类别的空位。
 * 我们没有真实首发数据，号码靠前通常是主力，只是个"像样的 11 人"，用户可以再自己换。
 * 只填空位，不动已排好的球员；位置未知或没有号码的球员排在最后，仍会被用来补满空位。
 */
export function autoFillStartingEleven(slots: LineupSlots, formation: string, activeSquad: SquadPlayer[]): LineupSlots {
  const placed = new Set(slots.map((s) => s.player_id).filter((id): id is number => id !== null))
  const byNumber = (a: SquadPlayer, b: SquadPlayer) =>
    (a.shirt_number ?? Number.MAX_SAFE_INTEGER) - (b.shirt_number ?? Number.MAX_SAFE_INTEGER) || a.player_id - b.player_id
  const pool = activeSquad.filter((p) => !placed.has(p.player_id)).slice().sort(byNumber)
  const result = slots.map((s) => ({ ...s }))
  const emptySlots = (position: LineupPosition) =>
    getFormation(formation).filter((f) => f.position === position).map((f) => f.slot)
      .filter((slot) => result.find((s) => s.slot === slot)?.player_id === null)

  const taken = new Set<number>()
  for (const position of POSITION_ORDER) {
    const candidates = pool.filter((p) => p.position === position && !taken.has(p.player_id))
    emptySlots(position).forEach((slot, index) => {
      const player = candidates[index]
      if (!player) return
      result.find((s) => s.slot === slot)!.player_id = player.player_id
      taken.add(player.player_id)
    })
  }
  // 同类别人不够的位置，用剩下的球员（位置不符、未知位置、没号码）按号码顺序补满，门将位最后才补
  const leftover = pool.filter((p) => !taken.has(p.player_id))
  const rest = [
    ...POSITION_ORDER.filter((p) => p !== 'GK').flatMap((p) => emptySlots(p)),
    ...emptySlots('GK'),
  ]
  rest.forEach((slot, index) => {
    const player = leftover[index]
    if (player) result.find((s) => s.slot === slot)!.player_id = player.player_id
  })
  return result
}
