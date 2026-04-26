import type {
  TicketWatchBlockInterest,
  TicketWatchCurrentBoardResponse,
  TicketWatchCurrentMatchResponse,
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
  TicketWatchRegion,
  TicketWatchTrackedInterest,
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

export function getTicketWatchRegions(): Promise<TicketWatchRegion[]> {
  return request<TicketWatchRegion[]>({
    url: '/ticket-watch/regions',
  })
}

export function getTicketWatchInventory(matchId: number): Promise<TicketWatchInventoryEntry[]> {
  return getTicketWatchInventorySince(matchId)
}

export function buildTicketWatchInventoryUrl(
  matchId: number,
  since?: string | null,
  fallbackMatchId?: number | null,
): string {
  const queryParts: string[] = []

  if (since) {
    queryParts.push(`since=${encodeURIComponent(since)}`)
  }

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
