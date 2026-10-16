import { ApiRequestError } from '../../utils/apiError'

export function canShowOfficialWechatBindEntry(hasUser: boolean, underReview: boolean, officialBound?: boolean): boolean {
  return hasUser && !underReview && officialBound !== true
}

export function formatOfficialWechatBindCode(code: string): string {
  const normalized = code.trim().toUpperCase()
  return /^[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{6}$/.test(normalized) ? normalized : ''
}

export function getOfficialWechatBindRemainingSeconds(expiresAt: number | null, nowMs = Date.now()): number {
  return expiresAt === null ? 0 : Math.max(0, Math.ceil((expiresAt - nowMs) / 1000))
}

export function formatOfficialWechatBindCountdown(seconds: number): string {
  const remaining = Math.max(0, Math.floor(seconds))
  return `${String(Math.floor(remaining / 60)).padStart(2, '0')}:${String(remaining % 60).padStart(2, '0')}`
}

export function resolveOfficialWechatBindError(error: unknown): string {
  return error instanceof ApiRequestError && error.statusCode === 429
    ? '获取过于频繁，请稍后再试'
    : '获取绑定码失败，请稍后重试'
}

export function resolveCurrentUserInviteCode(inviteCode: string | null | undefined): string {
  return typeof inviteCode === 'string' ? inviteCode.trim() : ''
}

export interface UserAccountInfoItem {
  key: string
  iconName: 'badge-check' | 'calendar-days' | 'log-in'
  label: string
  value: string
}

export interface UserUpgradeStep {
  key: string
  code: string
  name: string
  condition: string
  refreshLabel: string
  toneClass: string
  isCurrent: boolean
}

interface BuildUserAccountInfoOptions {
  hasWechatBinding: boolean
  joinedAtLabel: string
}

interface MembershipTierGuideLike {
  code: string
  name: string
  condition: string
  refreshLabel: string
  toneClass: string
}

export function buildUserAccountInfoItems(
  options: BuildUserAccountInfoOptions,
): UserAccountInfoItem[] {
  return [
    {
      key: 'identity',
      iconName: 'badge-check',
      label: '身份状态',
      value: options.hasWechatBinding ? '微信会员' : '标准会员',
    },
    {
      key: 'joined-at',
      iconName: 'calendar-days',
      label: '加入时间',
      value: options.joinedAtLabel,
    },
    {
      key: 'login',
      iconName: 'log-in',
      label: '登录状态',
      value: options.hasWechatBinding ? '已绑定微信' : '未绑定微信',
    },
  ]
}

export function buildUserUpgradeSteps(
  guides: MembershipTierGuideLike[],
  currentCode: string,
): UserUpgradeStep[] {
  const normalizedCurrentCode = currentCode.trim().toUpperCase()

  return guides.map((guide) => ({
    key: guide.code,
    code: guide.code,
    name: guide.name,
    condition: guide.condition,
    refreshLabel: guide.refreshLabel,
    toneClass: guide.toneClass,
    isCurrent: guide.code.trim().toUpperCase() === normalizedCurrentCode,
  }))
}

export function canShowMembershipPurchaseEntry(
  hasUser: boolean,
  hasWechatBinding: boolean,
  membershipTier: string | null | undefined,
  membershipExpiresAt: string | null | undefined,
): boolean {
  if (!hasUser || !hasWechatBinding) {
    return false
  }

  const normalizedTier = typeof membershipTier === 'string'
    ? membershipTier.trim().toUpperCase()
    : ''

  if (normalizedTier !== 'V9') {
    return true
  }

  return typeof membershipExpiresAt === 'string' && membershipExpiresAt.trim().length > 0
}

export function formatMembershipExpiryLabel(
  expiresAt: string | null | undefined,
  nowMs = Date.now(),
): string {
  if (!expiresAt) {
    return '长期有效'
  }

  const date = new Date(expiresAt)

  if (Number.isNaN(date.getTime())) {
    return '长期有效'
  }

  if (date.getTime() <= nowMs) {
    return '已过期'
  }

  return `有效至 ${date.getFullYear()}-${date.getMonth() + 1}-${date.getDate()}`
}

/** 会员卡上的有效期文案：会员有效期只在会员卡里展示一次，账户信息列表不再重复。 */
export function formatMembershipCardExpiry(expiresAt: string | null | undefined): string {
  const label = formatMembershipExpiryLabel(expiresAt)
  if (label === '长期有效' || label === '已过期') return label
  return `${label.replace(/^有效至\s*/, '')} 到期`
}

export function isValidNotificationEmail(email: string): boolean {
  const normalized = email.trim()
  return normalized.length > 0 && normalized.length <= 254 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)
}

export function formatNotificationEmailLabel(email: string | null | undefined): string {
  const normalized = typeof email === 'string' ? email.trim() : ''
  return normalized || '未填写'
}

export interface UserMembershipMeta {
  code: string
  badgeLabel: string
  heroClass: string
  levelHint: string
  levelDescription: string
}

export function resolveUserMembershipMeta(tier: string | undefined): UserMembershipMeta {
  const normalizedTier = tier?.trim() || 'V1'
  const tierNumber = Number.parseInt(normalizedTier.replace(/^V/i, ''), 10)

  if (normalizedTier === 'V3') {
    return {
      code: 'V3',
      badgeLabel: 'V3 邀请会员',
      heroClass: 'account-hero--v3',
      levelHint: '邀请码会员',
      levelDescription: '通过邀请码完成首次绑定，当前按 V3 等级展示。',
    }
  }

  if (normalizedTier === 'V2') {
    return {
      code: 'V2',
      badgeLabel: 'V2 进阶会员',
      heroClass: 'account-hero--v2',
      levelHint: '进阶会员',
      levelDescription: '通过推荐好友注册已升级到 V2，回流频率按进阶档位开放。',
    }
  }

  if (Number.isFinite(tierNumber) && tierNumber >= 4) {
    return {
      code: normalizedTier,
      badgeLabel: `${normalizedTier} 推荐会员`,
      heroClass: 'account-hero--v3',
      levelHint: '推荐升级会员',
      levelDescription: `当前已经通过推荐升级到 ${normalizedTier}，回流频率和后续会员权益会按更高档位开放。`,
    }
  }

  return {
    code: normalizedTier,
    badgeLabel: `${normalizedTier} 标准会员`,
    heroClass: 'account-hero--v1',
    levelHint: '标准会员',
    levelDescription: '当前是 V1 基础会员身份。',
  }
}
