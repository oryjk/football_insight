import type { LineupPosition, SquadPlayer, SquadResponse } from '../../types/lineup'
import { getFormation } from './formations'
import type { LineupSlots } from './helpers'

export interface PitchPlayer {
  slot: number
  label: string
  position: LineupPosition
  x: number
  y: number
  player: null | {
    player_id: number
    name: string
    shirt_number: number | null
    avatar_url: string | null
    former: boolean
    last_season: number
  }
}

export interface SheetPlayer {
  player_id: number
  name: string
  shirt_number: number | null
  position: LineupPosition | null
  avatar_url: string | null
  former: boolean
  last_season: number
  onPitch: boolean
  preferred: boolean
}

export interface SheetGroup {
  key: 'active' | 'former'
  title: string
  players: SheetPlayer[]
}

/** 球场上 11 个位置的展示数据：位置信息来自阵型，坐标来自当前阵容，球员信息关联名单（现役或前队员）。 */
export function buildPitchPlayers(slots: LineupSlots, formation: string, squad: SquadResponse | null): PitchPlayer[] {
  const byId = new Map<number, { player: SquadPlayer; former: boolean }>()
  for (const player of squad?.active ?? []) byId.set(player.player_id, { player, former: false })
  for (const player of squad?.former ?? []) {
    if (!byId.has(player.player_id)) byId.set(player.player_id, { player, former: true })
  }
  const layout = getFormation(formation)
  return layout.map((spot) => {
    const slot = slots.find((s) => s.slot === spot.slot)
    const entry = slot && slot.player_id !== null ? byId.get(slot.player_id) : undefined
    return {
      slot: spot.slot,
      label: spot.label,
      position: spot.position,
      x: slot?.x ?? spot.x,
      y: slot?.y ?? spot.y,
      player: entry
        ? {
            player_id: entry.player.player_id,
            name: entry.player.name,
            shirt_number: entry.player.shirt_number,
            avatar_url: entry.player.avatar_url,
            former: entry.former,
            last_season: entry.player.last_season,
          }
        : null,
    }
  })
}

function toSheetPlayer(player: SquadPlayer, former: boolean, onPitch: Set<number>, target: LineupPosition): SheetPlayer {
  return {
    player_id: player.player_id,
    name: player.name,
    shirt_number: player.shirt_number,
    position: player.position,
    avatar_url: player.avatar_url,
    former,
    last_season: player.last_season,
    onPitch: onPitch.has(player.player_id),
    preferred: player.position === target,
  }
}

/** 选人弹层的分组：现役在前、前队员在后；每组内与该位置对口的排最前，其余保持名单原有顺序；已上场的做标记。 */
export function buildSheetGroups(squad: SquadResponse, slots: LineupSlots, targetPosition: LineupPosition): SheetGroup[] {
  const onPitch = new Set(slots.map((s) => s.player_id).filter((id): id is number => id !== null))
  const order = (players: SheetPlayer[]) => [...players.filter((p) => p.preferred), ...players.filter((p) => !p.preferred)]
  const groups: SheetGroup[] = [
    {
      key: 'active',
      title: '现役',
      players: order(squad.active.map((p) => toSheetPlayer(p, false, onPitch, targetPosition))),
    },
  ]
  if (squad.former.length) {
    groups.push({
      key: 'former',
      title: '前队员',
      players: order(squad.former.map((p) => toSheetPlayer(p, true, onPitch, targetPosition))),
    })
  }
  return groups
}

export function positionIndex(squad: SquadResponse | null): (playerId: number) => LineupPosition | null {
  const map = new Map<number, LineupPosition | null>()
  for (const player of [...(squad?.former ?? []), ...(squad?.active ?? [])]) map.set(player.player_id, player.position)
  return (playerId) => map.get(playerId) ?? null
}
