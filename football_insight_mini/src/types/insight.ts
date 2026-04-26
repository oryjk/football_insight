export type ViewMode = 'live' | 'round'

export interface RoundReference {
  season: number
  round_number: number
  finalized_at: string | null
  status: 'completed' | 'current' | 'upcoming' | string
  total_matches: number
  completed_matches: number
}

export interface OverviewStanding {
  rank_no: number
  team_id: number
  team_name: string
  points: number
  avatar_storage_url: string | null
}

export interface OverviewMatch {
  match_id: number
  round_number: number
  match_date: string
  match_time: string
  home_team_name: string
  away_team_name: string
  home_score: string
  away_score: string
}

export interface OverviewPlayer {
  rank_no: number
  player_id: number
  player_name: string
  team_name: string
  score_value: string
  avatar_storage_url: string | null
}

export interface InsightSummary {
  headline: string
  summary: string
  bullets: string[]
  focus_match_id: number | null
}

export interface InsightOverviewResponse {
  view_kind: string
  round_number: number | null
  current_season: number
  latest_scrape_finished_at: string | null
  total_matches: number
  total_teams: number
  total_players: number
  player_ranking_categories: number
  team_ranking_categories: number
  standings_top: OverviewStanding[]
  recent_matches: OverviewMatch[]
  top_scorers: OverviewPlayer[]
  insight_summary: InsightSummary | null
}

export interface TeamRankingEntry {
  rank_no: number
  team_id: number
  team_name: string
  score_value: string
  avatar_storage_url: string | null
}

export interface TeamRankingCategory {
  slug: string
  label: string
  item_id: number
  entries: TeamRankingEntry[]
}

export interface PlayerRankingEntry {
  rank_no: number
  player_id: number
  player_name: string
  team_id: number
  team_name: string
  score_value: string
  penalty_value: string | null
  avatar_storage_url: string | null
}

export interface PlayerRankingCategory {
  slug: string
  label: string
  item_id: number
  entries: PlayerRankingEntry[]
}

export interface StandingsTableEntry {
  rank_no: number
  team_id: number
  team_name: string
  played: number
  wins: number
  draws: number
  losses: number
  goals_for: number
  goals_against: number
  goal_difference: number
  points: number
  points_without_penalty: number
  points_adjustment: number
  avatar_storage_url: string | null
}

export interface StandingsTable {
  slug: string
  label: string
  note: string
  entries: StandingsTableEntry[]
}

export interface RankingsViewResponse {
  view_kind: string
  round_number: number | null
  current_season: number
  standings_tables: StandingsTable[]
  team_categories: TeamRankingCategory[]
  player_categories: PlayerRankingCategory[]
}

export interface MatchCard {
  match_id: number
  round_number: number
  match_date: string
  match_time: string
  status: 'scheduled' | 'live' | 'finished' | string
  home_team_id: number
  home_team_name: string
  home_score: string
  away_team_id: number
  away_team_name: string
  away_score: string
  home_team_avatar: string | null
  away_team_avatar: string | null
  leisu_match_id: number | null
  home_corners: number | null
  away_corners: number | null
  corner_source: string | null
  technical_stats: MatchTechnicalStat[]
}

export interface MatchListResponse {
  view_kind: string
  round_number: number | null
  current_season: number
  matches: MatchCard[]
}

export interface MatchTechnicalStat {
  slug: string
  label: string
  home_value: number
  away_value: number
  unit: string | null
}

export interface OpponentContribution {
  opponent_team_id: number
  opponent_team_name: string
  opponent_avatar_storage_url: string | null
  goals: number
  share: number
}

export interface PlayerContribution {
  player_id: number | null
  player_name: string
  avatar_storage_url: string | null
  goals: number
  share: number
}

export interface AssistContribution {
  player_id: number | null
  player_name: string
  avatar_storage_url: string | null
  assists: number
  share: number
}

export interface TeamInsightTeam {
  team_id: number
  team_name: string
  rank_no: number
  avatar_storage_url: string | null
}

export interface TeamInsight {
  team_id: number
  team_name: string
  rank_no: number
  avatar_storage_url: string | null
  goals_for_total: number
  goals_against_total: number
  goals_for_by_opponent: OpponentContribution[]
  goals_for_by_player: PlayerContribution[]
  assists_for_by_player: AssistContribution[]
  goals_against_by_opponent: OpponentContribution[]
}

export interface TeamInsightsResponse {
  view_kind: string
  round_number: number | null
  current_season: number
  teams: TeamInsightTeam[]
  insights: TeamInsight[]
}
