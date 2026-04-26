export interface SupportTeam {
  team_id: number
  team_name: string
  avatar_storage_url: string | null
  rank_no: number | null
}

export interface SupportMatchTeam {
  team_id: number
  team_name: string
  avatar_storage_url: string | null
  score: string
  support_count: number
  support_share_pct: number
  season_support_rank: number | null
}

export interface SupportViewerState {
  favorite_team_id: number | null
  supported_team_id: number | null
  has_supported: boolean
  can_support: boolean
}

export interface SupportMatchDetail {
  match_id: number
  season: number
  round_number: number
  match_date: string
  match_time: string
  kickoff_at: string
  status: string
  support_window_status: 'locked' | 'open' | 'closed'
  countdown_seconds: number
  total_support_count: number
  home_team: SupportMatchTeam
  away_team: SupportMatchTeam
  viewer: SupportViewerState
}

export interface SupportProfile {
  favorite_team: SupportTeam | null
  next_match: SupportMatchDetail | null
}

export interface SetFavoriteTeamPayload {
  team_id: number
}

export interface CastSupportVotePayload {
  supported_team_id: number
}
