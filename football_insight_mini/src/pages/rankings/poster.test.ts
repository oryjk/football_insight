import { describe, expect, test } from 'bun:test'
import type { StandingsTable, StandingsTableEntry } from '../../types/insight'
import { buildStandingsPosterColumns, buildStandingsPosterMetrics, buildStandingsPosterTeamLayout } from './poster'

const baseEntry: StandingsTableEntry = {
  rank_no: 1,
  team_id: 1,
  team_name: '成都蓉城',
  played: 7,
  wins: 5,
  draws: 1,
  losses: 1,
  goals_for: 13,
  goals_against: 5,
  goal_difference: 8,
  points: 16,
  points_without_penalty: 18,
  points_adjustment: -2,
  avatar_storage_url: null,
}

function createTable(slug: StandingsTable['slug']): StandingsTable {
  return {
    slug,
    label: slug === 'standings_without_penalty' ? '无罚分版积分榜' : '含罚分版积分榜',
    note: 'test',
    entries: [baseEntry],
  }
}

describe('buildStandingsPosterColumns', () => {
  test('adds goals for and goals against columns', () => {
    expect(buildStandingsPosterColumns(createTable('standings_with_penalty'))).toEqual([
      { label: '排名', x: 88 },
      { label: '球队', x: 200 },
      { label: '积分', x: 818 },
      { label: '进球', x: 904 },
      { label: '失球', x: 980 },
    ])
  })

  test('uses theoretical points label for standings without penalty', () => {
    expect(buildStandingsPosterColumns(createTable('standings_without_penalty'))[2]).toEqual({
      label: '理论积分',
      x: 818,
    })
  })
})

describe('buildStandingsPosterMetrics', () => {
  test('includes points, goals for, and goals against values', () => {
    expect(buildStandingsPosterMetrics(createTable('standings_with_penalty'), baseEntry)).toEqual([
      { value: '-2', x: 760, highlight: true, compact: true },
      { value: '16', x: 818 },
      { value: '13', x: 904 },
      { value: '5', x: 980 },
    ])
  })

  test('uses theoretical points for standings without penalty', () => {
    expect(buildStandingsPosterMetrics(createTable('standings_without_penalty'), baseEntry)).toEqual([
      { value: '18', x: 818 },
      { value: '13', x: 904 },
      { value: '5', x: 980 },
    ])
  })
})

describe('buildStandingsPosterTeamLayout', () => {
  test('reserves space for logo when it exists', () => {
    expect(buildStandingsPosterTeamLayout(true)).toEqual({
      logoX: 200,
      logoSize: 28,
      nameX: 240,
    })
  })

  test('keeps team name aligned when logo is unavailable', () => {
    expect(buildStandingsPosterTeamLayout(false)).toEqual({
      logoX: 200,
      logoSize: 0,
      nameX: 200,
    })
  })
})
