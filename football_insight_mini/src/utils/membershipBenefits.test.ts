import { describe, expect, test } from 'bun:test'

import { resolveMembershipBenefitsLocked, buildMembershipBenefitItems, buildMembershipBenefitPreview } from './membershipBenefits'

describe('resolveMembershipBenefitsLocked', () => {
  test('keeps benefits unlocked when the backend field is missing', () => {
    expect(resolveMembershipBenefitsLocked(null)).toBe(false)
    expect(resolveMembershipBenefitsLocked({})).toBe(false)
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: undefined })).toBe(false)
  })

  test('locks benefits only when the backend explicitly returns false', () => {
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: true })).toBe(false)
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: false })).toBe(true)
  })
})

describe('buildMembershipBenefitItems', () => {
  test('uses richer captions for higher membership tiers', () => {
    expect(buildMembershipBenefitItems({
      membershipCode: 'V9',
      refreshLabel: '1 秒刷新',
    })).toEqual([
      { key: 'watch', iconName: 'ticket', title: '余票监控', caption: '1 秒刷新' },
      { key: 'recent-reflux', iconName: 'activity', title: '最近回流速览', caption: '3 分钟内' },
      { key: 'tracking', iconName: 'radar', title: '更多钓区跟踪', caption: '跟踪上限更高' },
      { key: 'history', iconName: 'history', title: '历史回流复盘', caption: '更全面复盘' },
    ])
  })

  test('describes recent reflux access by membership tier', () => {
    expect(buildMembershipBenefitItems({
      membershipCode: 'V6',
      refreshLabel: '15 秒刷新',
    })[1]).toEqual({ key: 'recent-reflux', iconName: 'activity', title: '最近回流速览', caption: '30 分钟内' })

    expect(buildMembershipBenefitItems({
      membershipCode: 'V7',
      refreshLabel: '7 秒刷新',
    })[1].caption).toBe('10 分钟内')

    expect(buildMembershipBenefitItems({
      membershipCode: 'V5',
      refreshLabel: '30 秒刷新',
    })[1].caption).toBe('V6 起开放')
  })
})

describe('buildMembershipBenefitPreview', () => {
  test('uses the selected tier rule instead of another tier refresh speed', () => {
    const rules = [
      { code: 'V3', kind: 'invite', min_referrals: 0, ticket_watch_poll_interval_seconds: 120 },
      { code: 'V6', kind: 'referral', min_referrals: 20, ticket_watch_poll_interval_seconds: 15 },
      { code: 'V7', kind: 'referral', min_referrals: 30, ticket_watch_poll_interval_seconds: 7 },
      { code: 'V8', kind: 'referral', min_referrals: 50, ticket_watch_poll_interval_seconds: 3 },
      { code: 'V9', kind: 'referral', min_referrals: 80, ticket_watch_poll_interval_seconds: 1 },
    ]
    for (const [tier, refresh, reflux] of [
      ['V3', '2 分钟刷新', 'V6 起开放'],
      ['V6', '15 秒刷新', '30 分钟内'],
      ['V7', '7 秒刷新', '10 分钟内'],
      ['V8', '3 秒刷新', '3 分钟内'],
      ['V9', '1 秒刷新', '3 分钟内'],
    ]) {
      const preview = buildMembershipBenefitPreview(tier, rules)
      expect(preview?.code).toBe(tier)
      expect(preview?.items[0].caption).toBe(refresh)
      expect(preview?.items[1].caption).toBe(reflux)
      expect(preview?.items.length).toBe(4)
    }
  })

  test('does not show a default tier without a selected product', () => {
    expect(buildMembershipBenefitPreview(undefined)).toBeNull()
    expect(buildMembershipBenefitPreview('')).toBeNull()
  })

  test('does not invent a refresh interval when the selected rule is missing', () => {
    const preview = buildMembershipBenefitPreview('v9')
    expect(preview?.code).toBe('V9')
    expect(preview?.items[0].caption).toBe('以后端配置为准')
  })
})
