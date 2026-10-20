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

export interface SavedLineupDraft {
  formation: string
  slots: LineupSlotDto[]
  updated_at: string
}

export interface LineupEditorResponse {
  lineup: SavedLineup | null
  draft: SavedLineupDraft | null
}

export interface SavedLineup {
  formation: string
  slots: LineupSlotDto[]
  share_code: string
  updated_at: string
}


export type LineupSort = 'latest' | 'likes'
export interface LineupAuthor { display_name: string; avatar_url: string | null }
export interface LineupLikes { likes_count: number; liked_by_me: boolean; disliked_by_me?: boolean }
export type LineupReaction = 'like' | 'dislike'
export interface PublicLineupSummary extends LineupLikes {
  share_code: string
  formation: string
  author: LineupAuthor
  created_at: string
  updated_at: string
}
export interface PublicLineup extends PublicLineupSummary {
  team: SquadResponse['team']
  slots: LineupSlotDto[]
  players: Array<SquadPlayer & { former: boolean }>
}
export interface TeamLineups {
  team: SquadResponse['team']
  items: PublicLineupSummary[]
  total: number
  page: number
  page_size: number
}
export interface LineupCodeImage { image_base64: string; mime_type: 'image/png' | 'image/jpeg' }

export interface LineupConsensusPlayer {
  player_id: number
  name: string
  avatar_url: string | null
  shirt_number: number | null
  position: LineupPosition | null
  /** 在共识里占用的位置类；补齐的球员可能与名单位置不同 */
  group: LineupPosition
  pick_rate: number
}
export interface LineupConsensus {
  team: SquadResponse['team']
  total: number
  threshold: number
  unlocked: boolean
  formation: string | null
  formation_share: number
  players: LineupConsensusPlayer[]
}
