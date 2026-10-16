import type { TicketWatchGroupedInventorySection, TicketWatchMatchSummary } from '../../types/ticketWatch'
import { isMembershipTierAtLeast } from '../../utils/membershipRules'
import { resolveTicketWatchBlockKey } from './ticketWatchSharedHelpers'
import { selectHistoryMatchesWithData } from './ticketWatchLoadHelpers'

export type TicketWatchInventoryRefluxFreshness = 'strong' | 'weak' | 'none'

export type TicketWatchRecentRefluxBucketKey = 'within3' | 'within10' | 'within30'

export type TicketWatchRecentRefluxPanelMode = 'hidden' | 'locked' | 'unlocked'

export interface TicketWatchRecentRefluxItem {
  block_key: string
  block_name: string
  price: string
  occurrences: number
  latest_time: string
  minutes_ago: number
}

export interface TicketWatchRecentRefluxBucket {
  key: TicketWatchRecentRefluxBucketKey
  title: string
  subtitle: string
  items: TicketWatchRecentRefluxItem[]
}

export type TicketWatchHistoryRefluxTrendTone = 'empty' | 'low' | 'mid' | 'high'

export interface TicketWatchHistoryRefluxTrendPoint {
  match_id: number
  label: string
  teams: string
  total_occurrences: number
  bar_percent: number
  is_selected: boolean
  is_loaded: boolean
  tone: TicketWatchHistoryRefluxTrendTone
}

export interface TicketWatchHistoryRefluxTrend {
  points: TicketWatchHistoryRefluxTrendPoint[]
  maxTotal: number
  averageTotal: number
  latestTotal: number
  loadedCount: number
  totalCount: number
}

export function buildHistoryRefluxTrend(
  matches: TicketWatchMatchSummary[],
  totalsByMatchId: Record<number, number | undefined>,
  selectedMatchId: number | null,
  primaryTeamName: string = '成都蓉城',
): TicketWatchHistoryRefluxTrend {
  const matchesWithData = selectHistoryMatchesWithData(matches, totalsByMatchId, primaryTeamName)
  const loadedTotals = matchesWithData
    .map((match) => totalsByMatchId[match.match_id])
    .filter((value): value is number => typeof value === 'number' && Number.isFinite(value))
    .map((value) => Math.max(0, value))
  const maxTotal = loadedTotals.reduce((selected, value) => Math.max(selected, value), 0)
  const loadedCount = loadedTotals.length
  const averageTotal = loadedCount
    ? Math.round(loadedTotals.reduce((sum, value) => sum + value, 0) / loadedCount)
    : 0

  return {
    points: matchesWithData.map((match) => {
      const rawTotal = totalsByMatchId[match.match_id]
      const isLoaded = typeof rawTotal === 'number' && Number.isFinite(rawTotal)
      const total = isLoaded ? Math.max(0, rawTotal) : 0
      const barPercent = maxTotal > 0 ? Math.round((total / maxTotal) * 100) : 0

      return {
        match_id: match.match_id,
        label: match.match_date.slice(5),
        teams: resolveHistoryRefluxOpponentName(match, primaryTeamName),
        total_occurrences: total,
        bar_percent: barPercent,
        is_selected: selectedMatchId === match.match_id,
        is_loaded: isLoaded,
        tone: resolveHistoryRefluxTrendTone(barPercent, total),
      }
    }),
    maxTotal,
    averageTotal,
    latestTotal: loadedTotals[0] ?? 0,
    loadedCount,
    totalCount: matchesWithData.length,
  }
}

function resolveHistoryRefluxOpponentName(
  match: TicketWatchMatchSummary,
  primaryTeamName: string,
): string {
  if (match.home_team_name === primaryTeamName) {
    return match.away_team_name
  }

  if (match.away_team_name === primaryTeamName) {
    return match.home_team_name
  }

  return `${match.home_team_name} vs ${match.away_team_name}`
}

function resolveHistoryRefluxTrendTone(
  barPercent: number,
  totalOccurrences: number,
): TicketWatchHistoryRefluxTrendTone {
  if (totalOccurrences <= 0) {
    return 'empty'
  }

  if (barPercent >= 75) {
    return 'high'
  }

  if (barPercent >= 35) {
    return 'mid'
  }

  return 'low'
}

export function resolveInventoryRefluxFreshness(
  latestTime: string,
  nowIso = new Date().toISOString(),
): TicketWatchInventoryRefluxFreshness {
  if (!latestTime) {
    return 'none'
  }

  const latestTimestamp = new Date(latestTime).getTime()
  const nowTimestamp = new Date(nowIso).getTime()

  if (!Number.isFinite(latestTimestamp) || !Number.isFinite(nowTimestamp)) {
    return 'none'
  }

  const diffMinutes = (nowTimestamp - latestTimestamp) / 60000

  if (diffMinutes < 0) {
    return 'none'
  }

  if (diffMinutes <= 3) {
    return 'strong'
  }

  if (diffMinutes <= 10) {
    return 'weak'
  }

  return 'none'
}

export function buildRecentRefluxBuckets(
  sections: TicketWatchGroupedInventorySection[],
  nowIso = new Date().toISOString(),
): TicketWatchRecentRefluxBucket[] {
  const nowTimestamp = new Date(nowIso).getTime()
  const buckets: TicketWatchRecentRefluxBucket[] = [
    {
      key: 'within3',
      title: '3 分钟内',
      subtitle: '刚出现',
      items: [],
    },
    {
      key: 'within10',
      title: '10 分钟内',
      subtitle: '仍很近',
      items: [],
    },
    {
      key: 'within30',
      title: '30 分钟内',
      subtitle: '可回看',
      items: [],
    },
  ]

  if (!Number.isFinite(nowTimestamp)) {
    return buckets
  }

  sections
    .flatMap((section) => section.items)
    .filter((item) => item.has_inventory && item.latest_time)
    .forEach((item) => {
      const latestTimestamp = new Date(item.latest_time).getTime()

      if (!Number.isFinite(latestTimestamp)) {
        return
      }

      const diffMinutes = (nowTimestamp - latestTimestamp) / 60000

      if (diffMinutes < 0 || diffMinutes > 30) {
        return
      }

      const bucket = diffMinutes <= 3
        ? buckets[0]
        : diffMinutes <= 10
          ? buckets[1]
          : buckets[2]

      bucket.items.push({
        block_key: resolveTicketWatchBlockKey(item),
        block_name: item.block_name,
        price: item.price,
        occurrences: item.occurrences,
        latest_time: item.latest_time,
        minutes_ago: Math.floor(diffMinutes),
      })
    })

  return buckets.map((bucket) => ({
    ...bucket,
    items: bucket.items.sort((left, right) => {
      const latestDiff = new Date(right.latest_time).getTime() - new Date(left.latest_time).getTime()

      if (latestDiff !== 0) {
        return latestDiff
      }

      return right.occurrences - left.occurrences
    }),
  }))
}

export function resolveRecentRefluxPanelMode(
  hasRecentReflux: boolean,
  membershipTier: string | null | undefined,
): TicketWatchRecentRefluxPanelMode {
  void hasRecentReflux

  if (isMembershipTierAtLeast(membershipTier, 'V6')) {
    return 'unlocked'
  }

  return 'locked'
}

export function resolveRecentRefluxBucketRequiredTier(
  bucketKey: TicketWatchRecentRefluxBucketKey,
): 'V6' | 'V7' | 'V8' {
  if (bucketKey === 'within3') {
    return 'V8'
  }

  if (bucketKey === 'within10') {
    return 'V7'
  }

  return 'V6'
}

export function isRecentRefluxBucketUnlocked(
  bucketKey: TicketWatchRecentRefluxBucketKey,
  membershipTier: string | null | undefined,
): boolean {
  return isMembershipTierAtLeast(membershipTier, resolveRecentRefluxBucketRequiredTier(bucketKey))
}
