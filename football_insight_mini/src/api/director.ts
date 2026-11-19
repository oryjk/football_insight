import type {
  DirectorCandidatesResponse,
  DirectorCircleResponse,
  DirectorCommentDto,
  DirectorContextResponse,
  DirectorPlanDetailResponse,
  DirectorReactionResponse,
  UpsertDirectorPlanPayload,
} from '../types/director'
import { request } from '../utils/request'

export interface DirectorCandidatesQuery {
  teamId: number
  isForeign: boolean
  league?: string
  q?: string
}

export function getDirectorContext(teamId?: number): Promise<DirectorContextResponse> {
  const query = teamId && teamId > 0 ? `?team_id=${teamId}` : ''
  return request<DirectorContextResponse>({
    url: `/director/context${query}`,
    auth: true,
  })
}

export function getDirectorCandidates(params: DirectorCandidatesQuery): Promise<DirectorCandidatesResponse> {
  const parts: string[] = [
    `team_id=${params.teamId}`,
    `is_foreign=${params.isForeign ? 'true' : 'false'}`,
  ]
  if (params.league && params.league !== 'all') {
    parts.push(`league=${encodeURIComponent(params.league)}`)
  }
  const keyword = params.q?.trim()
  if (keyword) {
    parts.push(`q=${encodeURIComponent(keyword)}`)
  }
  return request<DirectorCandidatesResponse>({
    url: `/director/candidates?${parts.join('&')}`,
  })
}

export function upsertDirectorPlan(payload: UpsertDirectorPlanPayload): Promise<DirectorPlanDetailResponse> {
  return request<DirectorPlanDetailResponse>({
    url: '/director/plans/me',
    method: 'PUT',
    auth: true,
    data: payload,
  })
}

export function getDirectorPlan(shareCode: string): Promise<DirectorPlanDetailResponse> {
  return request<DirectorPlanDetailResponse>({
    url: `/director/plans/${encodeURIComponent(shareCode)}`,
    auth: true,
  })
}

export function getDirectorCircle(circleCode: string): Promise<DirectorCircleResponse> {
  return request<DirectorCircleResponse>({
    url: `/director/circles/${encodeURIComponent(circleCode)}`,
    auth: true,
  })
}

export function setDirectorReaction(shareCode: string, value: 1 | -1 | 0): Promise<DirectorReactionResponse> {
  return request<DirectorReactionResponse>({
    url: `/director/plans/${encodeURIComponent(shareCode)}/reaction`,
    method: 'PUT',
    auth: true,
    data: { value },
  })
}

export function postDirectorComment(shareCode: string, content: string): Promise<DirectorCommentDto> {
  return request<DirectorCommentDto>({
    url: `/director/plans/${encodeURIComponent(shareCode)}/comments`,
    method: 'POST',
    auth: true,
    data: { content },
  })
}
