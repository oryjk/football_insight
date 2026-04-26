import { describe, expect, test } from 'bun:test'

import type { MatchCard } from '../../types/insight'
import {
  buildCurrentRoundSectionTitle,
  formatMatchScoreboardText,
  hasMatchTechStats,
  shouldShowLiveStatusTag,
  resolveMatchDisplayStatus,
} from './helpers'

function createMatch(overrides: Partial<MatchCard> = {}): MatchCard {
  return {
    match_id: 1,
    round_number: 5,
    match_date: '2026-04-10',
    match_time: '19:35',
    status: 'scheduled',
    home_team_id: 1,
    home_team_name: '大连英博',
    home_score: '',
    away_team_id: 2,
    away_team_name: '浙江队',
    away_score: '',
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

describe('resolveMatchDisplayStatus', () => {
  test('treats a non-finished match as live once kickoff time has started', () => {
    const status = resolveMatchDisplayStatus(
      createMatch({
        match_date: '2026-04-10',
        match_time: '19:35',
        status: 'scheduled',
      }),
      '2026-04-10T20:10:00+08:00',
    )

    expect(status).toBe('live')
  })

  test('keeps a started non-finished match as live even after the default match window', () => {
    const status = resolveMatchDisplayStatus(
      createMatch({
        match_date: '2026-04-10',
        match_time: '19:35',
        status: 'scheduled',
      }),
      '2026-04-10T22:31:00+08:00',
    )

    expect(status).toBe('live')
  })

  test('keeps future matches as scheduled', () => {
    const status = resolveMatchDisplayStatus(
      createMatch({
        match_date: '2026-04-10',
        match_time: '19:35',
        status: 'scheduled',
      }),
      '2026-04-10T18:10:00+08:00',
    )

    expect(status).toBe('scheduled')
  })
})

describe('buildCurrentRoundSectionTitle', () => {
  test('uses current live wording when at least one current-round match is live', () => {
    const title = buildCurrentRoundSectionTitle([
      createMatch({ status: 'scheduled' }),
      createMatch({ match_id: 2, status: 'live' }),
    ])

    expect(title).toBe('本轮进行中')
  })

  test('keeps not-started wording when no current-round match is live', () => {
    const title = buildCurrentRoundSectionTitle([
      createMatch({ status: 'scheduled' }),
      createMatch({ match_id: 2, status: 'scheduled' }),
    ])

    expect(title).toBe('本轮未开赛')
  })
})

describe('formatMatchScoreboardText', () => {
  test('shows the live score when an in-progress match already has scores', () => {
    const text = formatMatchScoreboardText(
      createMatch({
        status: 'live',
        home_score: '1',
        away_score: '0',
      }),
      '2026-04-10T20:10:00+08:00',
    )

    expect(text).toBe('1 : 0')
  })

  test('keeps the status label for upcoming matches', () => {
    const text = formatMatchScoreboardText(
      createMatch({
        status: 'scheduled',
        home_score: '',
        away_score: '',
      }),
      '2026-04-10T18:10:00+08:00',
    )

    expect(text).toBe('未开赛')
  })
})

describe('shouldShowLiveStatusTag', () => {
  test('shows live status tag after kickoff when match is in progress', () => {
    const visible = shouldShowLiveStatusTag(
      createMatch({
        status: 'scheduled',
        home_score: '2',
        away_score: '1',
      }),
      '2026-04-10T20:10:00+08:00',
    )

    expect(visible).toBe(true)
  })

  test('hides live status tag for finished matches', () => {
    const visible = shouldShowLiveStatusTag(
      createMatch({
        status: 'finished',
        home_score: '2',
        away_score: '1',
      }),
      '2026-04-10T22:10:00+08:00',
    )

    expect(visible).toBe(false)
  })
})

describe('hasMatchTechStats', () => {
  test('treats explicit technical stats as available', () => {
    const available = hasMatchTechStats(
      createMatch({
        technical_stats: [
          {
            slug: 'shots_on_target',
            label: '射正',
            home_value: 5,
            away_value: 3,
            unit: null,
          },
        ],
      }),
    )

    expect(available).toBe(true)
  })

  test('treats corner data as fallback technical stats', () => {
    const available = hasMatchTechStats(
      createMatch({
        technical_stats: [],
        home_corners: 4,
        away_corners: 2,
      }),
    )

    expect(available).toBe(true)
  })

  test('returns false when no technical data exists', () => {
    const available = hasMatchTechStats(
      createMatch({
        technical_stats: [],
        home_corners: null,
        away_corners: null,
      }),
    )

    expect(available).toBe(false)
  })
})
