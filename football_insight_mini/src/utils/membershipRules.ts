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

function buildMembershipCondition(rule: MembershipTierRule): string {
  const normalizedCode = normalizeMembershipTierCode(rule.code)

  if (normalizedCode === 'V1' || (rule.kind === 'standard' && rule.min_referrals == null)) {
    return '默认等级：未走邀请码链路时进入 V1。'
  }

  if (typeof rule.min_referrals === 'number') {
    return `升级条件：累计推荐 ${rule.min_referrals} 人。`
  }

  if (rule.kind === 'invite') {
    return '升级条件：使用邀请码完成注册或首次微信绑定。'
  }

  return '升级条件：以后端当前配置为准。'
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
