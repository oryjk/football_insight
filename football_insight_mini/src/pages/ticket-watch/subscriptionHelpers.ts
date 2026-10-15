import type { RefluxSubscriptionPlan, RefluxSubscriptionSummary } from '../../types/ticketWatch'

export {
  formatTicketWatchPollIntervalLabel,
  isMembershipTierAtLeast,
  normalizeTicketWatchPollIntervalSeconds,
} from '../../utils/membershipRules'

export function isValidNotificationEmail(email: string): boolean {
  const normalized = email.trim()
  return normalized.length > 0 && normalized.length <= 254 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)
}

export function formatRefluxSubscriptionPrice(priceCents: number): string {
  if (!Number.isFinite(priceCents)) {
    return '¥0'
  }

  const yuan = priceCents / 100
  return Number.isInteger(yuan) ? `¥${yuan}` : `¥${yuan.toFixed(2)}`
}

export function selectPurchasableRefluxSubscriptionPlans(
  plans: RefluxSubscriptionPlan[],
  activeSubscriptions: RefluxSubscriptionSummary[],
  teamCode: string,
  matchId: number,
): RefluxSubscriptionPlan[] {
  const normalizedTeamCode = teamCode.trim().toLowerCase()
  const hasCurrentSingleMatchSubscription = activeSubscriptions.some((subscription) => (
    subscription.scope === 'single_match'
    && subscription.match_id === matchId
    && subscription.team_code.trim().toLowerCase() === normalizedTeamCode
  ))

  if (!hasCurrentSingleMatchSubscription) {
    return plans
  }

  return plans.filter((plan) => plan.scope !== 'single_match')
}

export function isRefluxSubscriptionActiveForCurrentMatch(
  activeSubscriptions: RefluxSubscriptionSummary[],
  teamCode: string,
  season: number,
  matchId: number,
): boolean {
  const normalizedTeamCode = teamCode.trim().toLowerCase()

  return activeSubscriptions.some((subscription) => {
    if (subscription.team_code.trim().toLowerCase() !== normalizedTeamCode) {
      return false
    }

    if (subscription.scope === 'single_match') {
      return subscription.match_id === matchId
    }

    if (subscription.scope === 'season') {
      return subscription.season === season
    }

    return subscription.scope === 'lifetime'
  })
}

export function formatTicketWatchMembershipBadgeTier(membershipTier?: string | null): string {
  const normalizedTier = membershipTier?.trim().toUpperCase() || 'V1'
  return `${normalizedTier} 会员`
}
