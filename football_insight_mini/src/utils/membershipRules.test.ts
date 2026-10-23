import { describe, expect, test } from 'bun:test'
import type { MembershipTierRule } from '../types/system'
import {
  buildInviteMilestones,
  buildMembershipLevelRows,
  buildMembershipTierGuides,
  formatInviteUpgradeHint,
  isMembershipTierAtLeast,
  normalizeTicketWatchPollIntervalSeconds,
  resolveMembershipTierGuide,
} from './membershipRules'

describe('normalizeTicketWatchPollIntervalSeconds', () => {
  test('prefers backend-provided positive intervals and falls back safely', () => {
    expect(normalizeTicketWatchPollIntervalSeconds(45)).toBe(45)
    expect(normalizeTicketWatchPollIntervalSeconds(1)).toBe(1)
    expect(normalizeTicketWatchPollIntervalSeconds(0)).toBe(600)
    expect(normalizeTicketWatchPollIntervalSeconds(-5)).toBe(600)
    expect(normalizeTicketWatchPollIntervalSeconds(undefined)).toBe(600)
  })
})

describe('isMembershipTierAtLeast', () => {
  test('checks whether a membership tier reaches a minimum tier', () => {
    expect(isMembershipTierAtLeast('V6', 'V6')).toBe(true)
    expect(isMembershipTierAtLeast('V9', 'V6')).toBe(true)
    expect(isMembershipTierAtLeast('V5', 'V6')).toBe(false)
    expect(isMembershipTierAtLeast(undefined, 'V6')).toBe(false)
  })
})

describe('buildMembershipTierGuides', () => {
  test('renders upgrade conditions and refresh labels from backend rules', () => {
    const rules: MembershipTierRule[] = [
      { code: 'V5', kind: 'referral', min_referrals: 30, ticket_watch_poll_interval_seconds: 45 },
      { code: 'V1', kind: 'standard', ticket_watch_poll_interval_seconds: 900 },
      { code: 'V2', kind: 'standard', min_referrals: 2, ticket_watch_poll_interval_seconds: 300 },
      { code: 'V3', kind: 'invite', min_referrals: 4, ticket_watch_poll_interval_seconds: 180 },
    ]

    const guides = buildMembershipTierGuides(rules)

    expect(guides.map((item) => item.code)).toEqual(['V1', 'V2', 'V3', 'V5'])
    expect(guides[0].code).toBe('V1')
    expect(guides[0].name).toBe('标准会员')
    expect(guides[0].condition).toBe('默认等级：注册时没有填邀请码的账号为 V1。')
    expect(guides[0].refreshLabel).toBe('15 分钟刷新')
    expect(guides[1].code).toBe('V2')
    expect(guides[1].name).toBe('进阶会员')
    // 后端不会按人数把用户升到 standard 档位，不能展示「推荐 2 人」。
    expect(guides[1].condition).toBe('升级条件：暂未开放。')
    expect(guides[2].code).toBe('V3')
    expect(guides[2].name).toBe('邀请会员')
    expect(guides[2].condition).toBe('升级条件：用公众号领取的邀请码注册。')
    expect(guides[2].refreshLabel).toBe('3 分钟刷新')
    expect(guides[3].code).toBe('V5')
    expect(guides[3].name).toBe('推荐会员')
    expect(guides[3].condition).toBe('升级条件：累计邀请 30 位好友注册。')
    expect(guides[3].refreshLabel).toBe('45 秒刷新')
  })
})

describe('resolveMembershipTierGuide', () => {
  test('falls back to the current user interval when the tier is absent from public config', () => {
    const guide = resolveMembershipTierGuide('V8', [], 2)

    expect(guide.code).toBe('V8')
    expect(guide.name).toBe('推荐会员')
    expect(guide.refreshLabel).toBe('2 秒刷新')
    expect(guide.condition).toBe('升级条件：以后端当前配置为准。')
  })
})

const productionLikeRules: MembershipTierRule[] = [
  { code: 'V1', kind: 'standard', ticket_watch_poll_interval_seconds: 600 },
  { code: 'V2', kind: 'standard', min_referrals: 2, ticket_watch_poll_interval_seconds: 300 },
  { code: 'V3', kind: 'invite', min_referrals: 4, ticket_watch_poll_interval_seconds: 120 },
  { code: 'V6', kind: 'referral', min_referrals: 20, ticket_watch_poll_interval_seconds: 15 },
  { code: 'V4', kind: 'referral', min_referrals: 10, ticket_watch_poll_interval_seconds: 60 },
  { code: 'V9', kind: 'referral', min_referrals: 80, ticket_watch_poll_interval_seconds: 1 },
]

describe('buildMembershipLevelRows', () => {
  test('lists how each tier is obtained and hides tiers with no path', () => {
    const rows = buildMembershipLevelRows(productionLikeRules, ['V6', 'v9'])
    expect(rows.map(row => [row.code, row.ways, row.refreshLabel])).toEqual([
      ['V1', '注册即有', '10 分钟刷新'],
      ['V3', '用邀请码注册', '2 分钟刷新'],
      ['V4', '邀请 10 人', '1 分钟刷新'],
      ['V6', '邀请 20 人 / 开通', '15 秒刷新'],
      ['V9', '邀请 80 人 / 开通', '1 秒刷新'],
    ])
    expect(buildMembershipLevelRows(null, ['V6'])).toEqual([])
  })
})

describe('buildInviteMilestones', () => {
  test('only referral tiers count, sorted by required invites', () => {
    expect(buildInviteMilestones(productionLikeRules)).toEqual([
      { code: 'V4', referrals: 10, refreshLabel: '1 分钟刷新' },
      { code: 'V6', referrals: 20, refreshLabel: '15 秒刷新' },
      { code: 'V9', referrals: 80, refreshLabel: '1 秒刷新' },
    ])
    expect(buildInviteMilestones(undefined)).toEqual([])
  })
})

describe('formatInviteUpgradeHint', () => {
  test('tells the friend which field takes the invite code', () => {
    expect(formatInviteUpgradeHint(productionLikeRules, 'V6'))
      .toBe('累计邀请 20 位好友注册，就能免费升到 V6。好友第一次登录时在「推荐码」一栏填你的邀请码即可。')
  })
})
