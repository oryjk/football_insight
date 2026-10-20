import type { LineupConsensus, LineupConsensusPlayer, LineupPosition } from '../types/lineup'
import { getFormation } from './lineupFormations'

/** 与共识 11 人相同的球员数；共识未解锁时为 null（不显示重合度）。 */
export function lineupOverlap(playerIds: Array<number | null>, consensus: LineupConsensus | null): number | null {
  if (!consensus?.unlocked) return null
  const chosen = new Set(consensus.players.map(player => player.player_id))
  return new Set(playerIds.filter((id): id is number => id !== null && chosen.has(id))).size
}

/** 不在共识里的球员（"独家"）；未解锁时为空。 */
export function exclusivePlayerIds(playerIds: Array<number | null>, consensus: LineupConsensus | null): number[] {
  if (!consensus?.unlocked) return []
  const chosen = new Set(consensus.players.map(player => player.player_id))
  return playerIds.filter((id): id is number => id !== null && !chosen.has(id))
}

export function overlapText(count: number | null, who: '你' | 'TA'): string {
  return count === null ? '' : `${who}和大家重合 ${count} / 11`
}

export function consensusProgressText(total: number, threshold: number): string {
  return `已有 ${total} 套阵容，再有 ${Math.max(0, threshold - total)} 套就能看到大家心中的最佳 11 人`
}

export function consensusPlayerHint(player: { name: string; pick_rate: number }, mine: boolean): string {
  return `${player.pick_rate}% 的球迷选了${player.name}${mine ? ' · 你也选了' : ''}`
}

/** 按 group 把共识 11 人依次放进共识阵型的同类位置（坐标沿用阵型定义）。 */
export function placeConsensus(consensus: LineupConsensus): Array<{ slot: number; x: number; y: number; player: LineupConsensusPlayer }> {
  if (!consensus.unlocked || !consensus.formation) return []
  const queues: Record<LineupPosition, LineupConsensusPlayer[]> = { GK: [], DF: [], MF: [], FW: [] }
  for (const player of consensus.players) queues[player.group].push(player)
  return getFormation(consensus.formation).flatMap(slot => {
    const player = queues[slot.position].shift()
    return player ? [{ slot: slot.slot, x: slot.x, y: slot.y, player }] : []
  })
}
