export type TransferIntent = 'out' | 'stay' | 'in'
export type TransferVoteValue = 1 | -1
export interface TransferTeam { team_id: number; team_name: string; avatar_url: string | null }
export interface TransferWindow { code: string; name: string; opens_at: string; closes_at: string; open: boolean }
export interface TransferPlayer {
  player_id: number
  name: string
  avatar_url: string | null
  position: 'GK' | 'DF' | 'MF' | 'FW' | null
  shirt_number: number | null
  current_team: TransferTeam | null
  departed: boolean
}
export interface TransferCard {
  share_code: string
  intent: TransferIntent
  returning: boolean
  player: TransferPlayer
  up: number
  down: number
  my_vote: -1 | 0 | 1
  rank: number | null
  created_at: string
}
export interface TransferTally { up: number; down: number; my_vote: -1 | 0 | 1 }
export interface TransferBoardSlice { intent: TransferIntent; total: number; items: TransferCard[] }
export interface TransferSummary { team: TransferTeam; window: TransferWindow | null; can_vote: boolean; boards: TransferBoardSlice[] }
export interface TransferBoard {
  team: TransferTeam
  window: TransferWindow | null
  intent: TransferIntent
  page: number
  page_size: number
  total: number
  can_vote: boolean
  items: TransferCard[]
}
export interface TransferCardView { card: TransferCard; team: TransferTeam; window: TransferWindow; can_vote: boolean }
export interface TransferCandidate {
  player_id: number
  name: string
  shirt_number: number | null
  position: TransferPlayer['position']
  avatar_url: string | null
  current_team: TransferTeam | null
}
export interface TransferCandidateGroup { key: string; title: string; players: TransferCandidate[] }
export interface TransferCandidates { intent: TransferIntent; groups: TransferCandidateGroup[] }
export interface TransferNominateResult { created: boolean; view: TransferCardView }
