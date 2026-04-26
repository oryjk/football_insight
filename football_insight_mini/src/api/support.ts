import type {
  CastSupportVotePayload,
  SetFavoriteTeamPayload,
  SupportMatchDetail,
  SupportProfile,
  SupportTeam,
} from '../types/support'
import { request } from '../utils/request'

export function listSupportTeams(): Promise<SupportTeam[]> {
  return request<SupportTeam[]>({
    url: '/support/teams',
  })
}

export function getSupportProfile(): Promise<SupportProfile> {
  return request<SupportProfile>({
    url: '/support/profile',
    auth: true,
  })
}

export function setFavoriteTeam(payload: SetFavoriteTeamPayload): Promise<SupportTeam> {
  return request<SupportTeam>({
    url: '/support/favorite-team',
    method: 'PUT',
    auth: true,
    data: payload,
  })
}

export function getMatchSupportDetail(matchId: number): Promise<SupportMatchDetail> {
  return request<SupportMatchDetail>({
    url: `/support/matches/${matchId}`,
    auth: true,
  })
}

export function castSupportVote(
  matchId: number,
  payload: CastSupportVotePayload,
): Promise<SupportMatchDetail> {
  return request<SupportMatchDetail>({
    url: `/support/matches/${matchId}/votes`,
    method: 'POST',
    auth: true,
    data: payload,
  })
}
