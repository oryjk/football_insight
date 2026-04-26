import { describe, expect, test } from 'bun:test'
import type { MembershipTierRule } from '../types/system'
import {
  buildMembershipTierGuides,
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
    expect(guides[0].condition).toBe('默认等级：未走邀请码链路时进入 V1。')
    expect(guides[0].refreshLabel).toBe('15 分钟刷新')
    expect(guides[1].code).toBe('V2')
    expect(guides[1].name).toBe('进阶会员')
    expect(guides[2].code).toBe('V3')
    expect(guides[2].name).toBe('邀请会员')
    expect(guides[2].condition).toBe('升级条件：累计推荐 4 人。')
    expect(guides[2].refreshLabel).toBe('3 分钟刷新')
    expect(guides[3].code).toBe('V5')
    expect(guides[3].name).toBe('推荐会员')
    expect(guides[3].condition).toBe('升级条件：累计推荐 30 人。')
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
