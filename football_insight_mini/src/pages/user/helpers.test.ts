import { describe, expect, test } from 'bun:test'

import {
  buildUserAccountInfoItems,
  buildUserBenefitItems,
  buildUserUpgradeSteps,
  formatMembershipExpiryLabel,
  resolveCurrentUserInviteCode,
} from './helpers'
import type { MembershipTierGuide } from '../../utils/membershipRules'

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
      membershipCode: 'V9',
      hasWechatBinding: true,
      joinedAtLabel: '2026-4-8',
      membershipExpiresAtLabel: '有效至 2027-4-24',
    })).toEqual([
      { key: 'membership', iconLabel: '级', label: '当前等级', value: 'V9' },
      { key: 'identity', iconLabel: '身', label: '身份状态', value: '微信会员' },
      { key: 'joined-at', iconLabel: '时', label: '加入时间', value: '2026-4-8' },
      { key: 'membership-expiry', iconLabel: '期', label: '会员有效期', value: '有效至 2027-4-24' },
      { key: 'login', iconLabel: '微', label: '登录状态', value: '已绑定微信' },
    ])
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

describe('buildUserBenefitItems', () => {
  test('uses richer captions for higher membership tiers', () => {
    expect(buildUserBenefitItems({
      membershipCode: 'V9',
      refreshLabel: '1 秒刷新',
    })).toEqual([
      { key: 'watch', iconLabel: '票', title: '余票监控', caption: '1 秒刷新' },
      { key: 'recent-reflux', iconLabel: '回', title: '最近回流速览', caption: '3 分钟内' },
      { key: 'tracking', iconLabel: '盯', title: '更多钓区跟踪', caption: '跟踪上限更高' },
      { key: 'history', iconLabel: '盘', title: '历史回流复盘', caption: '更全面复盘' },
    ])
  })

  test('describes recent reflux access by membership tier', () => {
    expect(buildUserBenefitItems({
      membershipCode: 'V6',
      refreshLabel: '15 秒刷新',
    })[1]).toEqual({ key: 'recent-reflux', iconLabel: '回', title: '最近回流速览', caption: '30 分钟内' })

    expect(buildUserBenefitItems({
      membershipCode: 'V7',
      refreshLabel: '7 秒刷新',
    })[1].caption).toBe('10 分钟内')

    expect(buildUserBenefitItems({
      membershipCode: 'V5',
      refreshLabel: '30 秒刷新',
    })[1].caption).toBe('V6 起开放')
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
