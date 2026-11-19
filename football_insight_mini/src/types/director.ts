export type DirectorPosition = 'GK' | 'DF' | 'MF' | 'FW'

export type DirectorPlayerSource = 'squad' | 'pool' | 'custom'

export type DirectorLeagueFilter =
  | 'all'
  | 'K'
  | 'J1'
  | 'EUR'
  | 'ACL'
  | 'CL1'
  | 'SPL'
  | 'MLS'
  | 'CSL'
  | 'FREE'
  | 'CUSTOM'

export interface PlanPlayerItem {
  key: string
  source: DirectorPlayerSource
  player_id: number | null
  candidate_code: string | null
  player_name: string
  position: DirectorPosition
  country: string
  is_foreign: boolean
  league_tag: string
  current_club: string
  avatar_url: string | null
  market_value_eur?: number | null
  birth_date?: string | null
  contract_until?: string | null
}

export interface DirectorWindowDto {
  id: number
  code: string
  name: string
  opens_at: string
  closes_at: string
}

export interface DirectorTeamDto {
  team_id: number
  team_name: string
  logo_url: string | null
}

export interface DirectorSquadPlayerDto {
  player_id: number
  player_name: string
  shirt_number: number | null
  position: DirectorPosition
  country: string
  avatar_url: string | null
  market_value_eur?: number | null
  birth_date?: string | null
  contract_until?: string | null
}

export interface DirectorAuthorDto {
  user_id: string
  nickname: string
  avatar_url: string | null
}

export interface DirectorPlanDto {
  id: string
  window_code: string
  window_name: string
  team_id: number
  team_name: string
  share_code: string
  circle_code: string
  author: DirectorAuthorDto
  is_mine: boolean
  foreign_stays: PlanPlayerItem[]
  foreign_leaves: PlanPlayerItem[]
  foreign_ins: PlanPlayerItem[]
  domestic_ins: PlanPlayerItem[]
  domestic_outs: PlanPlayerItem[]
  style_tag: string
  manifesto: string
  likes_count: number
  dislikes_count: number
  my_reaction: 1 | -1 | 0 | null
  comments_count: number
  updated_at: string
}

export interface DirectorContextResponse {
  window: DirectorWindowDto
  team: DirectorTeamDto
  foreign_quota: number
  max_domestic_ins: number
  max_domestic_outs: number
  squad_foreign: DirectorSquadPlayerDto[]
  squad_domestic: DirectorSquadPlayerDto[]
  my_plan: DirectorPlanDto | null
  team_plan_count: number
}

export interface DirectorCandidateDto {
  key: string
  source: DirectorPlayerSource
  player_id: number | null
  candidate_code: string | null
  player_name: string
  position: DirectorPosition
  country: string
  is_foreign: boolean
  league_tag: string
  current_club: string
  tags: string[]
  avatar_url: string | null
  market_value_eur?: number | null
  birth_date?: string | null
  contract_until?: string | null
}

export interface DirectorCandidatesResponse {
  items: DirectorCandidateDto[]
}

export interface UpsertDirectorPlayerInput {
  source: DirectorPlayerSource
  candidate_code: string | null
  player_id: number | null
  player_name: string | null
  position: DirectorPosition | null
  league_tag: string | null
  current_club: string | null
}

export interface UpsertDirectorPlanPayload {
  team_id: number
  circle_code: string | null
  parent_share_code: string | null
  foreign_stay_ids: number[]
  foreign_leave_ids: number[]
  foreign_ins: UpsertDirectorPlayerInput[]
  domestic_ins: UpsertDirectorPlayerInput[]
  domestic_outs: number[]
  manifesto: string
}

export interface DirectorCommentDto {
  id: string
  author: DirectorAuthorDto
  content: string
  created_at: string
}

export interface DirectorLeaveDisagreementDto {
  player: PlanPlayerItem
  target_action: 'stay' | 'leave'
  my_action: 'stay' | 'leave'
}

export interface DirectorComparisonDto {
  my_share_code: string
  overlap_percent: number
  common_foreign: PlanPlayerItem[]
  only_in_target: PlanPlayerItem[]
  only_in_mine: PlanPlayerItem[]
  leave_disagreements: DirectorLeaveDisagreementDto[]
  summary_line: string
}

export interface DirectorPlanDetailResponse {
  plan: DirectorPlanDto
  comments: DirectorCommentDto[]
  comparison: DirectorComparisonDto | null
}

export interface DirectorCircleTopForeignDto {
  player: PlanPlayerItem
  pick_count: number
  pick_rate: number
}

export interface DirectorCircleTopLeaveDto {
  player: PlanPlayerItem
  leave_count: number
  leave_rate: number
}

export interface DirectorCirclePlanItemDto {
  share_code: string
  author: DirectorAuthorDto
  style_tag: string
  manifesto: string
  foreign_count: number
  likes_count: number
  dislikes_count: number
  updated_at: string
}

export interface DirectorCircleResponse {
  circle_code: string
  window_name: string
  team_id: number
  team_name: string
  member_count: number
  unlocked: boolean
  top_foreign: DirectorCircleTopForeignDto[]
  top_leaves: DirectorCircleTopLeaveDto[]
  plans: DirectorCirclePlanItemDto[]
}

export interface DirectorReactionResponse {
  likes_count: number
  dislikes_count: number
  my_reaction: 1 | -1 | 0
}
