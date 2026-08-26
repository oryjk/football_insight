import { describe, expect, test } from 'bun:test'

import {
  buildAssistContributionRows,
  buildOpponentContributionRows,
  buildPlayerContributionRows,
  formatInsightShare,
  getVisibleInsightContributions,
  insightBarWidth,
  shouldShowInsightToggle,
} from './helpers'

describe('insights contribution helpers', () => {
  test('visible contributions cap at 3 unless expanded', () => {
    const items = [1, 2, 3, 4, 5]
    expect(getVisibleInsightContributions(items, false)).toEqual([1, 2, 3])
    expect(getVisibleInsightContributions(items, true)).toEqual(items)
    expect(getVisibleInsightContributions([1, 2], false)).toEqual([1, 2])
  })

  test('toggle only shows when more than 3 rows', () => {
    expect(shouldShowInsightToggle([1, 2, 3])).toBe(false)
    expect(shouldShowInsightToggle([1, 2, 3, 4])).toBe(true)
  })

  test('formats share as one-decimal percent', () => {
    expect(formatInsightShare(0.5)).toBe('50.0%')
    expect(formatInsightShare(0.1234)).toBe('12.3%')
  })

  test('bar width clamps to 8% minimum for positive shares', () => {
    expect(insightBarWidth(0)).toBe('0%')
    expect(insightBarWidth(-0.2)).toBe('0%')
    expect(insightBarWidth(0.01)).toBe('8%')
    expect(insightBarWidth(0.5)).toBe('50%')
  })

  test('builds opponent rows with team-scoped instance keys', () => {
    const rows = buildOpponentContributionRows('goals-for-opponent', 7, [
      {
        opponent_team_id: 11,
        opponent_team_name: '青岛西海岸',
        opponent_avatar_storage_url: 'a.png',
        goals: 3,
        share: 0.3,
      },
    ], 'ink')

    expect(rows[0].name).toBe('青岛西海岸')
    expect(rows[0].note).toBe('3 球')
    expect(rows[0].avatarMode).toBe('fit')
    expect(rows[0].variant).toBe('ink')
    expect(rows[0].key).toBe('team-11')
    expect(rows[0].instanceKey).toBe('7-goals-for-opponent-team-11')
  })

  test('player rows fall back to name key without player_id', () => {
    const rows = buildPlayerContributionRows('scope', null, [
      { player_id: null, player_name: '费利佩', avatar_storage_url: null, goals: 2, share: 0.2 },
    ], 'red')

    expect(rows[0].key).toBe('player-费利佩')
    expect(rows[0].instanceKey).toBe('none-scope-player-费利佩')
    expect(rows[0].variant).toBe('red')
  })

  test('assist rows count in assists unit with green variant', () => {
    const rows = buildAssistContributionRows('assists', 5, [
      { player_id: 3, player_name: '席尔瓦', avatar_storage_url: null, assists: 4, share: 0.4 },
    ])

    expect(rows[0].note).toBe('4 次')
    expect(rows[0].variant).toBe('green')
    expect(rows[0].instanceKey).toBe('5-assists-assist-3')
  })
})
