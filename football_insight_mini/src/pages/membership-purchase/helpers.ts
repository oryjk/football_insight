import type { MembershipProduct, MembershipProductOption } from '../../api/payment'

export const MEMBERSHIP_PURCHASE_AGREEMENT_POINTS = [
  '会员服务自支付成功并完成系统确认之时起生效；在服务器正常运营、相关服务未因不可抗力或监管要求中止的情况下，有效期为一年。',
  '会员权益仅限当前登录账号使用，不支持转让、拆分、出租、出借或用于任何违反平台规则的用途。',
  '余票监控、最近回流速览等能力依赖公开数据、第三方接口、网络环境及系统调度状态，平台将尽合理努力保持服务稳定，但不承诺任何票务结果。',
  '如遇服务器维护、升级、故障排查或第三方服务异常，部分功能可能短时不可用；平台会在合理范围内尽快恢复服务。',
  '本会员属于虚拟服务，支付成功后将即时开通对应会员权益。请在支付前确认所选档位、价格及账号信息；除因平台原因导致服务无法开通或法律法规另有规定外，支付后不支持退款。',
]

export function normalizeMembershipProductOptions(
  product: MembershipProduct | null | undefined,
): MembershipProductOption[] {
  const source = product?.products?.length ? product.products : []

  return [...source].sort((left, right) =>
    parseTierRank(left.target_tier) - parseTierRank(right.target_tier),
  )
}

export function filterMembershipProductOptionsForUpgrade(
  options: MembershipProductOption[],
  currentTier: string | null | undefined,
): MembershipProductOption[] {
  const currentRank = parseTierRank(currentTier || 'V1')

  return options.filter((option) => parseTierRank(option.target_tier) > currentRank)
}

export function formatMembershipPrice(priceCents: number | undefined): string {
  if (typeof priceCents !== 'number' || !Number.isFinite(priceCents)) {
    return '--'
  }

  return (priceCents / 100).toFixed(0)
}

export function getMembershipPayPriceCents(option: MembershipProductOption | null | undefined): number | undefined {
  if (!option) {
    return undefined
  }

  return getFiniteCents(option.pay_price_cents) ?? getFiniteCents(option.price_cents)
}

export function getMembershipOriginalPriceCents(option: MembershipProductOption | null | undefined): number | undefined {
  if (!option) {
    return undefined
  }

  return getFiniteCents(option.original_price_cents) ?? getFiniteCents(option.price_cents)
}

export function isMembershipUpgradePrice(option: MembershipProductOption | null | undefined): boolean {
  const payPrice = getMembershipPayPriceCents(option)
  const originalPrice = getMembershipOriginalPriceCents(option)
  const upgradeFee = getFiniteCents(option?.upgrade_fee_cents) ?? 0

  return upgradeFee > 0
    && typeof payPrice === 'number'
    && typeof originalPrice === 'number'
    && payPrice < originalPrice
}

export function formatMembershipUpgradeNote(option: MembershipProductOption | null | undefined): string {
  if (!isMembershipUpgradePrice(option)) {
    return ''
  }

  return `补差价 + ¥${formatMembershipPrice(option?.upgrade_fee_cents)} 服务费`
}

function getFiniteCents(value: number | undefined): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}

function parseTierRank(tier: string): number {
  const parsed = Number.parseInt(tier.trim().replace(/^V/i, ''), 10)
  return Number.isFinite(parsed) ? parsed : 0
}
