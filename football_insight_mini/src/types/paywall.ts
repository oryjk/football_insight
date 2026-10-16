export type PaywallSurface =
  | 'ticket_watch_recent_reflux_lock'
  | 'ticket_watch_match_id_sheet'
  | 'insights_ticket_watch_entry'
  | 'user_membership_entry'

export type PaywallAction = 'impression' | 'tap'

export interface PaywallEventPayload {
  surface: PaywallSurface
  action: PaywallAction
  membership_tier?: string
  detail: Record<string, string | number>
}

export interface PaywallEventsPayload {
  events: PaywallEventPayload[]
}

export interface PaywallContext {
  membershipTier?: string | null
  miniVersion?: string
  platform?: string
}
