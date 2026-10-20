import { describe, expect, test } from 'bun:test'

import { resolveMembershipBenefitsLocked, buildMembershipBenefitItems, buildMembershipBenefitPreview, buildMembershipTierSummary, buildMembershipPassSummary } from './membershipBenefits'

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

  test('requires binding for a non-paying unbound account even when the old benefits flag allows access', () => {
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: true, official_wechat_binding_required: true })).toBe(true)
    expect(resolveMembershipBenefitsLocked({ membership_benefits_enabled: true, official_wechat_binding_required: false })).toBe(false)
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

describe('buildMembershipTierSummary', () => {
  const rules = [
    { code: 'V6', kind: 'referral', min_referrals: 20, ticket_watch_poll_interval_seconds: 15 },
    { code: 'V8', kind: 'referral', min_referrals: 50, ticket_watch_poll_interval_seconds: 3 },
    { code: 'V9', kind: 'referral', min_referrals: 80, ticket_watch_poll_interval_seconds: 1 },
  ]

  test('各档位按同一格式展示回流时间窗与刷新速度', () => {
    expect(buildMembershipTierSummary('V6', rules, '查看 30 分钟内')).toBe('最近回流 30 分钟内 · 15 秒刷新')
    expect(buildMembershipTierSummary('V8', rules, '查看 3 分钟内')).toBe('最近回流 3 分钟内 · 3 秒刷新')
  })

  test('V9 与 V8 时间窗相同时，用刷新速度体现差异', () => {
    expect(buildMembershipTierSummary('V9', rules, '同 V8 速览权限')).toBe('最近回流 3 分钟内 · 1 秒刷新')
  })

  test('缺少该档位规则时退回后端副标题，不臆造刷新速度', () => {
    expect(buildMembershipTierSummary('V7', rules, '查看 10 分钟内')).toBe('查看 10 分钟内')
    expect(buildMembershipTierSummary('V7', null, '查看 10 分钟内')).toBe('查看 10 分钟内')
  })
})

describe('buildMembershipPassSummary', () => {
  const rules = [
    { code: 'V4', kind: 'referral', min_referrals: 10, ticket_watch_poll_interval_seconds: 60 },
    { code: 'V5', kind: 'referral', min_referrals: 15, ticket_watch_poll_interval_seconds: 30 },
  ]

  test('体验卡只说明刷新速度，不把回流时间窗当成卖点', () => {
    expect(buildMembershipPassSummary('V4', rules, '次日 23:59 前有效')).toBe('余票监控 1 分钟刷新')
    expect(buildMembershipPassSummary('V5', rules, '次日 23:59 前有效')).toBe('余票监控 30 秒刷新')
  })

  test('缺少该档位规则时退回后端副标题，不臆造刷新速度', () => {
    expect(buildMembershipPassSummary('V4', null, '次日 23:59 前有效')).toBe('次日 23:59 前有效')
    expect(buildMembershipPassSummary('V7', rules, '次日 23:59 前有效')).toBe('次日 23:59 前有效')
  })
})
