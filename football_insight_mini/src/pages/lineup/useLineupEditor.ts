import { computed, ref } from 'vue'
import type { LineupPosition } from '../../types/lineup'
import { DEFAULT_FORMATION } from './formations'
import {
  applyDragOutcome,
  assignPlayer,
  countFilled,
  emptyLineup,
  reassignPlayersToFormation,
  resetPositions,
  type DragOutcome,
  type LineupSlots,
} from './helpers'

/** 编辑状态：阵型、11 个位置、是否有未保存的改动。所有判定都委托给 helpers 里的纯函数。 */
export function useLineupEditor() {
  const formation = ref<string>(DEFAULT_FORMATION)
  const slots = ref<LineupSlots>(emptyLineup(DEFAULT_FORMATION))
  const dirty = ref(false)
  const selectedSlot = ref<number | null>(null)
  const filled = computed(() => countFilled(slots.value))

  function load(nextFormation: string, nextSlots: LineupSlots, markDirty = false): void {
    formation.value = nextFormation
    slots.value = nextSlots
    selectedSlot.value = null
    dirty.value = markDirty
  }

  function setFormation(next: string, positionOf: (playerId: number) => LineupPosition | null): void {
    if (next === formation.value) return
    slots.value = reassignPlayersToFormation(slots.value, next, positionOf)
    formation.value = next
    dirty.value = true
  }

  function pick(slot: number, playerId: number | null): void {
    slots.value = assignPlayer(slots.value, slot, playerId)
    dirty.value = true
  }

  function applyDrag(slot: number, outcome: DragOutcome): void {
    if (outcome.kind === 'tap') return
    slots.value = applyDragOutcome(slots.value, slot, outcome)
    dirty.value = true
  }

  function reset(): void {
    slots.value = resetPositions(slots.value, formation.value)
    dirty.value = true
  }

  function markSaved(): void {
    dirty.value = false
  }

  return { formation, slots, dirty, selectedSlot, filled, load, setFormation, pick, applyDrag, reset, markSaved }
}
