import { describe, expect, test } from 'bun:test'
import {
  resolveSupportCountdownSeconds,
  resolveSupportCountdownLabel,
  resolveSupportHeroBadge,
  resolveSupportHeroSummary,
  resolveSupportStanceDescription,
  resolveSupportVoteButtonText,
  resolveSupportWindowLabel,
  resolveSupportedTeam,
} from './helpers'

describe('support page voting window copy', () => {
  test('treats locked matches as already open before kickoff', () => {
    const detail = {
      support_window_status: 'locked' as const,
      status: 'scheduled',
      countdown_seconds: 166560,
      viewer: {
        has_supported: false,
      },
    }

    expect(resolveSupportHeroBadge(detail)).toBe('助力进行中')
    expect(resolveSupportHeroSummary(detail)).toBe('任选主队或客队助力，并把页面转发给朋友继续拉票。')
    expect(resolveSupportWindowLabel(detail)).toBe('助力开放')
    expect(resolveSupportStanceDescription(detail as never, { team_name: '成都蓉城' })).toBe('主队和客队均可助力，每场比赛只能选择一队投一次。')
    expect(resolveSupportVoteButtonText(detail as never, { team_name: '成都蓉城' }, true)).toBe('为 成都蓉城 助力')
    expect(resolveSupportCountdownLabel(detail)).toBe('距开赛 46 小时 16 分 0 秒')
  })

  test('助力结果按实际投票球队展示，不按关注球队推断', () => {
    const home = { team_id: 1, team_name: '主队' }
    const away = { team_id: 2, team_name: '客队' }
    const detail = { home_team: home, away_team: away,
      viewer: { has_supported: true, favorite_team_id: 1, supported_team_id: 2 } }
    expect(resolveSupportedTeam(detail as never)).toEqual(away)
    expect(resolveSupportedTeam({ ...detail, viewer: { ...detail.viewer, has_supported: false } } as never)).toBeNull()
  })

  test('formats countdown to second precision', () => {
    expect(resolveSupportCountdownLabel({
      support_window_status: 'open',
      status: 'scheduled',
      countdown_seconds: 3665,
    })).toBe('距开赛 1 小时 1 分 5 秒')
  })

  test('computes remaining seconds from kickoff timestamp', () => {
    expect(resolveSupportCountdownSeconds('2026-04-12T11:35:00+00:00', Date.parse('2026-04-12T10:34:30+00:00'))).toBe(3630)
  })
})
