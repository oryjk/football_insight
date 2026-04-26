import type {
  InsightOverviewResponse,
  MatchListResponse,
  RankingsViewResponse,
  RoundReference,
  TeamInsightsResponse,
  ViewMode,
} from '../types/insight'
import { request } from '../utils/request'

interface ViewRequest {
  mode: ViewMode
  season: number
  roundNumber: number | null
}

export function getAvailableRounds(season: number): Promise<RoundReference[]> {
  return request<RoundReference[]>({
    url: `/rounds?season=${season}`,
  })
}

export function getOverview(payload: ViewRequest): Promise<InsightOverviewResponse> {
  const path = payload.mode === 'live'
    ? '/live/overview'
    : `/rounds/${payload.season}/${payload.roundNumber}/overview`

  return request<InsightOverviewResponse>({ url: path })
}

export function getRankings(payload: ViewRequest): Promise<RankingsViewResponse> {
  const path = payload.mode === 'live'
    ? '/live/rankings'
    : `/rounds/${payload.season}/${payload.roundNumber}/rankings`

  return request<RankingsViewResponse>({ url: path })
}

export function getMatches(payload: ViewRequest): Promise<MatchListResponse> {
  const path = payload.mode === 'live'
    ? '/live/matches'
    : `/rounds/${payload.season}/${payload.roundNumber}/matches`

  return request<MatchListResponse>({ url: path })
}

export function getLiveTeamInsights(): Promise<TeamInsightsResponse> {
  return request<TeamInsightsResponse>({ url: '/live/team-insights' })
}
