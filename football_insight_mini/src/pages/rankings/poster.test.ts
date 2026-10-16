import { buildStandingsTeams } from './helpers'
import { describe, expect, test } from 'bun:test'
import type { StandingsTable, StandingsTableEntry } from '../../types/insight'
import {
  buildStandingsPosterSharePath,
  buildStandingsPosterShareTitle,
  buildStandingsFallbackMetrics,
  buildStandingsPosterColumns,
  buildStandingsPosterMetrics,
  buildStandingsRankingEntries,
  buildStandingsPosterTeamLayout,
} from './poster'

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

function createTables(): StandingsTable[] {
  return [
    {
      ...createTable('standings_with_penalty'),
      entries: [
        { ...baseEntry, team_id: 1, team_name: '成都蓉城', rank_no: 1, points: 16, points_without_penalty: 18 },
      ],
    },
    {
      ...createTable('standings_without_penalty'),
      entries: [
        { ...baseEntry, team_id: 2, team_name: '大连英博', rank_no: 1, points: 14, points_without_penalty: 22 },
      ],
    },
  ]
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

describe('buildStandingsFallbackMetrics', () => {
  test('keeps points, goals for, and goals against visible when poster generation fails', () => {
    expect(buildStandingsFallbackMetrics(createTable('standings_with_penalty'), baseEntry)).toEqual([
      { label: '积分', value: '16' },
      { label: '进球', value: '13' },
      { label: '失球', value: '5' },
    ])
  })

  test('uses theoretical points in fallback rows for standings without penalty', () => {
    expect(buildStandingsFallbackMetrics(createTable('standings_without_penalty'), baseEntry)).toEqual([
      { label: '理论积分', value: '18' },
      { label: '进球', value: '13' },
      { label: '失球', value: '5' },
    ])
  })
})

describe('standings poster sharing', () => {
  test('builds a share title from season and table label', () => {
    expect(buildStandingsPosterShareTitle(2026, createTable('standings_with_penalty'))).toBe('2026 中超含罚分版积分榜')
  })

  test('shares a path that lands on the rankings page', () => {
    expect(buildStandingsPosterSharePath()).toBe('/pages/rankings/index?mode=rankings')
  })
})

describe('buildStandingsRankingEntries', () => {
  test('uses actual points from the penalty table by default', () => {
    expect(buildStandingsRankingEntries(createTables(), 'with_penalty')).toEqual([
      {
        rank_no: 1,
        team_id: 1,
        team_name: '成都蓉城',
        score_value: '16',
        avatar_storage_url: null,
      },
    ])
  })

  test('uses theoretical points from the no-penalty table when selected', () => {
    expect(buildStandingsRankingEntries(createTables(), 'without_penalty')).toEqual([
      {
        rank_no: 1,
        team_id: 2,
        team_name: '大连英博',
        score_value: '22',
        avatar_storage_url: null,
      },
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

describe('standings teams for the shared table', () => {
  test('switches order and points while keeping each team’s recent form', () => {
    const penalized = { ...baseEntry, team_id: 10, rank_no: 2, points: 2, points_without_penalty: 7, recent_form: ['loss', 'draw', 'win'] as const }
    const leader = { ...baseEntry, team_id: 11, rank_no: 1, points: 6, points_without_penalty: 6, recent_form: ['win'] as const }
    const tables: StandingsTable[] = [
      { ...createTable('standings_with_penalty'), entries: [{ ...leader, recent_form: [...leader.recent_form] }, { ...penalized, recent_form: [...penalized.recent_form] }] },
      { ...createTable('standings_without_penalty'), entries: [{ ...penalized, rank_no: 1, recent_form: [...penalized.recent_form] }, { ...leader, rank_no: 2, recent_form: [...leader.recent_form] }] },
    ]
    const actual = buildStandingsTeams(tables, 'with_penalty')
    const theoretical = buildStandingsTeams(tables, 'without_penalty')
    expect(actual.map(team => [team.team_id, team.rank_no, team.points])).toEqual([[11, 1, 6], [10, 2, 2]])
    expect(theoretical.map(team => [team.team_id, team.rank_no, team.points])).toEqual([[10, 1, 7], [11, 2, 6]])
    expect(actual.find(team => team.team_id === 10)?.recent_form).toEqual(['loss', 'draw', 'win'])
    expect(theoretical.find(team => team.team_id === 10)?.recent_form).toEqual(['loss', 'draw', 'win'])
    expect(buildStandingsTeams([], 'with_penalty')).toEqual([])
    expect(buildStandingsTeams([createTable('standings_with_penalty')], 'with_penalty')[0]?.recent_form).toBe(undefined)
  })
})
