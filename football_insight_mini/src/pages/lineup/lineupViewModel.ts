import type { ClubPlayer, LineupPosition, SquadPlayer, SquadResponse } from '../../types/lineup'
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
    /** 其他球队的球员：队名简称角标（如「泰山」）；本队球员为 null。 */
    club: string | null
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
  /** 搜索结果里其他球队的球员：所在球队全名；本队球员为 null。 */
  club: string | null
}

export interface SheetGroup {
  key: 'active' | 'former' | 'team' | 'others'
  title: string
  players: SheetPlayer[]
}

/** 球队简称，用作外队球员角标：去掉「队」字，否则去掉前两个字的城市名（山东泰山→泰山、天津津门虎→津门虎）。 */
export function teamShortName(name: string): string {
  const chars = Array.from(name.trim())
  if (chars[chars.length - 1] === '队') return chars.slice(0, -1).join('')
  return chars.length >= 4 ? chars.slice(2).join('') : chars.join('')
}

/** 其他球队的球员（不在本队名单里、带所在球队）。 */
export function isOutsider(player: ClubPlayer, squad: SquadResponse | null): boolean {
  return Boolean(player.team && player.team.team_id !== squad?.team.team_id)
}

/**
 * 球场上 11 个位置的展示数据：位置信息来自阵型，坐标来自当前阵容，球员信息关联名单（现役或前队员），
 * 名单里没有的再从 `extras`（搜索选来的或补查到的其他球队球员）里找，带队名角标。
 */
export function buildPitchPlayers(slots: LineupSlots, formation: string, squad: SquadResponse | null, extras: ClubPlayer[] = []): PitchPlayer[] {
  const byId = new Map<number, { player: SquadPlayer; former: boolean; club: string | null }>()
  for (const player of squad?.active ?? []) byId.set(player.player_id, { player, former: false, club: null })
  for (const player of squad?.former ?? []) {
    if (!byId.has(player.player_id)) byId.set(player.player_id, { player, former: true, club: null })
  }
  for (const player of extras) {
    if (!byId.has(player.player_id) && player.team) {
      byId.set(player.player_id, { player, former: false, club: teamShortName(player.team.team_name) })
    }
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
            club: entry.club,
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
    club: null,
  }
}

/**
 * 选人弹层的搜索结果：本队（现役和前队员，在本地名单里按名字匹配，所以常见姓氏也不会被截掉）在前，
 * 其他球队（后端搜索结果去掉本队名单里已有的）在后；每组内与该位置对口的排前面。
 */
export function buildSearchGroups(
  query: string, squad: SquadResponse, slots: LineupSlots, targetPosition: LineupPosition, remote: ClubPlayer[],
): SheetGroup[] {
  const keyword = query.trim()
  if (!keyword) return []
  const onPitch = new Set(slots.map((s) => s.player_id).filter((id): id is number => id !== null))
  const order = (players: SheetPlayer[]) => [...players.filter((p) => p.preferred), ...players.filter((p) => !p.preferred)]
  const own = new Set([...squad.active, ...squad.former].map((p) => p.player_id))
  const team = order([
    ...squad.active.filter((p) => p.name.includes(keyword)).map((p) => toSheetPlayer(p, false, onPitch, targetPosition)),
    ...squad.former.filter((p) => p.name.includes(keyword) && !squad.active.some((a) => a.player_id === p.player_id))
      .map((p) => toSheetPlayer(p, true, onPitch, targetPosition)),
  ])
  const others = order(remote.filter((p) => !own.has(p.player_id) && p.team)
    .map((p) => ({ ...toSheetPlayer(p, false, onPitch, targetPosition), club: p.team!.team_name })))
  const groups: SheetGroup[] = []
  if (team.length) groups.push({ key: 'team', title: '本队', players: team })
  if (others.length) groups.push({ key: 'others', title: '其他球队', players: others })
  return groups
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

export function positionIndex(squad: SquadResponse | null, extras: ClubPlayer[] = []): (playerId: number) => LineupPosition | null {
  const map = new Map<number, LineupPosition | null>()
  for (const player of [...extras, ...(squad?.former ?? []), ...(squad?.active ?? [])]) map.set(player.player_id, player.position)
  return (playerId) => map.get(playerId) ?? null
}

/** 选人弹层顶部的胶囊：每组一个，带人数；只有一组时不需要切换。 */
export function buildSheetTabs(groups: SheetGroup[]): Array<{ key: string; label: string }> {
  if (groups.length < 2) return []
  return groups.map((group) => ({ key: group.key, label: `${group.title} ${group.players.length}` }))
}

/** 当前应显示的分组：选中的那组存在就用它，否则回到现役（没有前队员时不会停在不存在的那一页）。 */
export function resolveSheetTab(groups: SheetGroup[], selected: string): SheetGroup | null {
  return groups.find((group) => group.key === selected) ?? groups.find((group) => group.key === 'active') ?? groups[0] ?? null
}
