import { describe, expect, test } from 'bun:test'
import { ApiRequestError } from '../../utils/apiError'
import type {
  MatchCard,
  OverviewMatch,
  OverviewPlayer,
  OverviewStanding,
  RankingsViewResponse,
  RoundReference,
} from '../../types/insight'
import {
  buildHomeBriefingItems,
  buildHomeHeadlineBody,
  buildHomeHeroGuide,
  buildHomeWatchPoints,
  formatHomeTeamSeasonRecord,
  formatHomePulseCornerText,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  resolveHomeGuideNote,
  resolveHomeGuideLeaders,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomeHasAuthToken,
  resolveHomeSupportNextMatchLabel,
  resolveHomeSupportTeamRankLabel,
  resolveHomeSupportMatchWeekdayLabel,
  resolveHomeSupportWindowShortLabel,
  shouldShowHomeSupportLoading,
  isHomeAuthExpiredError,
  resolveHomeLoadPlan,
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
    technical_stats: [],
    home_corners: null,
    away_corners: null,
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

describe('shouldShowHomeSupportLoading', () => {
  test('shows the blocking support loader only before any profile is available', () => {
    expect(shouldShowHomeSupportLoading({
      loading: true,
      hasCachedProfile: false,
    })).toBe(true)
  })

  test('keeps the current support panel visible while refreshing cached data', () => {
    expect(shouldShowHomeSupportLoading({
      loading: true,
      hasCachedProfile: true,
    })).toBe(false)
  })
})

describe('resolveHomeLoadPlan', () => {
  test('keeps only overview on the first-paint critical path for logged-out users', () => {
    expect(resolveHomeLoadPlan(false)).toEqual({
      critical: ['overview'],
      deferred: ['rankings', 'matches', 'rounds', 'public-config'],
      onDemand: ['auth-me', 'support-teams'],
    })
  })

  test('loads the favorite-team profile after first paint for logged-in users', () => {
    expect(resolveHomeLoadPlan(true)).toEqual({
      critical: ['overview'],
      deferred: ['rankings', 'matches', 'rounds', 'public-config', 'support-profile', 'support-teams'],
      onDemand: ['auth-me'],
    })
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

describe('resolveHomeSupportTeamRankLabel', () => {
  test('formats league ranks for the favorite-team next-match card', () => {
    expect(resolveHomeSupportTeamRankLabel(1)).toBe('积分榜第 1')
    expect(resolveHomeSupportTeamRankLabel(8)).toBe('积分榜第 8')
    expect(resolveHomeSupportTeamRankLabel(null)).toBe('排名待同步')
    expect(resolveHomeSupportTeamRankLabel(0)).toBe('排名待同步')
  })
})

describe('resolveHomeSupportMatchWeekdayLabel', () => {
  test('formats fixture dates as fan-friendly weekdays', () => {
    expect(resolveHomeSupportMatchWeekdayLabel('2026-05-30')).toBe('周六')
    expect(resolveHomeSupportMatchWeekdayLabel('2026-05-27')).toBe('周三')
  })

  test('returns empty text for missing or invalid dates', () => {
    expect(resolveHomeSupportMatchWeekdayLabel(null)).toBe('')
    expect(resolveHomeSupportMatchWeekdayLabel('')).toBe('')
    expect(resolveHomeSupportMatchWeekdayLabel('not-a-date')).toBe('')
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
      resultLabel: '进',
      resultTone: 'live',
      opponentName: '云南玉昆',
    })
    expect({
      resultLabel: matches[2]?.resultLabel,
      resultTone: matches[2]?.resultTone,
      scoreText: matches[2]?.scoreText,
      opponentName: matches[2]?.opponentName,
    }).toEqual({
      resultLabel: '未',
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

describe('buildHomeHeroGuide', () => {
  const leaders = { topTeamNames: ['上海海港'], topScorerNames: ['武磊'], source: 'live' as const }

  test('combines team, scorer and finished lead match', () => {
    const guide = buildHomeHeroGuide({
      guideLeaders: leaders,
      leadMatch: { status: 'finished' },
    })

    expect(guide).toEqual({
      mode: 'team-and-scorer-with-match',
      topTeamName: '上海海港',
      topScorerName: '武磊',
    })
  })

  test('marks live lead match explicitly', () => {
    const guide = buildHomeHeroGuide({
      guideLeaders: leaders,
      leadMatch: { status: 'live' },
    })

    expect(guide.mode).toBe('team-and-scorer-with-live-match')
  })

  test('falls back to team-and-scorer without lead match', () => {
    const guide = buildHomeHeroGuide({ guideLeaders: leaders, leadMatch: null })

    expect(guide.mode).toBe('team-and-scorer')
  })

  test('falls back to generic copy when leaders missing', () => {
    const guide = buildHomeHeroGuide({
      guideLeaders: { topTeamNames: [], topScorerNames: [], source: 'live' },
      leadMatch: { status: 'finished' },
    })

    expect(guide.mode).toBe('fallback')
  })
})

describe('buildHomeBriefingItems', () => {
  test('builds leader/scorer/assist items and skips missing ones', () => {
    const items = buildHomeBriefingItems({
      topTeam: { team_name: '上海海港', points: 30 },
      leadingTeams: [{ team_name: '上海海港', points: 30, avatar_storage_url: 'a.png' }],
      topScorer: { player_name: '武磊', score_value: '12' },
      leadingScorers: [
        { player_name: '武磊', team_name: '上海海港', score_value: '12', avatar_storage_url: 'b.png' },
        { player_name: '队友', team_name: '上海海港', score_value: '12', avatar_storage_url: 'c.png' },
      ],
      topAssist: null,
      leadingAssists: [],
    })

    expect(items.map(item => item.accent)).toEqual(['leader', 'scorer'])
    expect(items[0].metricLabel).toBe('分领跑')
    expect(items[1].metricLabel).toBe('球并列头名')
    expect(items[1].entities.length).toBe(2)
  })
})

describe('buildHomeHeadlineBody', () => {
  test('prefers backend summary', () => {
    expect(buildHomeHeadlineBody({ summary: '后端摘要', topScorer: null })).toBe('后端摘要')
  })

  test('derives copy from top scorer', () => {
    expect(buildHomeHeadlineBody({ summary: null, topScorer: { player_name: '武磊' } }).includes('武磊')).toBe(true)
  })

  test('falls back to generic copy', () => {
    expect(buildHomeHeadlineBody({ summary: null, topScorer: null }).includes('积分榜头部')).toBe(true)
  })
})

describe('buildHomeWatchPoints', () => {
  test('prefers backend bullets', () => {
    expect(buildHomeWatchPoints({ bullets: ['a'], leadMatch: null, topTeam: null, topScorer: null, secondScorer: null }))
      .toEqual(['a'])
  })

  test('derives points from lead match and standings', () => {
    const items = buildHomeWatchPoints({
      bullets: [],
      leadMatch: { status: 'live', home_team_name: '甲', away_team_name: '乙' },
      topTeam: { team_name: '上海海港' },
      topScorer: { player_name: '武磊' },
      secondScorer: { player_name: '对手' },
    })

    expect(items.length).toBe(3)
    expect(items[0].includes('进行中焦点')).toBe(true)
  })
})

describe('formatHomeTeamSeasonRecord', () => {
  test('counts only finished results', () => {
    const record = formatHomeTeamSeasonRecord([
      { resultTone: 'win' },
      { resultTone: 'win' },
      { resultTone: 'draw' },
      { resultTone: 'loss' },
      { resultTone: 'live' },
      { resultTone: 'scheduled' },
    ])

    expect(record).toBe('2胜 1平 1负')
  })
})


describe('isHomeAuthExpiredError', () => {
  test('treats only HTTP 401 ApiRequestError as expired session', () => {
    expect(isHomeAuthExpiredError(new ApiRequestError('未登录', 401))).toBe(true)
    expect(isHomeAuthExpiredError(new ApiRequestError('登录后才能查看', 403))).toBe(false)
    expect(isHomeAuthExpiredError(new Error('请先登录'))).toBe(false)
    expect(isHomeAuthExpiredError(null)).toBe(false)
  })
})
