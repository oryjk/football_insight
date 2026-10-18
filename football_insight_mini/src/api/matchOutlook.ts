import { request } from '../utils/request'
import type { OutlookResponse, OutlookRecord } from '../types/matchOutlook'
export function getMatchOutlook(matchId: number): Promise<OutlookResponse> {
  return request({
    url: `/live/match-outlook?match_id=${matchId}`,
    silentNetworkError: true,
  })
}
export function getOutlookRecord(season: number): Promise<OutlookRecord> {
  return request({
    url: `/live/match-outlook/record?season=${season}`,
    silentNetworkError: true,
  })
}
