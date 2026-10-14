import { expect, test } from 'bun:test'
import { buildGoalMatchDetail, goalChartScale, normalizeRecentGoals } from './playerGoals'
import type { PlayerMatchGoals } from '../types/insight'

const appearance = (goals: number, matchId = 1): PlayerMatchGoals => ({
  goals, match_id: matchId, round_number: matchId, match_date: '2026-09-01',
  match_time: '19:35', home_team_name: '主队', away_team_name: '客队', minutes_played: 90,
})

test('retains genuine zero goals, refuses missing/invalid values and absent appearances', () => {
  expect(normalizeRecentGoals(undefined)).toEqual([])
  const rows = [appearance(0), appearance(3, 2), appearance(-1, 3), appearance(1.5, 4),
    { ...appearance(0, 5), goals: null }, { ...appearance(0, 6), minutes_played: 0 }]
  expect(normalizeRecentGoals(rows as PlayerMatchGoals[]).map(r => r.goals)).toEqual([0, 3])
})

test('keeps the latest ten actual appearances in stable chronological order', () => {
  const rows = Array.from({ length: 12 }, (_, i) => appearance(1, 12 - i))
  expect(normalizeRecentGoals(rows).map(r => r.match_id)).toEqual([3,4,5,6,7,8,9,10,11,12])
})

test('all rows share one scale and a four-goal match is taller than three', () => {
  expect(goalChartScale([[appearance(1)], [appearance(4)]])).toBe(4)
  expect(goalChartScale([[appearance(0)], []])).toBe(3)
})

test('details identify exactly three goals as a hat trick while preserving real scores', () => {
  expect(buildGoalMatchDetail('球员', appearance(3)).title.includes('帽子戏法')).toBe(true)
  expect(buildGoalMatchDetail('球员', appearance(4)).title.includes('4 球')).toBe(true)
  expect(buildGoalMatchDetail('球员', appearance(4)).title.includes('帽子戏法')).toBe(false)
  expect(buildGoalMatchDetail('球员', appearance(0)).content.includes('90 分钟')).toBe(true)
})
