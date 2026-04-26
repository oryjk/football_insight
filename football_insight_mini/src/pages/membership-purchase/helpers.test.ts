import { describe, expect, test } from 'bun:test'

import {
  MEMBERSHIP_PURCHASE_AGREEMENT_POINTS,
  formatMembershipUpgradeNote,
  formatMembershipPrice,
  filterMembershipProductOptionsForUpgrade,
  getMembershipOriginalPriceCents,
  getMembershipPayPriceCents,
  isMembershipUpgradePrice,
  normalizeMembershipProductOptions,
} from './helpers'

const backendProductOptions = [
  {
    target_tier: 'V6',
    price_cents: 3900,
    title: 'V6 回流速览会员',
    subtitle: '查看 30 分钟内',
    description: '购买后立即升级为 V6 会员，可查看最近 30 分钟内的回流区域。',
  },
  {
    target_tier: 'V7',
    price_cents: 5900,
    title: 'V7 进阶回流会员',
    subtitle: '查看 10 分钟内',
    description: '购买后立即升级为 V7 会员，可查看最近 10 分钟内的回流区域。',
  },
  {
    target_tier: 'V8',
    price_cents: 7900,
    title: 'V8 高阶回流会员',
    subtitle: '查看 3 分钟内',
    description: '购买后立即升级为 V8 会员，可查看最近 3 分钟内的回流区域。',
  },
  {
    target_tier: 'V9',
    price_cents: 9900,
    title: 'V9 旗舰回流会员',
    subtitle: '同 V8 速览权限',
    description: '购买后立即升级为 V9 会员，最近回流速览权限与 V8 一致。',
  },
]

describe('normalizeMembershipProductOptions', () => {
  test('does not provide frontend fallback products when backend catalog is missing', () => {
    expect(normalizeMembershipProductOptions(null)).toEqual([])
    expect(normalizeMembershipProductOptions({ price_cents: 0, title: '', subtitle: '', description: '', products: [] })).toEqual([])
  })

  test('sorts backend product options by tier', () => {
    expect(normalizeMembershipProductOptions({
      price_cents: 9900,
      title: '会员',
      subtitle: '会员',
      description: '会员',
      products: [
        backendProductOptions[3],
        backendProductOptions[0],
      ],
    }).map((item) => item.target_tier)).toEqual(['V6', 'V9'])
  })
})

describe('filterMembershipProductOptionsForUpgrade', () => {
  test('keeps only products above the current membership tier', () => {
    expect(filterMembershipProductOptionsForUpgrade(backendProductOptions, 'V7').map((item) => item.target_tier)).toEqual(['V8', 'V9'])
    expect(filterMembershipProductOptionsForUpgrade(backendProductOptions, 'V9')).toEqual([])
    expect(filterMembershipProductOptionsForUpgrade(backendProductOptions, 'V3').map((item) => item.target_tier)).toEqual(['V6', 'V7', 'V8', 'V9'])
  })
})

describe('formatMembershipPrice', () => {
  test('formats cents as yuan without redundant decimals', () => {
    expect(formatMembershipPrice(3900)).toBe('39')
    expect(formatMembershipPrice(undefined)).toBe('--')
  })
})

describe('membership checkout price helpers', () => {
  test('prefers backend computed pay price for paid upgrades', () => {
    const option = {
      ...backendProductOptions[1],
      original_price_cents: 5900,
      pay_price_cents: 3500,
      upgrade_fee_cents: 1500,
    }

    expect(getMembershipPayPriceCents(option)).toBe(3500)
    expect(getMembershipOriginalPriceCents(option)).toBe(5900)
    expect(isMembershipUpgradePrice(option)).toBe(true)
    expect(formatMembershipUpgradeNote(option)).toBe('补差价 + ¥15 服务费')
  })

  test('falls back to product price when backend computed fields are absent', () => {
    const option = backendProductOptions[0]

    expect(getMembershipPayPriceCents(option)).toBe(3900)
    expect(getMembershipOriginalPriceCents(option)).toBe(3900)
    expect(isMembershipUpgradePrice(option)).toBe(false)
    expect(formatMembershipUpgradeNote(option)).toBe('')
  })
})

describe('MEMBERSHIP_PURCHASE_AGREEMENT_POINTS', () => {
  test('states the one year validity condition formally', () => {
    const content = MEMBERSHIP_PURCHASE_AGREEMENT_POINTS.join('')

    expect(content.includes('有效期为一年')).toBe(true)
    expect(content.includes('服务器正常运营')).toBe(true)
  })

  test('states that membership is a virtual service and paid orders are non-refundable', () => {
    const content = MEMBERSHIP_PURCHASE_AGREEMENT_POINTS.join('')

    expect(content.includes('虚拟服务')).toBe(true)
    expect(content.includes('支付后不支持退款')).toBe(true)
  })
})
