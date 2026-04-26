export function resolveCurrentUserInviteCode(inviteCode: string | null | undefined): string {
  return typeof inviteCode === 'string' ? inviteCode.trim() : ''
}

export interface UserAccountInfoItem {
  key: string
  iconLabel: string
  label: string
  value: string
}

export interface UserBenefitItem {
  key: string
  iconLabel: string
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
  membershipCode: string
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
      key: 'membership',
      iconLabel: '级',
      label: '当前等级',
      value: options.membershipCode,
    },
    {
      key: 'identity',
      iconLabel: '身',
      label: '身份状态',
      value: options.hasWechatBinding ? '微信会员' : '标准会员',
    },
    {
      key: 'joined-at',
      iconLabel: '时',
      label: '加入时间',
      value: options.joinedAtLabel,
    },
    {
      key: 'membership-expiry',
      iconLabel: '期',
      label: '会员有效期',
      value: options.membershipExpiresAtLabel,
    },
    {
      key: 'login',
      iconLabel: '微',
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
      iconLabel: '票',
      title: '余票监控',
      caption: options.refreshLabel,
    },
    {
      key: 'recent-reflux',
      iconLabel: '回',
      title: '最近回流速览',
      caption: resolveRecentRefluxBenefitCaption(tierNumber),
    },
    {
      key: 'tracking',
      iconLabel: '盯',
      title: '更多钓区跟踪',
      caption: tierNumber >= 4 ? '跟踪上限更高' : '常用钓区可跟踪',
    },
    {
      key: 'history',
      iconLabel: '盘',
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
