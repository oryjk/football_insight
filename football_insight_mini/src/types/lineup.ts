export type LineupPosition = 'GK' | 'DF' | 'MF' | 'FW'

export interface SquadPlayer {
  player_id: number
  name: string
  shirt_number: number | null
  position: LineupPosition | null
  avatar_url: string | null
  last_season: number
}

export interface SquadResponse {
  team: { team_id: number; team_name: string; avatar_url: string | null }
  active: SquadPlayer[]
  former: SquadPlayer[]
}

export interface LineupSlotDto {
  slot: number
  player_id: number | null
  x: number
  y: number
}

export interface SavedLineup {
  formation: string
  slots: LineupSlotDto[]
  share_code: string
  updated_at: string
}
