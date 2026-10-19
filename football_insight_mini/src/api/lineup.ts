import type { LineupSlotDto, SavedLineup, SquadResponse } from '../types/lineup'
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
