import { describe, expect, test } from 'bun:test'
import type {
  MatchCard,
  OverviewMatch,
  OverviewPlayer,
  OverviewStanding,
  RankingsViewResponse,
  RoundReference,
} from '../../types/insight'
import {
  formatHomePulseCornerText,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  resolveHomeGuideNote,
  resolveHomeAiEntryTapResult,
  resolveHomeGuideLeaders,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomeHasAuthToken,
  resolveHomeSupportNextMatchLabel,
  resolveHomeSupportWindowShortLabel,
} from './helpers'

function createLiveMatch(overrides: Partial<MatchCard> = {}): MatchCard {
  return {
    match_id: 1,
    round_number: 5,
    match_date: '2026-04-17',
    match_time: '19:35',
    status: 'live',
    home_team_id: 1,
    home_team_name: '河南队',
    home_score: '1',
    away_team_id: 2,
    away_team_name: '山东泰山',
    away_score: '0',
    home_team_avatar: null,
    away_team_avatar: null,
    leisu_match_id: null,
    home_corners: null,
    away_corners: null,
    corner_source: null,
    technical_stats: [],
    ...overrides,
  }
}

function createRecentMatch(overrides: Partial<OverviewMatch> = {}): OverviewMatch {
  return {
    match_id: 11,
    round_number: 5,
    match_date: '2026-04-16',
    match_time: '19:35',
    home_team_name: '上海申花',
    home_score: '2',
    away_team_name: '上海海港',
    away_score: '1',
    ...overrides,
  }
}

function createRoundReference(overrides: Partial<RoundReference> = {}): RoundReference {
  return {
    season: 2026,
    round_number: 5,
    finalized_at: null,
    status: 'current',
    total_matches: 8,
    completed_matches: 4,
    ...overrides,
  }
}

function createStanding(overrides: Partial<OverviewStanding> = {}): OverviewStanding {
  return {
    rank_no: 1,
    team_id: 1,
    team_name: '河南队',
    points: 12,
    avatar_storage_url: null,
    ...overrides,
  }
}

function createScorer(overrides: Partial<OverviewPlayer> = {}): OverviewPlayer {
  return {
    rank_no: 1,
    player_id: 1,
    player_name: '卡多索',
    team_name: '河南队',
    score_value: '5',
    avatar_storage_url: null,
    ...overrides,
  }
}

function createReferenceRankings(): RankingsViewResponse {
  return {
    view_kind: 'round',
    round_number: 4,
    current_season: 2026,
    standings_tables: [{
      slug: 'standings',
      label: '积分榜',
      note: '',
      entries: [{
        rank_no: 1,
        team_id: 9,
        team_name: '成都蓉城',
        played: 4,
        wins: 4,
        draws: 0,
        losses: 0,
        goals_for: 10,
        goals_against: 2,
        goal_difference: 8,
        points: 12,
        points_without_penalty: 12,
        points_adjustment: 0,
        avatar_storage_url: null,
      }],
    }],
    team_categories: [],
    player_categories: [{
      slug: 'goals',
      label: '射手榜',
      item_id: 1,
      entries: [{
        rank_no: 1,
        player_id: 99,
        player_name: '费利佩',
        team_id: 9,
        team_name: '成都蓉城',
        score_value: '6',
        penalty_value: null,
        avatar_storage_url: null,
      }],
    }],
  }
}

describe('resolveHomeHasAuthToken', () => {
  test('treats non-empty access tokens as logged in', () => {
    expect(resolveHomeHasAuthToken('token')).toBe(true)
    expect(resolveHomeHasAuthToken(' token ')).toBe(true)
  })

  test('treats empty access tokens as logged out', () => {
    expect(resolveHomeHasAuthToken('')).toBe(false)
    expect(resolveHomeHasAuthToken('   ')).toBe(false)
    expect(resolveHomeHasAuthToken(null)).toBe(false)
  })
})

describe('resolveHomeAiEntryTapResult', () => {
  test('expands the compact ai entry on the first tap', () => {
    expect(resolveHomeAiEntryTapResult({ expanded: false, hasAuthToken: true })).toBe('expand')
    expect(resolveHomeAiEntryTapResult({ expanded: false, hasAuthToken: false })).toBe('expand')
  })

  test('opens chat after expansion when the user is logged in', () => {
    expect(resolveHomeAiEntryTapResult({ expanded: true, hasAuthToken: true })).toBe('open-chat')
  })

  test('prompts login after expansion when the user is logged out', () => {
    expect(resolveHomeAiEntryTapResult({ expanded: true, hasAuthToken: false })).toBe('prompt-login')
  })
})

describe('resolveHomeSupportWindowShortLabel', () => {
  test('does not expose voting window restrictions for upcoming matches on the home card', () => {
    expect(resolveHomeSupportWindowShortLabel({ support_window_status: 'locked' })).toBe('VS')
    expect(resolveHomeSupportWindowShortLabel({ support_window_status: 'open' })).toBe('VS')
  })
})

describe('resolveHomeSupportNextMatchLabel', () => {
  test('uses entry guidance instead of vote-window timing for upcoming matches on the home card', () => {
    expect(resolveHomeSupportNextMatchLabel({ support_window_status: 'locked', status: 'scheduled' })).toBe('随时进入助力页')
    expect(resolveHomeSupportNextMatchLabel({ support_window_status: 'open', status: 'scheduled' })).toBe('随时进入助力页')
  })

  test('keeps finished or started copy when the match is already closed', () => {
    expect(resolveHomeSupportNextMatchLabel({ support_window_status: 'closed', status: 'finished' })).toBe('比赛已完赛')
    expect(resolveHomeSupportNextMatchLabel({ support_window_status: 'closed', status: 'live' })).toBe('比赛已开始')
  })
})

describe('resolveHomePulseLeadMatch', () => {
  test('prefers a live match for the realtime pulse area', () => {
    const match = resolveHomePulseLeadMatch(
      [createLiveMatch(), createLiveMatch({ match_id: 2, status: 'scheduled' })],
      [createRecentMatch()],
    )

    expect(match).toEqual({
      match_id: 1,
      round_number: 5,
      match_date: '2026-04-17',
      match_time: '19:35',
      home_team_name: '河南队',
      home_score: '1',
      away_team_name: '山东泰山',
      away_score: '0',
      technical_stats: [],
      home_corners: null,
      away_corners: null,
      status: 'live',
    })
  })

  test('falls back to the latest recent finished match when there is no live match', () => {
    const match = resolveHomePulseLeadMatch(
      [createLiveMatch({ status: 'scheduled' })],
      [createRecentMatch()],
      '2026-04-17T18:00:00+08:00',
    )

    expect(match).toEqual({
      match_id: 11,
      round_number: 5,
      match_date: '2026-04-16',
      match_time: '19:35',
      home_team_name: '上海申花',
      home_score: '2',
      away_team_name: '上海海港',
      away_score: '1',
      technical_stats: [],
      home_corners: null,
      away_corners: null,
      status: 'finished',
    })
  })
})

describe('resolveHomePulseMatches', () => {
  test('returns all live matches for the realtime pulse area', () => {
    const matches = resolveHomePulseMatches(
      [
        createLiveMatch(),
        createLiveMatch({
          match_id: 3,
          home_team_name: '上海申花',
          away_team_name: '上海海港',
          home_score: '2',
          away_score: '2',
        }),
      ],
      [createRecentMatch()],
    )

    expect(matches.length).toBe(2)
    expect(matches.map((match) => match.match_id)).toEqual([1, 3])
    expect(matches.every((match) => match.status === 'live')).toBe(true)
  })

  test('treats started matches as live even before the backend status flips to live', () => {
    const matches = resolveHomePulseMatches(
      [
        createLiveMatch({
          match_id: 21,
          status: 'scheduled',
          match_date: '2026-04-21',
          match_time: '19:35',
          home_team_name: '北京国安',
          away_team_name: '上海海港',
        }),
      ],
      [createRecentMatch()],
      '2026-04-21T20:05:00+08:00',
    )

    expect(matches.length).toBe(1)
    expect(matches[0]?.match_id).toBe(21)
    expect(matches[0]?.status).toBe('live')
  })

  test('falls back to a single recent finished match when there are no live matches', () => {
    const matches = resolveHomePulseMatches(
      [
        createLiveMatch({ status: 'scheduled' }),
        createLiveMatch({
          match_id: 11,
          status: 'finished',
          home_team_name: '上海申花',
          away_team_name: '上海海港',
          home_score: '2',
          away_score: '1',
          home_corners: 4,
          away_corners: 8,
          corner_source: 'leisu_detail',
          technical_stats: [
            { slug: 'attacks', label: '进攻', home_value: 92, away_value: 118, unit: null },
            { slug: 'corners', label: '角球', home_value: 4, away_value: 8, unit: null },
          ],
        }),
      ],
      [
        createRecentMatch(),
        createRecentMatch({ match_id: 12, home_team_name: '北京国安', away_team_name: '成都蓉城' }),
        createRecentMatch({ match_id: 13, home_team_name: '天津津门虎', away_team_name: '青岛海牛' }),
        createRecentMatch({ match_id: 14, home_team_name: '浙江队', away_team_name: '武汉三镇' }),
      ],
      '2026-04-17T18:00:00+08:00',
    )

    expect(matches.length).toBe(3)
    expect(matches[0]?.status).toBe('finished')
    expect(matches[0]?.match_id).toBe(11)
    expect(matches[0]?.home_corners).toBe(4)
    expect(matches[0]?.away_corners).toBe(8)
    expect(matches[0]?.technical_stats).toEqual([
      { slug: 'attacks', label: '进攻', home_value: 92, away_value: 118, unit: null },
      { slug: 'corners', label: '角球', home_value: 4, away_value: 8, unit: null },
    ])
    expect(matches[1]?.match_id).toBe(12)
    expect(matches[2]?.match_id).toBe(13)
  })
})

describe('formatHomePulseCornerText', () => {
  test('shows corner text when both sides have corner counts', () => {
    expect(formatHomePulseCornerText({
      home_corners: 4,
      away_corners: 8,
    })).toBe('角球 4 : 8')
  })

  test('hides corner text when either side is missing', () => {
    expect(formatHomePulseCornerText({
      home_corners: 4,
      away_corners: null,
    })).toBe('')
  })
})

describe('resolveHomePulseTechStats', () => {
  test('builds a centered corner stat row when both teams have corner counts', () => {
    expect(resolveHomePulseTechStats({
      technical_stats: [],
      home_corners: 6,
      away_corners: 1,
    })).toEqual([{
      key: 'corners',
      label: '角球',
      homeValue: '6',
      awayValue: '1',
      homeBarPercent: 100,
      awayBarPercent: 17,
    }])
  })

  test('returns no technical stats when the corner counts are missing', () => {
    expect(resolveHomePulseTechStats({
      technical_stats: [],
      home_corners: 6,
      away_corners: null,
    })).toEqual([])
  })

  test('prefers full technical stats when the backend already returns them', () => {
    expect(resolveHomePulseTechStats({
      technical_stats: [
        { slug: 'attacks', label: '进攻', home_value: 101, away_value: 73, unit: null },
        { slug: 'possession', label: '控球率', home_value: 58, away_value: 42, unit: '%' },
        { slug: 'corners', label: '角球', home_value: 6, away_value: 3, unit: null },
      ],
      home_corners: 6,
      away_corners: 3,
    })).toEqual([
      {
        key: 'attacks',
        label: '进攻',
        homeValue: '101',
        awayValue: '73',
        homeBarPercent: 100,
        awayBarPercent: 72,
      },
      {
        key: 'possession',
        label: '控球率',
        homeValue: '58%',
        awayValue: '42%',
        homeBarPercent: 100,
        awayBarPercent: 72,
      },
      {
        key: 'corners',
        label: '角球',
        homeValue: '6',
        awayValue: '3',
        homeBarPercent: 100,
        awayBarPercent: 50,
      },
    ])
  })

  test('shows empty bars when both sides have zero corners', () => {
    expect(resolveHomePulseTechStats({
      technical_stats: [],
      home_corners: 0,
      away_corners: 0,
    })).toEqual([{
      key: 'corners',
      label: '角球',
      homeValue: '0',
      awayValue: '0',
      homeBarPercent: 0,
      awayBarPercent: 0,
    }])
  })
})

describe('resolveHomeGuideLeaders', () => {
  test('uses the latest completed round leaders when the current round is still in progress', () => {
    const leaders = resolveHomeGuideLeaders({
      rounds: [
        createRoundReference({ round_number: 4, status: 'completed', completed_matches: 8, total_matches: 8 }),
        createRoundReference({ round_number: 5, status: 'current', completed_matches: 4, total_matches: 8 }),
      ],
      liveStandings: [createStanding({ team_name: '上海申花', points: 15 })],
      liveScorers: [createScorer({ player_name: '路易斯', score_value: '7' })],
      referenceRankings: createReferenceRankings(),
    })

    expect(leaders.topTeamNames).toEqual(['成都蓉城'])
    expect(leaders.topScorerNames).toEqual(['费利佩'])
    expect(leaders.source).toBe('previous_round')
  })

  test('uses live leaders when there is no in-progress current round', () => {
    const leaders = resolveHomeGuideLeaders({
      rounds: [
        createRoundReference({ round_number: 4, status: 'completed', completed_matches: 8, total_matches: 8 }),
        createRoundReference({ round_number: 5, status: 'completed', completed_matches: 8, total_matches: 8 }),
      ],
      liveStandings: [createStanding({ team_name: '上海申花', points: 15 })],
      liveScorers: [createScorer({ player_name: '路易斯', score_value: '7' })],
      referenceRankings: createReferenceRankings(),
    })

    expect(leaders.topTeamNames).toEqual(['上海申花'])
    expect(leaders.topScorerNames).toEqual(['路易斯'])
    expect(leaders.source).toBe('live')
  })
})

describe('resolveHomeGuideNote', () => {
  test('shows the previous-round note when the current round is still in progress', () => {
    expect(resolveHomeGuideNote('previous_round')).toBe('当前轮次尚未全部结束，榜首和射手判断先按上一轮结清结果来看。')
  })

  test('hides the note once the current round has finished', () => {
    expect(resolveHomeGuideNote('live')).toBe('')
  })
})

describe('resolveHomeTeamSeasonMatches', () => {
  test('shows finished matches first and sorts each group by kickoff ascending', () => {
    const matches = resolveHomeTeamSeasonMatches(
      { team_id: 9, team_name: '成都蓉城' },
      [
        createLiveMatch({
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
        createLiveMatch({
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
        createLiveMatch({
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
        createLiveMatch({
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
    expect({
      resultLabel: matches[0]?.resultLabel,
      resultTone: matches[0]?.resultTone,
      scoreText: matches[0]?.scoreText,
      opponentName: matches[0]?.opponentName,
    }).toEqual({
      resultLabel: '胜',
      resultTone: 'win',
      scoreText: '2 : 1',
      opponentName: '河南队',
    })
    expect({
      resultLabel: matches[1]?.resultLabel,
      resultTone: matches[1]?.resultTone,
      opponentName: matches[1]?.opponentName,
    }).toEqual({
      resultLabel: '进行中',
      resultTone: 'live',
      opponentName: '云南玉昆',
    })
    expect({
      resultLabel: matches[2]?.resultLabel,
      resultTone: matches[2]?.resultTone,
      scoreText: matches[2]?.scoreText,
      opponentName: matches[2]?.opponentName,
    }).toEqual({
      resultLabel: '未开赛',
      resultTone: 'scheduled',
      scoreText: 'VS',
      opponentName: '浙江队',
    })
  })

  test('uses team name fallback and returns draw or loss labels for finished matches', () => {
    const matches = resolveHomeTeamSeasonMatches(
      { team_id: 9, team_name: '成都蓉城' },
      [
        createLiveMatch({
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
        createLiveMatch({
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

    expect({
      matchId: matches[0]?.matchId,
      resultLabel: matches[0]?.resultLabel,
      resultTone: matches[0]?.resultTone,
    }).toEqual({
      matchId: 72,
      resultLabel: '负',
      resultTone: 'loss',
    })
    expect({
      matchId: matches[1]?.matchId,
      resultLabel: matches[1]?.resultLabel,
      resultTone: matches[1]?.resultTone,
    }).toEqual({
      matchId: 71,
      resultLabel: '平',
      resultTone: 'draw',
    })
  })
})
