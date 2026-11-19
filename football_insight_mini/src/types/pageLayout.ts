export type PageLayoutKey =
  | 'home'
  | 'rankings'
  | 'insights'
  | 'seat_swap'
  | 'user'
  | 'ticket_watch'
  | 'season_recap'
  | 'membership_purchase'
  | 'support'
  | 'topics'
  | 'student'
  | 'user_settings'
  | 'team_board'
  | 'insights_tactics'
  | 'insights_transfer'
  | 'lineup'
  | 'lineup_list'
  | 'transfer_board'
  | 'director'
  | 'topic_detail'
  | 'lineup_view'
  | 'transfer_view'
  | 'telegraph_entry'
  | 'telegraph_comment'
  | 'director_view'

export interface PageLayoutBlock {
  key: string
  visible: boolean
  background_image_url: string | null
}

export interface PublicPageLayout {
  page_key: PageLayoutKey
  background_image_url: string | null
  blocks: PageLayoutBlock[]
  version: number
  updated_at: string | null
}

export interface PublicPageLayoutsResponse {
  items: PublicPageLayout[]
}
