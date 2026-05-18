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
  include_in_reflux_stats?: boolean
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
  block_key?: string
  block_name: string
  price: string
  usable_count: number
  estate: number
}

export interface TicketWatchInventoryEntry {
  block_key?: string
  block_name: string
  occurrences: number
  latest_time: string
}

export interface TicketWatchBlockInterest {
  block_key?: string
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
  block_key?: string
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

export interface RefluxSubscriptionPlan {
  code: string
  scope: 'single_match' | 'season' | 'lifetime'
  team_code: string
  season?: number | null
  title: string
  description: string
  price_cents: number
  expires_at?: string | null
}

export interface RefluxNotificationTarget {
  channel: string
  target: string
}

export interface RefluxSubscriptionSummary {
  scope: 'single_match' | 'season' | 'lifetime'
  team_code: string
  season?: number | null
  match_id?: number | null
  starts_at: string
  expires_at?: string | null
}

export interface RefluxSubscriptionPlansResponse {
  plans: RefluxSubscriptionPlan[]
  active_subscriptions: RefluxSubscriptionSummary[]
  email_target?: RefluxNotificationTarget | null
}

export interface RefluxSubscriptionStatusResponse {
  subscribed: boolean
  active_subscriptions: RefluxSubscriptionSummary[]
  email_target?: RefluxNotificationTarget | null
}

export interface CreateRefluxSubscriptionOrderResponse {
  order_no: string
  params: {
    timeStamp: string
    nonceStr: string
    package: string
    signType: string
    paySign: string
  }
}
