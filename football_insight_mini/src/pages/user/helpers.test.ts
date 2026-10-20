import { describe, expect, test } from 'bun:test'

import {
  buildUserAccountInfoItems,
  formatMembershipCardExpiry,
  buildUserUpgradeSteps,
  canShowMembershipPurchaseEntry,
  canShowOfficialWechatBindEntry,
  formatOfficialWechatBindCode,
  getOfficialWechatBindRemainingSeconds,
  formatOfficialWechatBindCountdown,
  resolveOfficialWechatBindError,
  formatMembershipExpiryLabel,
  formatNotificationEmailLabel,
  isValidNotificationEmail,
  resolveCurrentUserInviteCode,
  resolveUserMembershipMeta,
} from './helpers'
import type { MembershipTierGuide } from '../../utils/membershipRules'
import { ApiRequestError } from '../../utils/apiError'

describe('official account binding helpers', () => {
  test('shows entry only for logged in, non-review users without a confirmed official binding', () => {
    expect(canShowOfficialWechatBindEntry(true, false, undefined)).toBe(true)
    expect(canShowOfficialWechatBindEntry(true, false, false)).toBe(true)
    expect(canShowOfficialWechatBindEntry(true, false, true)).toBe(false)
    expect(canShowOfficialWechatBindEntry(false, false, false)).toBe(false)
    expect(canShowOfficialWechatBindEntry(true, true, false)).toBe(false)
  })

  test('normalizes six safe code characters and rejects malformed codes', () => {
    expect(formatOfficialWechatBindCode(' abc234 ')).toBe('ABC234')
    for (const code of ['', 'ABC23', 'ABC2345', 'AB I23', 'ABO234', 'AB1234', 'AB0234']) {
      expect(formatOfficialWechatBindCode(code)).toBe('')
    }
  })

  test('derives countdown from expiry so time spent in the background is included', () => {
    expect(getOfficialWechatBindRemainingSeconds(601_000, 1_000)).toBe(600)
    expect(getOfficialWechatBindRemainingSeconds(601_000, 599_001)).toBe(2)
    expect(getOfficialWechatBindRemainingSeconds(601_000, 601_000)).toBe(0)
    expect(getOfficialWechatBindRemainingSeconds(601_000, 900_000)).toBe(0)
    expect(getOfficialWechatBindRemainingSeconds(null, 1_000)).toBe(0)
    expect(formatOfficialWechatBindCountdown(600)).toBe('10:00')
    expect(formatOfficialWechatBindCountdown(9)).toBe('00:09')
    expect(formatOfficialWechatBindCountdown(0)).toBe('00:00')
  })

  test('maps real 429 status to rate limit and never guesses from error text', () => {
    expect(resolveOfficialWechatBindError(new ApiRequestError('server', 429))).toBe('获取过于频繁，请稍后再试')
    expect(resolveOfficialWechatBindError(new ApiRequestError('429', 500))).toBe('获取绑定码失败，请稍后重试')
    expect(resolveOfficialWechatBindError(new Error('获取过于频繁'))).toBe('获取绑定码失败，请稍后重试')
  })
})

describe('resolveCurrentUserInviteCode', () => {
  test('returns trimmed invite code when present', () => {
    expect(resolveCurrentUserInviteCode('  FI-ABC123-XYZ789  ')).toBe('FI-ABC123-XYZ789')
  })

  test('returns empty string when invite code is missing', () => {
    expect(resolveCurrentUserInviteCode('   ')).toBe('')
    expect(resolveCurrentUserInviteCode(undefined)).toBe('')
    expect(resolveCurrentUserInviteCode(null)).toBe('')
  })
})

describe('buildUserAccountInfoItems', () => {
  test('builds the account summary items for a bound wechat member', () => {
    expect(buildUserAccountInfoItems({
      hasWechatBinding: true,
      officialWechatBound: true,
      officialWechatSubscribed: true,
      joinedAtLabel: '2026-4-8',
    })).toEqual([
      { key: 'identity', iconName: 'badge-check', label: '身份状态', value: '微信会员' },
      { key: 'joined-at', iconName: 'calendar-days', label: '加入时间', value: '2026-4-8' },
      { key: 'official-follow', iconName: 'badge-check', label: '公众号关注状态', value: '已关注' },
      { key: 'official-bind', iconName: 'log-in', label: '公众号绑定状态', value: '已绑定公众号' },
    ])
  })

  test('uses official binding independently of mini-program wechat login binding', () => {
    for (const [hasWechatBinding, officialWechatBound, value] of [
      [true, false, '未绑定公众号'],
      [true, undefined, '未绑定公众号'],
      [false, true, '已绑定公众号'],
    ] as const) {
      const items = buildUserAccountInfoItems({ hasWechatBinding, officialWechatBound, joinedAtLabel: '' })
      expect(items.find(item => item.key === 'official-bind')?.value).toBe(value)
    }
  })

  test('shows follow and binding states independently, including unknown follow status', () => {
    for (const [officialWechatBound, officialWechatSubscribed, expectedFollow, expectedBinding] of [
      [true, true, '已关注', '已绑定公众号'],
      [true, false, '已取关', '已绑定公众号'],
      [false, true, '已关注', '未绑定公众号'],
      [false, false, '已取关', '未绑定公众号'],
      [true, undefined, '关注状态待确认', '已绑定公众号'],
      [false, undefined, '关注状态待确认', '未绑定公众号'],
    ] as const) {
      const items = buildUserAccountInfoItems({ hasWechatBinding: true, officialWechatBound, officialWechatSubscribed, joinedAtLabel: '' })
      expect(items.find(item => item.key === 'official-follow')?.value).toBe(expectedFollow)
      expect(items.find(item => item.key === 'official-bind')?.value).toBe(expectedBinding)
    }
  })

  test('does not repeat the membership expiry, which the membership card already shows', () => {
    const keys = buildUserAccountInfoItems({ hasWechatBinding: false, joinedAtLabel: '' }).map(item => item.key)
    expect(keys.includes('membership-expiry')).toBe(false)
  })
})

describe('formatMembershipCardExpiry', () => {
  test('perpetual membership reads as long-term', () => {
    expect(formatMembershipCardExpiry(null)).toBe('长期有效')
    expect(formatMembershipCardExpiry(undefined)).toBe('长期有效')
  })

  test('a future expiration shows the date followed by 到期', () => {
    expect(formatMembershipCardExpiry('2099-04-24T12:00:00Z')).toBe('2099-4-24 到期')
  })

  test('a past expiration reads as expired', () => {
    expect(formatMembershipCardExpiry('2020-01-01T00:00:00Z')).toBe('已过期')
  })
})

describe('formatMembershipExpiryLabel', () => {
  test('formats a future paid membership expiration date', () => {
    expect(formatMembershipExpiryLabel(
      '2027-04-24T12:00:00Z',
      new Date('2026-04-24T00:00:00Z').getTime(),
    )).toBe('有效至 2027-4-24')
  })

  test('uses stable labels for natural or expired memberships', () => {
    expect(formatMembershipExpiryLabel(null)).toBe('长期有效')
    expect(formatMembershipExpiryLabel(
      '2026-04-23T23:59:59Z',
      new Date('2026-04-24T00:00:00Z').getTime(),
    )).toBe('已过期')
  })
})

describe('notification email helpers', () => {
  test('validates email syntax', () => {
    expect(isValidNotificationEmail('user@example.com')).toBe(true)
    expect(isValidNotificationEmail(' user.name+tag@example.cn ')).toBe(true)
    expect(isValidNotificationEmail('invalid')).toBe(false)
    expect(isValidNotificationEmail('user example@example.com')).toBe(false)
  })

  test('formats notification email fallback label', () => {
    expect(formatNotificationEmailLabel(' user@example.com ')).toBe('user@example.com')
    expect(formatNotificationEmailLabel(null)).toBe('未填写')
  })
})

describe('canShowMembershipPurchaseEntry', () => {
  test('shows purchase entry for expiring V9 memberships so users can renew', () => {
    expect(canShowMembershipPurchaseEntry(
      true,
      true,
      'V9',
      '2027-05-08T04:00:00Z',
    )).toBe(true)
  })

  test('hides purchase entry for lifetime V9 memberships', () => {
    expect(canShowMembershipPurchaseEntry(true, true, 'V9', null)).toBe(false)
  })

  test('keeps purchase entry for lower tiers with wechat binding', () => {
    expect(canShowMembershipPurchaseEntry(true, true, 'V8', null)).toBe(true)
    expect(canShowMembershipPurchaseEntry(true, false, 'V8', null)).toBe(false)
  })
})

describe('buildUserUpgradeSteps', () => {
  test('maps guides into ordered upgrade steps and marks the current tier', () => {
    const guides: MembershipTierGuide[] = [
      {
        code: 'V1',
        name: '标准会员',
        condition: '默认等级：未走邀请码链路时进入 V1。',
        refreshLabel: '10 分钟刷新',
        description: '当前基础档位，余票监控按 10 分钟刷新。',
        toneClass: 'tier-tone--v1',
        pipCount: 1,
      },
      {
        code: 'V3',
        name: '邀请会员',
        condition: '升级条件：使用邀请码完成注册或首次微信绑定。',
        refreshLabel: '3 分钟刷新',
        description: '邀请码会员默认档位，余票监控按 3 分钟刷新。',
        toneClass: 'tier-tone--v3',
        pipCount: 2,
      },
    ]

    expect(buildUserUpgradeSteps(guides, 'V3')).toEqual([
      {
        key: 'V1',
        code: 'V1',
        name: '标准会员',
        condition: '默认等级：未走邀请码链路时进入 V1。',
        refreshLabel: '10 分钟刷新',
        toneClass: 'tier-tone--v1',
        isCurrent: false,
      },
      {
        key: 'V3',
        code: 'V3',
        name: '邀请会员',
        condition: '升级条件：使用邀请码完成注册或首次微信绑定。',
        refreshLabel: '3 分钟刷新',
        toneClass: 'tier-tone--v3',
        isCurrent: true,
      },
    ])
  })
})

describe('resolveUserMembershipMeta', () => {
  test('maps V3 to invite tier', () => {
    const meta = resolveUserMembershipMeta('V3')
    expect(meta.code).toBe('V3')
    expect(meta.heroClass).toBe('account-hero--v3')
    expect(meta.badgeLabel).toBe('V3 邀请会员')
  })

  test('maps V2 to advanced tier', () => {
    const meta = resolveUserMembershipMeta('V2')
    expect(meta.heroClass).toBe('account-hero--v2')
  })

  test('maps V4+ to referral ladder reusing v3 hero', () => {
    const meta = resolveUserMembershipMeta('V7')
    expect(meta.badgeLabel).toBe('V7 推荐会员')
    expect(meta.heroClass).toBe('account-hero--v3')
  })

  test('defaults unknown or empty to standard member', () => {
    expect(resolveUserMembershipMeta(undefined).code).toBe('V1')
    expect(resolveUserMembershipMeta('  ').heroClass).toBe('account-hero--v1')
    expect(resolveUserMembershipMeta('V1').badgeLabel).toBe('V1 标准会员')
  })
})
