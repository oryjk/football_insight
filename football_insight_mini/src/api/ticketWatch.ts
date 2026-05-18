import type {
  TicketWatchBlockInterest,
  TicketWatchCurrentBoardResponse,
  TicketWatchCurrentMatchResponse,
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
  TicketWatchRegion,
  TicketWatchTrackedInterest,
  RefluxSubscriptionPlansResponse,
  RefluxSubscriptionStatusResponse,
  CreateRefluxSubscriptionOrderResponse,
} from '../types/ticketWatch'
import { request } from '../utils/request'

export function getCurrentTicketWatchMatch(): Promise<TicketWatchCurrentMatchResponse> {
  return request<TicketWatchCurrentMatchResponse>({
    url: '/ticket-watch/current-match',
  })
}

export function buildCurrentTicketWatchBoardUrl(): string {
  return '/ticket-watch/current-board'
}

export function getCurrentTicketWatchBoard(): Promise<TicketWatchCurrentBoardResponse> {
  return request<TicketWatchCurrentBoardResponse>({
    url: buildCurrentTicketWatchBoardUrl(),
    auth: true,
  })
}

export function getTicketWatchMatches(): Promise<TicketWatchMatchSummary[]> {
  return request<TicketWatchMatchSummary[]>({
    url: '/ticket-watch/matches',
  })
}

export function getYukunTicketWatchMatches(): Promise<TicketWatchMatchSummary[]> {
  return request<TicketWatchMatchSummary[]>({
    url: '/ticket-watch/yukun/matches',
  })
}

export function getYukunCurrentTicketWatchMatch(): Promise<TicketWatchCurrentMatchResponse> {
  return request<TicketWatchCurrentMatchResponse>({
    url: '/ticket-watch/yukun/current-match',
  })
}

export function getTicketWatchRegions(): Promise<TicketWatchRegion[]> {
  return request<TicketWatchRegion[]>({
    url: '/ticket-watch/regions',
  })
}

export function getTicketWatchInventory(
  matchId: number,
  since: string,
  fallbackMatchId?: number | null,
): Promise<TicketWatchInventoryEntry[]> {
  return getTicketWatchInventorySince(matchId, since, fallbackMatchId)
}

export function buildTicketWatchInventoryUrl(
  matchId: number,
  since?: string | null,
  fallbackMatchId?: number | null,
): string {
  const normalizedSince = since?.trim()
  if (!normalizedSince) {
    throw new Error('sale_start_at is required to build inventory since; refusing full inventory query')
  }

  const queryParts: string[] = []

  queryParts.push(`since=${encodeURIComponent(normalizedSince)}`)

  if (fallbackMatchId && fallbackMatchId !== matchId) {
    queryParts.push(`fallback_match_id=${encodeURIComponent(String(fallbackMatchId))}`)
  }

  const query = queryParts.join('&')

  return `/ticket-watch/matches/${matchId}/inventory${query ? `?${query}` : ''}`
}

export function getTicketWatchInventorySince(
  matchId: number,
  since?: string | null,
  fallbackMatchId?: number | null,
): Promise<TicketWatchInventoryEntry[]> {
  return request<TicketWatchInventoryEntry[]>({
    url: buildTicketWatchInventoryUrl(matchId, since, fallbackMatchId),
  })
}

export function buildTicketWatchInterestsUrl(matchId: number): string {
  return `/ticket-watch/matches/${matchId}/interests`
}

export function getTicketWatchBlockInterests(matchId: number): Promise<TicketWatchBlockInterest[]> {
  return request<TicketWatchBlockInterest[]>({
    url: buildTicketWatchInterestsUrl(matchId),
    auth: true,
  })
}

export function getTicketWatchTrackedInterests(matchId: number): Promise<TicketWatchTrackedInterest[]> {
  return request<TicketWatchTrackedInterest[]>({
    url: `/ticket-watch/matches/${matchId}/tracked-interests`,
    auth: true,
  })
}

export function toggleTicketWatchBlockInterest(
  matchId: number,
  blockName: string,
): Promise<TicketWatchBlockInterest> {
  return request<TicketWatchBlockInterest>({
    url: `/ticket-watch/matches/${matchId}/interests/toggle`,
    method: 'POST',
    data: {
      block_name: blockName,
    },
    auth: true,
  })
}

export function buildYukunTicketWatchInventoryUrl(
  matchId: number,
  since?: string | null,
): string {
  const normalizedSince = since?.trim()
  if (!normalizedSince) {
    throw new Error('sale_start_at is required to build inventory since; refusing full inventory query')
  }
  return `/ticket-watch/yukun/matches/${matchId}/inventory?since=${encodeURIComponent(normalizedSince)}`
}

export function getYukunTicketWatchInventory(
  matchId: number,
  since?: string | null,
): Promise<TicketWatchInventoryEntry[]> {
  return request<TicketWatchInventoryEntry[]>({
    url: buildYukunTicketWatchInventoryUrl(matchId, since),
  })
}

export function buildYukunTicketWatchRegionsUrl(
  matchId: number,
  since?: string | null,
): string {
  const normalizedSince = since?.trim()
  if (!normalizedSince) {
    throw new Error('sale_start_at is required to build regions since; refusing full regions query')
  }
  return `/ticket-watch/yukun/matches/${matchId}/regions?since=${encodeURIComponent(normalizedSince)}`
}

export function getYukunTicketWatchRegions(
  matchId: number,
  since?: string | null,
): Promise<TicketWatchRegion[]> {
  return request<TicketWatchRegion[]>({
    url: buildYukunTicketWatchRegionsUrl(matchId, since),
  })
}

export function getRefluxSubscriptionPlans(
  teamCode: string,
  matchId?: number | null,
): Promise<RefluxSubscriptionPlansResponse> {
  const query = [`team_code=${encodeURIComponent(teamCode)}`]
  if (matchId) {
    query.push(`match_id=${encodeURIComponent(String(matchId))}`)
  }

  return request<RefluxSubscriptionPlansResponse>({
    url: `/ticket-watch/reflux-subscriptions/plans?${query.join('&')}`,
    auth: true,
  })
}

export function getRefluxSubscriptionStatus(
  teamCode: string,
  season: number,
  matchId?: number | null,
): Promise<RefluxSubscriptionStatusResponse> {
  const query = [
    `team_code=${encodeURIComponent(teamCode)}`,
    `season=${encodeURIComponent(String(season))}`,
  ]
  if (matchId) {
    query.push(`match_id=${encodeURIComponent(String(matchId))}`)
  }

  return request<RefluxSubscriptionStatusResponse>({
    url: `/ticket-watch/reflux-subscriptions/status?${query.join('&')}`,
    auth: true,
  })
}

export function createRefluxSubscriptionOrder(input: {
  plan_code: string
  team_code: string
  match_id?: number | null
  email: string
}): Promise<CreateRefluxSubscriptionOrderResponse> {
  return request<CreateRefluxSubscriptionOrderResponse>({
    url: '/ticket-watch/reflux-subscriptions/order',
    method: 'POST',
    auth: true,
    data: input,
  })
}
