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
  formatMembershipDuration,
  buildMembershipPurchaseAgreement,
  formatMembershipTierLabel,
  resolveSelectedTier,
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

describe('configured membership duration', () => {
  test('shows one calendar year for twelve months and shorter product durations accurately', () => {
    expect(formatMembershipDuration(12)).toBe('一年')
    expect(formatMembershipDuration(3)).toBe('3 个月')
    expect(formatMembershipDuration(24)).toBe('2 年')
    expect(formatMembershipDuration(undefined)).toBe('一年')
  })
  test('uses the selected product duration in the purchase agreement', () => {
    expect(buildMembershipPurchaseAgreement(3)[0].includes('有效期为3 个月')).toBe(true)
    expect(buildMembershipPurchaseAgreement(12)[0].includes('有效期为一年')).toBe(true)
    expect(buildMembershipPurchaseAgreement(3)[1]).toBe(MEMBERSHIP_PURCHASE_AGREEMENT_POINTS[1])
  })
})

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

  test('keeps current V9 product when V9 membership has an expiration date for renewal', () => {
    expect(filterMembershipProductOptionsForUpgrade(
      backendProductOptions,
      'V9',
      '2027-05-08T04:00:00Z',
    ).map((item) => item.target_tier)).toEqual(['V9'])
  })
})

describe('formatMembershipPrice', () => {
  test('formats cents as yuan without redundant decimals', () => {
    expect(formatMembershipPrice(3900)).toBe('39')
    expect(formatMembershipPrice(1)).toBe('0.01')
    expect(formatMembershipPrice(10)).toBe('0.10')
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

describe('formatMembershipTierLabel', () => {
  test('V3 对外展示为 VIP3，其余档位原样展示', () => {
    expect(formatMembershipTierLabel('V3')).toBe('VIP3')
    expect(formatMembershipTierLabel('V6')).toBe('V6')
  })

  test('缺失档位展示占位符', () => {
    expect(formatMembershipTierLabel(undefined)).toBe('--')
    expect(formatMembershipTierLabel('')).toBe('--')
  })
})

describe('resolveSelectedTier', () => {
  const options = [{ target_tier: 'V6' }, { target_tier: 'V7' }, { target_tier: 'V9' }] as never[]

  test('刷新后保留用户已选档位', () => {
    expect(resolveSelectedTier('V9', options)).toBe('V9')
  })

  test('没有选择过时默认第一个档位', () => {
    expect(resolveSelectedTier('', options)).toBe('V6')
  })

  test('已选档位不再可购买（如已升级）时回到第一个档位', () => {
    expect(resolveSelectedTier('V8', options)).toBe('V6')
  })

  test('没有可购买档位时为空', () => {
    expect(resolveSelectedTier('V6', [])).toBe('')
  })
})
