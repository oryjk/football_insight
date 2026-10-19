import type { LineupSlotDto, SavedLineupDraft, LineupEditorResponse, SavedLineup, SquadResponse, LineupSort, TeamLineups, PublicLineup, LineupLikes, LineupCodeImage } from '../types/lineup'
import { request } from '../utils/request'

export function getSquad(teamId: number): Promise<SquadResponse> {
  return request<SquadResponse>({ url: `/lineups/squads/${teamId}` })
}

export function getMyLineup(teamId: number): Promise<{ lineup: SavedLineup | null }> {
  return request<{ lineup: SavedLineup | null }>({ url: `/lineups/me/${teamId}`, auth: true })
}

export function saveMyLineup(
  teamId: number,
  payload: { formation: string; slots: LineupSlotDto[] },
): Promise<{ lineup: SavedLineup }> {
  return request<{ lineup: SavedLineup }>({
    url: `/lineups/me/${teamId}`,
    method: 'PUT',
    auth: true,
    data: payload,
  })
}


export function getTeamLineups(teamId: number, sort: LineupSort, page = 1, pageSize = 20): Promise<TeamLineups> {
  return request({ url: `/lineups/teams/${teamId}?sort=${sort}&page=${page}&page_size=${pageSize}`, auth: true })
}
export function getSharedLineup(code: string): Promise<PublicLineup> {
  return request({ url: `/lineups/shared/${encodeURIComponent(code)}`, auth: true })
}
export function setLineupLike(code: string, liked: boolean): Promise<LineupLikes> {
  return request({ url: `/lineups/shared/${encodeURIComponent(code)}/like`, method: liked ? 'PUT' : 'DELETE', auth: true })
}
export function setLineupDislike(code: string, disliked: boolean): Promise<LineupLikes> {
  return request({ url: `/lineups/shared/${encodeURIComponent(code)}/dislike`, method: disliked ? 'PUT' : 'DELETE', auth: true })
}
export function getLineupCode(code: string): Promise<LineupCodeImage> {
  return request({ url: `/lineups/shared/${encodeURIComponent(code)}/wxacode`, auth: true })
}

/** 私密编辑器与公开快照分别读取，保存草稿不会更新公开版本。 */
export function getMyLineupEditor(teamId: number): Promise<LineupEditorResponse> {
  return request({ url: `/lineups/me/${teamId}/editor`, auth: true })
}
export function saveMyLineupDraft(teamId: number, payload: { formation: string; slots: LineupSlotDto[] }): Promise<{ draft: SavedLineupDraft }> {
  return request({ url: `/lineups/me/${teamId}/draft`, method: 'PUT', auth: true, data: payload })
}
export function publishMyLineup(teamId: number, payload: { formation: string; slots: LineupSlotDto[] }): Promise<{ lineup: SavedLineup; draft: SavedLineupDraft }> {
  return request({ url: `/lineups/me/${teamId}/publish`, method: 'PUT', auth: true, data: payload })
}
