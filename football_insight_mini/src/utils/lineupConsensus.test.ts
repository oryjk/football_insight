import { describe, expect, test } from 'bun:test'
import type { LineupConsensus } from '../types/lineup'
import { consensusPlayerHint, consensusProgressText, exclusivePlayerIds, lineupOverlap, overlapText, placeConsensus } from './lineupConsensus'

const ids = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32]
const consensus: LineupConsensus = {
  team: { team_id: 77680, team_name: '成都蓉城', avatar_url: null },
  total: 36, threshold: 10, unlocked: true, formation: '4-3-3', formation_share: 58,
  players: ids.map((id, index) => ({
    player_id: id, name: `P${id}`, avatar_url: null, shirt_number: id, position: null,
    group: index === 0 ? 'GK' : index <= 4 ? 'DF' : index <= 7 ? 'MF' : 'FW', pick_rate: 90 - index,
  })),
}
const locked: LineupConsensus = { ...consensus, total: 2, unlocked: false, formation: null, formation_share: 0, players: [] }

describe('lineup consensus helpers', () => {
  test('overlap counts shared players and ignores empty slots', () => {
    expect(lineupOverlap(ids, consensus)).toBe(11)
    expect(lineupOverlap([1, 10, 11, 99, 98, null, 20, 21, 22, 30, 31], consensus)).toBe(8)
    expect(lineupOverlap(ids, locked)).toBe(null)
    expect(lineupOverlap(ids, null)).toBe(null)
  })

  test('exclusive players are the ones outside the consensus', () => {
    expect(exclusivePlayerIds([1, 99, null, 10, 98], consensus)).toEqual([99, 98])
    expect(exclusivePlayerIds([99], locked)).toEqual([])
  })

  test('texts', () => {
    expect(overlapText(8, '你')).toBe('你和大家重合 8 / 11')
    expect(overlapText(6, 'TA')).toBe('TA和大家重合 6 / 11')
    expect(overlapText(null, '你')).toBe('')
    expect(consensusProgressText(2, 10)).toBe('已有 2 套阵容，再有 8 套，这里会汇总出大家心中的最佳 11 人')
    expect(consensusProgressText(12, 10)).toBe('已有 12 套阵容，再有 0 套，这里会汇总出大家心中的最佳 11 人')
  })

  test('player hint explains the pick rate and whether I picked them', () => {
    const player = { name: '某某', pick_rate: 78 }
    expect(consensusPlayerHint(player, true)).toBe('78% 的球迷选了某某 · 你也选了')
    expect(consensusPlayerHint(player, false)).toBe('78% 的球迷选了某某')
  })

  test('players are placed into the consensus formation by group', () => {
    const placed = placeConsensus(consensus)
    expect(placed.length).toBe(11)
    expect(placed.find(entry => entry.player.player_id === 1)).toMatchObject({ x: 0.5, y: 0.9 })
    expect(placed.filter(entry => entry.player.group === 'FW').every(entry => entry.y < 0.3)).toBe(true)
    expect(placeConsensus(locked)).toEqual([])
  })
})
