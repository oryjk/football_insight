import type { MembershipTierRule } from '../types/system'

export interface MembershipTierGuide {
  code: string
  name: string
  condition: string
  refreshLabel: string
  description: string
  toneClass: string
  pipCount: number
}

interface MembershipTierPresentation {
  name: string
  toneClass: string
  pipCount: number
}

const DEFAULT_TICKET_WATCH_POLL_INTERVAL_SECONDS = 600

const MEMBERSHIP_TIER_PRESENTATION: Record<string, MembershipTierPresentation> = {
  V1: {
    name: '标准会员',
    toneClass: 'tier-tone--v1',
    pipCount: 1,
  },
  V2: {
    name: '进阶会员',
    toneClass: 'tier-tone--v2',
    pipCount: 1,
  },
  V3: {
    name: '邀请会员',
    toneClass: 'tier-tone--v3',
    pipCount: 2,
  },
  V4: {
    name: '推荐会员',
    toneClass: 'tier-tone--v4',
    pipCount: 2,
  },
  V5: {
    name: '推荐会员',
    toneClass: 'tier-tone--v5',
    pipCount: 3,
  },
  V6: {
    name: '推荐会员',
    toneClass: 'tier-tone--v6',
    pipCount: 3,
  },
  V7: {
    name: '推荐会员',
    toneClass: 'tier-tone--v7',
    pipCount: 4,
  },
  V8: {
    name: '推荐会员',
    toneClass: 'tier-tone--v8',
    pipCount: 4,
  },
  V9: {
    name: '推荐会员',
    toneClass: 'tier-tone--v9',
    pipCount: 5,
  },
}

function normalizeMembershipTierCode(code?: string | null): string {
  const normalizedCode = code?.trim().toUpperCase()
  return normalizedCode || 'V1'
}

function membershipTierRank(code?: string | null): number {
  const normalizedCode = normalizeMembershipTierCode(code)
  const parsedRank = Number.parseInt(normalizedCode.replace(/^V/i, ''), 10)
  return Number.isFinite(parsedRank) ? parsedRank : 1
}

export function isMembershipTierAtLeast(code: string | null | undefined, minimumCode: string): boolean {
  return membershipTierRank(code) >= membershipTierRank(minimumCode)
}

function resolveMembershipTierPresentation(code?: string | null): MembershipTierPresentation {
  const normalizedCode = normalizeMembershipTierCode(code)

  if (MEMBERSHIP_TIER_PRESENTATION[normalizedCode]) {
    return MEMBERSHIP_TIER_PRESENTATION[normalizedCode]
  }

  if (membershipTierRank(normalizedCode) >= 4) {
    return {
      name: '推荐会员',
      toneClass: 'tier-tone--v9',
      pipCount: 5,
    }
  }

  return MEMBERSHIP_TIER_PRESENTATION.V1
}

function isReferralRule(rule: MembershipTierRule): boolean {
  return rule.kind === 'referral' && typeof rule.min_referrals === 'number' && rule.min_referrals > 0
}

// 后端只在目标档位为 referral 时按邀请人数升级；standard/invite 档位上的 min_referrals 不生效，不能写成升级条件。
function buildMembershipCondition(rule: MembershipTierRule): string {
  const normalizedCode = normalizeMembershipTierCode(rule.code)

  if (normalizedCode === 'V1' || (rule.kind === 'standard' && rule.min_referrals == null)) {
    return '默认等级：注册时没有填邀请码的账号为 V1。'
  }

  if (rule.kind === 'invite') {
    return '升级条件：用公众号领取的邀请码注册。'
  }

  if (isReferralRule(rule)) {
    return `升级条件：累计邀请 ${rule.min_referrals} 位好友注册。`
  }

  return '升级条件：暂未开放。'
}

function buildMembershipDescription(rule: MembershipTierRule, refreshLabel: string): string {
  if (rule.kind === 'standard' && rule.min_referrals == null) {
    return `当前基础档位，余票监控按 ${refreshLabel}。`
  }

  if (rule.kind === 'standard' && typeof rule.min_referrals === 'number') {
    return `进阶档位，余票监控按 ${refreshLabel}。`
  }

  if (rule.kind === 'invite') {
    return `邀请码会员默认档位，余票监控按 ${refreshLabel}。`
  }

  if (rule.kind === 'referral') {
    return `推荐升级后，余票监控按 ${refreshLabel}。`
  }

  return `当前会员权益按 ${refreshLabel}。`
}

export function normalizeTicketWatchPollIntervalSeconds(value?: number | null): number {
  if (typeof value !== 'number' || !Number.isFinite(value) || value <= 0) {
    return DEFAULT_TICKET_WATCH_POLL_INTERVAL_SECONDS
  }

  return Math.floor(value)
}

export function formatTicketWatchPollIntervalLabel(pollIntervalSeconds?: number | null): string {
  const seconds = normalizeTicketWatchPollIntervalSeconds(pollIntervalSeconds)

  if (seconds >= 60 && seconds % 60 === 0) {
    return `${seconds / 60} 分钟刷新`
  }

  return `${seconds} 秒刷新`
}

export function buildMembershipTierGuides(rules?: MembershipTierRule[] | null): MembershipTierGuide[] {
  return [...(rules ?? [])]
    .sort((left, right) => membershipTierRank(left.code) - membershipTierRank(right.code))
    .map((rule) => {
      const normalizedCode = normalizeMembershipTierCode(rule.code)
      const presentation = resolveMembershipTierPresentation(normalizedCode)
      const refreshLabel = formatTicketWatchPollIntervalLabel(rule.ticket_watch_poll_interval_seconds)

      return {
        code: normalizedCode,
        name: presentation.name,
        condition: buildMembershipCondition(rule),
        refreshLabel,
        description: buildMembershipDescription(rule, refreshLabel),
        toneClass: presentation.toneClass,
        pipCount: presentation.pipCount,
      }
    })
}

export function resolveMembershipTierGuide(
  tier?: string | null,
  rules?: MembershipTierRule[] | null,
  currentPollIntervalSeconds?: number | null,
): MembershipTierGuide {
  const normalizedTier = normalizeMembershipTierCode(tier)
  const guide = buildMembershipTierGuides(rules).find((item) => item.code === normalizedTier)

  if (guide) {
    return guide
  }

  const presentation = resolveMembershipTierPresentation(normalizedTier)
  const refreshLabel = formatTicketWatchPollIntervalLabel(currentPollIntervalSeconds)

  return {
    code: normalizedTier,
    name: presentation.name,
    condition: '升级条件：以后端当前配置为准。',
    refreshLabel,
    description: `当前会员权益按 ${refreshLabel}。`,
    toneClass: presentation.toneClass,
    pipCount: presentation.pipCount,
  }
}

/** 锁定卡片上的「邀请升级」说明；人数取自后端当前配置，缺失时不写具体数字。 */
export function formatInviteUpgradeHint(rules: MembershipTierRule[] | null | undefined, tier: string): string {
  const code = normalizeMembershipTierCode(tier)
  const referrals = rules?.find(rule => normalizeMembershipTierCode(rule.code) === code)?.min_referrals
  const target = typeof referrals === 'number' && referrals > 0
    ? `累计邀请 ${referrals} 位好友注册，就能免费升到 ${code}`
    : `邀请好友注册，累计达标就能免费升到 ${code}`
  return `${target}。好友第一次登录时在「推荐码」一栏填你的邀请码即可。`
}

export interface MembershipLevelRow {
  code: string
  name: string
  /** 获得方式，如「邀请 20 人 / 开通」。 */
  ways: string
  refreshLabel: string
}

/**
 * 会员等级说明表：每个等级怎么拿到。没有任何获得途径的档位（如配置里的 V2）不展示。
 * purchasableTiers 为当前可购买的年度档位。
 */
export function buildMembershipLevelRows(
  rules: MembershipTierRule[] | null | undefined,
  purchasableTiers: string[],
): MembershipLevelRow[] {
  const purchasable = new Set(purchasableTiers.map(normalizeMembershipTierCode))
  return [...(rules ?? [])]
    .sort((left, right) => membershipTierRank(left.code) - membershipTierRank(right.code))
    .flatMap((rule) => {
      const code = normalizeMembershipTierCode(rule.code)
      const ways: string[] = []
      if (code === 'V1') ways.push('注册即有')
      if (rule.kind === 'invite') ways.push('用邀请码注册')
      if (isReferralRule(rule)) ways.push(`邀请 ${rule.min_referrals} 人`)
      if (purchasable.has(code)) ways.push('开通')
      if (!ways.length) return []
      return [{
        code,
        name: resolveMembershipTierPresentation(code).name,
        ways: ways.join(' / '),
        refreshLabel: formatTicketWatchPollIntervalLabel(rule.ticket_watch_poll_interval_seconds),
      }]
    })
}

export interface InviteMilestone {
  code: string
  referrals: number
  refreshLabel: string
}

/** 邀请人数对应的免费升级档位（只含后端会按人数自动升级的 referral 档位）。 */
export function buildInviteMilestones(rules: MembershipTierRule[] | null | undefined): InviteMilestone[] {
  return (rules ?? [])
    .filter(isReferralRule)
    .map(rule => ({
      code: normalizeMembershipTierCode(rule.code),
      referrals: rule.min_referrals as number,
      refreshLabel: formatTicketWatchPollIntervalLabel(rule.ticket_watch_poll_interval_seconds),
    }))
    .sort((left, right) => left.referrals - right.referrals)
}
