export function resolveCurrentUserInviteCode(inviteCode: string | null | undefined): string {
  return typeof inviteCode === 'string' ? inviteCode.trim() : ''
}

export interface UserAccountInfoItem {
  key: string
  iconName: 'badge-check' | 'calendar-days' | 'calendar-check' | 'log-in'
  label: string
  value: string
}

export interface UserBenefitItem {
  key: string
  iconName: 'ticket' | 'activity' | 'radar' | 'history'
  title: string
  caption: string
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
  membershipExpiresAtLabel: string
}

interface BuildUserBenefitOptions {
  membershipCode: string
  refreshLabel: string
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
      key: 'membership-expiry',
      iconName: 'calendar-check',
      label: '会员有效期',
      value: options.membershipExpiresAtLabel,
    },
    {
      key: 'login',
      iconName: 'log-in',
      label: '登录状态',
      value: options.hasWechatBinding ? '已绑定微信' : '未绑定微信',
    },
  ]
}

export function buildUserBenefitItems(
  options: BuildUserBenefitOptions,
): UserBenefitItem[] {
  const tierNumber = parseMembershipTierNumber(options.membershipCode)

  return [
    {
      key: 'watch',
      iconName: 'ticket',
      title: '余票监控',
      caption: options.refreshLabel,
    },
    {
      key: 'recent-reflux',
      iconName: 'activity',
      title: '最近回流速览',
      caption: resolveRecentRefluxBenefitCaption(tierNumber),
    },
    {
      key: 'tracking',
      iconName: 'radar',
      title: '更多钓区跟踪',
      caption: tierNumber >= 4 ? '跟踪上限更高' : '常用钓区可跟踪',
    },
    {
      key: 'history',
      iconName: 'history',
      title: '历史回流复盘',
      caption: tierNumber >= 5 ? '更全面复盘' : '回看近期变化',
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

export function isValidNotificationEmail(email: string): boolean {
  const normalized = email.trim()
  return normalized.length > 0 && normalized.length <= 254 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)
}

export function formatNotificationEmailLabel(email: string | null | undefined): string {
  const normalized = typeof email === 'string' ? email.trim() : ''
  return normalized || '未填写'
}

function parseMembershipTierNumber(code: string): number {
  const parsed = Number.parseInt(code.trim().replace(/^V/i, ''), 10)
  return Number.isFinite(parsed) ? parsed : 1
}

function resolveRecentRefluxBenefitCaption(tierNumber: number): string {
  if (tierNumber >= 8) {
    return '3 分钟内'
  }

  if (tierNumber >= 7) {
    return '10 分钟内'
  }

  if (tierNumber >= 6) {
    return '30 分钟内'
  }

  return 'V6 起开放'
}
