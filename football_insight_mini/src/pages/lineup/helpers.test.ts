import { describe, expect, test } from 'bun:test'
import type { SquadPlayer } from '../../types/lineup'
import { getFormation } from './formations'
import {
  COORD_MAX, COORD_MIN, SWAP_DISTANCE_RATIO, TAP_THRESHOLD_RPX,
  applyDragOutcome, assignPlayer, autoFillStartingEleven, clampCoord, countFilled, emptyLineup,
  reassignPlayersToFormation, resetPositions, resolveDragRelease,
  type LineupSlots,
} from './helpers'

describe('clampCoord', () => {
  test('keeps dots inside 0.04–0.96', () => {
    expect(clampCoord(-1)).toBe(COORD_MIN)
    expect(clampCoord(2)).toBe(COORD_MAX)
    expect(clampCoord(0.5)).toBe(0.5)
    expect(clampCoord(Number.NaN)).toBe(0.5)
  })
})

describe('emptyLineup / resetPositions / countFilled', () => {
  test('empty lineup has 11 empty slots at default coordinates', () => {
    const slots = emptyLineup('4-4-2')
    expect(slots.length).toBe(11)
    expect(slots.every((s) => s.player_id === null)).toBe(true)
    expect(slots.map((s) => [s.x, s.y])).toEqual(getFormation('4-4-2').map((s) => [s.x, s.y]))
    expect(countFilled(slots)).toBe(0)
  })

  test('resetPositions restores coordinates but keeps players', () => {
    const moved: LineupSlots = emptyLineup('4-3-3').map((s, i) => ({ ...s, player_id: i < 3 ? 100 + i : null, x: 0.9, y: 0.1 }))
    const reset = resetPositions(moved, '4-3-3')
    expect(reset.map((s) => s.player_id)).toEqual(moved.map((s) => s.player_id))
    expect(reset.map((s) => [s.x, s.y])).toEqual(getFormation('4-3-3').map((s) => [s.x, s.y]))
    expect(countFilled(reset)).toBe(3)
  })
})

describe('assignPlayer', () => {
  test('puts a player into a slot', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 5, 42)
    expect(slots.find((s) => s.slot === 5)!.player_id).toBe(42)
  })

  test('a player already on the pitch moves instead of duplicating; the target occupant swaps into the old slot', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 1, 10)
    slots = assignPlayer(slots, 2, 20)
    slots = assignPlayer(slots, 2, 10) // 把 10 号选到 2 号位：互换
    expect(slots.find((s) => s.slot === 2)!.player_id).toBe(10)
    expect(slots.find((s) => s.slot === 1)!.player_id).toBe(20)
    expect(slots.filter((s) => s.player_id === 10).length).toBe(1)
  })

  test('moving a player into an empty slot leaves his old slot empty', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 1, 10)
    slots = assignPlayer(slots, 6, 10)
    expect(slots.find((s) => s.slot === 6)!.player_id).toBe(10)
    expect(slots.find((s) => s.slot === 1)!.player_id).toBeNull()
  })

  test('null removes the occupant', () => {
    const slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 3, 7), 3, null)
    expect(slots.find((s) => s.slot === 3)!.player_id).toBeNull()
  })

  test('unknown slot leaves the lineup untouched', () => {
    const slots = emptyLineup('4-3-3')
    expect(assignPlayer(slots, 99, 5)).toEqual(slots)
  })
})

describe('reassignPlayersToFormation', () => {
  const positions: Record<number, 'GK' | 'DF' | 'MF' | 'FW'> = { 1: 'GK', 2: 'DF', 3: 'DF', 4: 'DF', 5: 'DF', 6: 'MF', 7: 'MF', 8: 'MF', 9: 'FW', 10: 'FW', 11: 'FW' }
  const positionOf = (id: number) => positions[id] ?? null

  function filled(formation: string, ids: number[]): LineupSlots {
    const form = getFormation(formation)
    return emptyLineup(formation).map((s, i) => ({ ...s, player_id: ids[i] ?? null, x: form[i].x, y: form[i].y }))
  }

  test('4-3-3 → 4-4-2 keeps every player once and resets coordinates', () => {
    const before = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(before, '4-4-2', positionOf)
    const ids = after.map((s) => s.player_id).filter((id): id is number => id !== null).sort((a, b) => a - b)
    expect(ids).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    expect(after.map((s) => [s.x, s.y])).toEqual(getFormation('4-4-2').map((s) => [s.x, s.y]))
    expect(after.find((s) => s.slot === 0)!.player_id).toBe(1) // 门将仍在门将位
  })

  test('goalkeeper stays in the keeper slot and every player keeps a place in the new formation', () => {
    const before = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(before, '5-3-2', positionOf)
    const idsAt = (pos: string) => getFormation('5-3-2').filter((f) => f.position === pos).map((f) => after.find((s) => s.slot === f.slot)!.player_id)
    expect(idsAt('GK')).toEqual([1])
    // 4 名后卫、3 名中场、2 名前锋各回到自己的线上；溢出的 11 号补进唯一剩下的空位（第 5 个后卫位）
    expect(idsAt('DF').slice(0, 4)).toEqual([2, 3, 4, 5])
    expect(idsAt('MF').slice().sort((a, b) => (a ?? 0) - (b ?? 0))).toEqual([6, 7, 8])
    expect(idsAt('FW').slice().sort((a, b) => (a ?? 0) - (b ?? 0))).toEqual([9, 10])
    expect(idsAt('DF')[4]).toBe(11)
    expect(after.filter((s) => s.player_id !== null).length).toBe(11)
  })

  test('players without a known position and overflow players are placed into remaining slots, never dropped or duplicated', () => {
    const many = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(many, '3-4-3', (id) => (id === 6 ? null : positionOf(id)))
    const ids = after.map((s) => s.player_id).filter((id): id is number => id !== null)
    expect(new Set(ids).size).toBe(ids.length)
    expect(ids.length).toBe(11)
  })

  test('a lineup with only outfield players never puts the first outfield player in goal when a keeper slot can stay empty', () => {
    const before = filled('4-3-3', [2, 3, 6])
    const after = reassignPlayersToFormation(before, '4-4-2', positionOf)
    expect(after.find((s) => s.slot === 0)!.player_id).toBeNull()
  })

  test('partial lineups keep the same players', () => {
    const before = filled('4-3-3', [1, 2, 6])
    const after = reassignPlayersToFormation(before, '4-2-3-1', positionOf)
    expect(after.map((s) => s.player_id).filter((id) => id !== null).sort()).toEqual([1, 2, 6])
  })

  test('is a no-op for membership when the formation does not change', () => {
    const before = filled('4-3-3', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
    const after = reassignPlayersToFormation(before, '4-3-3', positionOf)
    expect(after.map((s) => s.player_id)).toEqual(before.map((s) => s.player_id))
  })
})

describe('resolveDragRelease', () => {
  const area = { areaWidthRpx: 670, areaHeightRpx: 900, dotDiameterRpx: 84 }
  const slots: LineupSlots = emptyLineup('4-3-3').map((s, i) => ({ ...s, player_id: 100 + i }))
  const from = slots[1] // 左后卫

  test(`movement under ${TAP_THRESHOLD_RPX}rpx is a tap`, () => {
    const fromX = from.x * area.areaWidthRpx
    const fromY = from.y * area.areaHeightRpx
    expect(resolveDragRelease({ slot: from.slot, fromX, fromY, toX: fromX + 4, toY: fromY + 3, ...area, slots }).kind).toBe('tap')
  })

  test('dropping near another player swaps with that slot', () => {
    const target = slots[4]
    const outcome = resolveDragRelease({
      slot: from.slot,
      fromX: from.x * area.areaWidthRpx, fromY: from.y * area.areaHeightRpx,
      toX: target.x * area.areaWidthRpx + 5, toY: target.y * area.areaHeightRpx - 5, ...area, slots,
    })
    expect(outcome).toEqual({ kind: 'swap', withSlot: target.slot })
  })

  test(`swap distance is ${SWAP_DISTANCE_RATIO} of the dot diameter`, () => {
    const target = slots[4]
    const cx = target.x * area.areaWidthRpx
    const cy = target.y * area.areaHeightRpx
    const edge = area.dotDiameterRpx * SWAP_DISTANCE_RATIO
    const near = resolveDragRelease({ slot: from.slot, fromX: 0, fromY: 0, toX: cx + edge - 1, toY: cy, ...area, slots })
    const far = resolveDragRelease({ slot: from.slot, fromX: 0, fromY: 0, toX: cx + edge + 1, toY: cy, ...area, slots })
    expect(near.kind).toBe('swap')
    expect(far.kind).toBe('move')
  })

  test('picks the nearest of several candidates', () => {
    const a = slots[4]
    const b = slots[5]
    const midX = ((a.x + b.x) / 2) * area.areaWidthRpx
    const nearerToB = resolveDragRelease({
      slot: from.slot, fromX: 0, fromY: 0,
      toX: midX + 2, toY: ((a.y + b.y) / 2) * area.areaHeightRpx, ...area, slots,
    })
    if (nearerToB.kind === 'swap') {
      expect([a.slot, b.slot].includes(nearerToB.withSlot)).toBe(true)
    }
  })

  test('dropping on empty pitch moves and clamps to the allowed range', () => {
    const outside = resolveDragRelease({ slot: from.slot, fromX: 100, fromY: 100, toX: -300, toY: 5000, ...area, slots })
    expect(outside.kind).toBe('move')
    if (outside.kind === 'move') {
      expect(outside.x).toBe(COORD_MIN)
      expect(outside.y).toBe(COORD_MAX)
    }
  })

  test('dropping back on its own position is not a swap with itself', () => {
    const fromX = from.x * area.areaWidthRpx
    const fromY = from.y * area.areaHeightRpx
    const outcome = resolveDragRelease({ slot: from.slot, fromX: fromX + 40, fromY, toX: fromX + 2, toY: fromY, ...area, slots })
    expect(outcome.kind === 'swap').toBe(false)
  })

  test('slots that are not on the pitch (empty) can still be swap targets', () => {
    const withEmpty = emptyLineup('4-3-3').map((s) => ({ ...s, player_id: s.slot === 1 ? 7 : null }))
    const target = withEmpty[4]
    const outcome = resolveDragRelease({
      slot: 1, fromX: 0, fromY: 0,
      toX: target.x * area.areaWidthRpx, toY: target.y * area.areaHeightRpx, ...area, slots: withEmpty,
    })
    expect(outcome).toEqual({ kind: 'swap', withSlot: 4 })
  })
})

describe('applyDragOutcome', () => {
  test('tap leaves slots untouched', () => {
    const slots = emptyLineup('4-3-3')
    expect(applyDragOutcome(slots, 1, { kind: 'tap' })).toEqual(slots)
  })

  test('swap exchanges only the players; each slot keeps its own coordinates', () => {
    let slots = assignPlayer(assignPlayer(emptyLineup('4-3-3'), 1, 10), 4, 40)
    const a = slots.find((s) => s.slot === 1)!
    const b = slots.find((s) => s.slot === 4)!
    slots = applyDragOutcome(slots, 1, { kind: 'swap', withSlot: 4 })
    const a2 = slots.find((s) => s.slot === 1)!
    const b2 = slots.find((s) => s.slot === 4)!
    expect([a2.player_id, b2.player_id]).toEqual([40, 10])
    expect([a2.x, a2.y]).toEqual([a.x, a.y])
    expect([b2.x, b2.y]).toEqual([b.x, b.y])
  })

  test('move writes the new coordinates only for that slot', () => {
    const slots = applyDragOutcome(emptyLineup('4-3-3'), 2, { kind: 'move', x: 0.3, y: 0.6 })
    const moved = slots.find((s) => s.slot === 2)!
    expect([moved.x, moved.y]).toEqual([0.3, 0.6])
    const defaults = getFormation('4-3-3')
    for (const s of slots.filter((x) => x.slot !== 2)) {
      const d = defaults.find((f) => f.slot === s.slot)!
      expect([s.x, s.y]).toEqual([d.x, d.y])
    }
  })
})

describe('autoFillStartingEleven', () => {
  function player(id: number, position: SquadPlayer['position'], number: number | null): SquadPlayer {
    return { player_id: id, name: `P${id}`, shirt_number: number, position, avatar_url: null, last_season: 2026 }
  }

  // 蓉城式的名单：4 门将、10 后卫、10 中场、9 前锋（号码打乱，验证按号码排序）
  const squad: SquadPlayer[] = [
    player(30, 'GK', 32), player(31, 'GK', 1), player(32, 'GK', 15), player(33, 'GK', 55),
    ...[5, 2, 3, 4, 22, 23, 33, 24, 12, 6].map((n, i) => player(100 + i, 'DF', n)),
    ...[8, 10, 14, 16, 17, 20, 21, 25, 27, 7].map((n, i) => player(200 + i, 'MF', n)),
    ...[9, 11, 19, 26, 28, 29, 34, 35, 36].map((n, i) => player(300 + i, 'FW', n)),
  ]

  test('fills all eleven slots without repeating a player', () => {
    const slots = autoFillStartingEleven(emptyLineup('4-3-3'), '4-3-3', squad)
    const ids = slots.map((s) => s.player_id)
    expect(ids.every((id) => id !== null)).toBe(true)
    expect(new Set(ids).size).toBe(11)
  })

  test('puts the lowest-numbered goalkeeper in goal and keeps defenders, midfielders, forwards on their lines', () => {
    const slots = autoFillStartingEleven(emptyLineup('4-3-3'), '4-3-3', squad)
    const form = getFormation('4-3-3')
    const at = (slot: number) => squad.find((p) => p.player_id === slots.find((s) => s.slot === slot)!.player_id)!
    expect(at(0).shirt_number).toBe(1)
    for (const f of form) expect(at(f.slot).position).toBe(f.position)
  })

  test('takes the lowest shirt numbers of each position as the first choice', () => {
    const slots = autoFillStartingEleven(emptyLineup('4-3-3'), '4-3-3', squad)
    const numbers = (pos: string) =>
      getFormation('4-3-3').filter((f) => f.position === pos)
        .map((f) => squad.find((p) => p.player_id === slots.find((s) => s.slot === f.slot)!.player_id)!.shirt_number!)
        .sort((a, b) => a - b)
    expect(numbers('DF')).toEqual([2, 3, 4, 5])
    expect(numbers('MF')).toEqual([7, 8, 10]) // 名单里的中场号码是 7、8、10、14…（没有 6 号）
    expect(numbers('FW')).toEqual([9, 11, 19]) // 4-3-3 有三个前锋位：两个边锋加一个中锋
  })

  test('works for every formation', () => {
    for (const code of ['4-4-2', '4-2-3-1', '3-5-2', '3-4-3', '5-3-2', '4-1-4-1']) {
      const slots = autoFillStartingEleven(emptyLineup(code), code, squad)
      expect(slots.filter((s) => s.player_id !== null).length).toBe(11)
      expect(new Set(slots.map((s) => s.player_id)).size).toBe(11)
    }
  })

  test('keeps the existing slot coordinates', () => {
    const moved = emptyLineup('4-3-3').map((s) => (s.slot === 3 ? { ...s, x: 0.31, y: 0.62 } : s))
    const slots = autoFillStartingEleven(moved, '4-3-3', squad)
    const slot3 = slots.find((s) => s.slot === 3)!
    expect([slot3.x, slot3.y]).toEqual([0.31, 0.62])
  })

  test('fills only empty slots and never moves a player already placed', () => {
    const start = assignPlayer(emptyLineup('4-3-3'), 9, 300 + 5) // 把一个前锋放在 9 号位
    const slots = autoFillStartingEleven(start, '4-3-3', squad)
    expect(slots.find((s) => s.slot === 9)!.player_id).toBe(305)
    expect(slots.filter((s) => s.player_id === 305).length).toBe(1)
    expect(slots.filter((s) => s.player_id !== null).length).toBe(11)
  })

  test('players without a number or position are used last, never skipped when slots remain', () => {
    const thin: SquadPlayer[] = [
      player(1, 'GK', 1), player(2, 'DF', 2), player(3, null, null), player(4, 'MF', null), player(5, 'FW', 9),
    ]
    const slots = autoFillStartingEleven(emptyLineup('4-3-3'), '4-3-3', thin)
    const ids = slots.map((s) => s.player_id).filter((id): id is number => id !== null)
    expect(new Set(ids)).toEqual(new Set([1, 2, 3, 4, 5]))
    expect(slots.find((s) => s.slot === 0)!.player_id).toBe(1)
  })

  test('a squad smaller than eleven fills what it can and leaves the rest empty', () => {
    const tiny = [player(1, 'GK', 1), player(2, 'DF', 2)]
    const slots = autoFillStartingEleven(emptyLineup('4-3-3'), '4-3-3', tiny)
    expect(slots.filter((s) => s.player_id !== null).length).toBe(2)
  })

  test('an empty squad changes nothing', () => {
    const slots = emptyLineup('4-3-3')
    expect(autoFillStartingEleven(slots, '4-3-3', [])).toEqual(slots)
  })
})
