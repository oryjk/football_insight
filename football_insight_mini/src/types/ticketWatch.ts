export interface TicketWatchMatchSummary {
  match_id: number
  external_match_id: string
  round_number: number
  sale_start_at?: string | null
  match_date: string
  match_time: string
  kickoff_at: string
  home_team_name: string
  away_team_name: string
  is_current: boolean
}

export interface TicketWatchCurrentMatchResponse {
  current_match: TicketWatchMatchSummary | null
  group_ticket_active: boolean
  message: string
}

export interface TicketWatchCurrentBoardResponse {
  current_match: TicketWatchMatchSummary | null
  group_ticket_active: boolean
  message: string
  inventory: TicketWatchInventoryEntry[]
  block_interests: TicketWatchBlockInterest[]
  tracked_interests: TicketWatchTrackedInterest[]
}

export interface TicketWatchRegion {
  block_name: string
  price: string
  usable_count: number
  estate: number
}

export interface TicketWatchInventoryEntry {
  block_name: string
  occurrences: number
  latest_time: string
}

export interface TicketWatchBlockInterest {
  block_name: string
  interested_user_count: number
  viewer_interested: boolean
}

export interface TicketWatchTrackedInterest {
  block_name: string
  started_at: string
  first_inventory_at?: string | null
}

export interface TicketWatchGroupedInventoryItem {
  block_name: string
  price: string
  occurrences: number
  latest_time: string
  has_inventory: boolean
  interested_user_count: number
  viewer_interested: boolean
}

export interface TicketWatchGroupedInventorySection {
  price: string
  region_count: number
  available_region_count: number
  total_occurrences: number
  items: TicketWatchGroupedInventoryItem[]
}
