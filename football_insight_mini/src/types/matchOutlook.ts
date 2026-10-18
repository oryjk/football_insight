export interface MatchOutlook {
  match_id: number
  home_win: number
  draw: number
  away_win: number
  tau: number | null
  model_version: string
  generated_at: string
  kickoff_at: string
  frozen: boolean
  outcome: 0 | 1 | 2 | null
  schedule_changed: boolean
}
export interface OutlookResponse {
  status: 'ready' | 'unavailable'
  outlook: MatchOutlook | null
}
export interface OutlookRecord {
  season: number
  matches: number
  hits: number
  accuracy: number | null
  rps_model: number | null
  rps_baseline: number | null
  rps_improvement: number | null
}
