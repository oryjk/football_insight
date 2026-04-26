import { describe, expect, test } from 'bun:test'

import type { MatchCard } from '../types/insight'
import { resolveTeamSeasonMatches } from './teamSeasonMatches'

function createMatch(overrides: Partial<MatchCard> = {}): MatchCard {
  return {
    match_id: overrides.match_id ?? 1,
    round_number: overrides.round_number ?? 1,
    match_date: overrides.match_date ?? '2026-04-21',
    match_time: overrides.match_time ?? '19:35',
    status: overrides.status ?? 'scheduled',
    home_team_id: overrides.home_team_id ?? 9,
    home_team_name: overrides.home_team_name ?? '成都蓉城',
    home_score: overrides.home_score ?? '',
    away_team_id: overrides.away_team_id ?? 10,
    away_team_name: overrides.away_team_name ?? '浙江队',
    away_score: overrides.away_score ?? '',
    home_team_avatar: overrides.home_team_avatar ?? null,
    away_team_avatar: overrides.away_team_avatar ?? null,
    leisu_match_id: overrides.leisu_match_id ?? null,
    home_corners: overrides.home_corners ?? null,
    away_corners: overrides.away_corners ?? null,
    corner_source: overrides.corner_source ?? null,
    technical_stats: overrides.technical_stats ?? [],
  }
}

describe('resolveTeamSeasonMatches', () => {
  test('filters one team and shows finished, live, then scheduled matches', () => {
    const matches = resolveTeamSeasonMatches(
      { team_id: 9, team_name: '成都蓉城' },
      [
        createMatch({
          match_id: 61,
          round_number: 6,
          match_date: '2026-04-18',
          match_time: '19:00',
          status: 'finished',
          home_team_id: 9,
          home_team_name: '成都蓉城',
          home_score: '2',
          away_team_id: 3,
          away_team_name: '河南队',
          away_score: '1',
        }),
        createMatch({
          match_id: 62,
          round_number: 7,
          match_date: '2026-04-21',
          match_time: '19:35',
          status: 'live',
          home_team_id: 4,
          home_team_name: '云南玉昆',
          home_score: '0',
          away_team_id: 9,
          away_team_name: '成都蓉城',
          away_score: '0',
        }),
        createMatch({
          match_id: 63,
          round_number: 8,
          match_date: '2026-04-25',
          match_time: '20:00',
          status: 'scheduled',
          home_team_id: 9,
          home_team_name: '成都蓉城',
          home_score: '',
          away_team_id: 10,
          away_team_name: '浙江队',
          away_score: '',
        }),
        createMatch({
          match_id: 64,
          round_number: 7,
          match_date: '2026-04-21',
          match_time: '19:35',
          status: 'live',
          home_team_id: 1,
          home_team_name: '河南队',
          home_score: '1',
          away_team_id: 2,
          away_team_name: '山东泰山',
          away_score: '0',
        }),
      ],
      '2026-04-21T20:00:00+08:00',
    )

    expect(matches.map((match) => match.matchId)).toEqual([61, 62, 63])
    expect(matches.map((match) => match.resultLabel)).toEqual(['胜', '进行中', '未开赛'])
    expect(matches.map((match) => match.opponentName)).toEqual(['河南队', '云南玉昆', '浙江队'])
  })

  test('uses team name fallback and derives draw or loss for finished matches', () => {
    const matches = resolveTeamSeasonMatches(
      { team_id: 9, team_name: '成都蓉城' },
      [
        createMatch({
          match_id: 71,
          round_number: 3,
          match_date: '2026-03-30',
          match_time: '20:00',
          status: 'finished',
          home_team_id: 99,
          home_team_name: '成都蓉城',
          home_score: '1',
          away_team_id: 6,
          away_team_name: '上海申花',
          away_score: '1',
        }),
        createMatch({
          match_id: 72,
          round_number: 2,
          match_date: '2026-03-22',
          match_time: '19:35',
          status: 'finished',
          home_team_id: 5,
          home_team_name: '北京国安',
          home_score: '2',
          away_team_id: 9,
          away_team_name: '成都蓉城',
          away_score: '0',
        }),
      ],
      '2026-04-21T20:00:00+08:00',
    )

    expect(matches.map((match) => match.matchId)).toEqual([72, 71])
    expect(matches.map((match) => match.resultLabel)).toEqual(['负', '平'])
    expect(matches.map((match) => match.resultTone)).toEqual(['loss', 'draw'])
  })
})
