export type AiChatMode = 'backend_proxy' | 'frontend_direct'

export interface HomeBriefingMarquees {
  leader: string[]
  scorer: string[]
  assist: string[]
}

export interface MembershipTierRule {
  code: string
  kind: string
  min_referrals?: number
  ticket_watch_poll_interval_seconds: number
}

export interface PublicSystemConfig {
  wechat_login_enabled: boolean
  ai_chat_mode?: AiChatMode
  home_briefing_marquees?: HomeBriefingMarquees
  membership_tier_rules?: MembershipTierRule[]
}

export interface SystemConfig {
  mini_program_app_id: string
  mini_program_version: string
  is_under_review: boolean
  matched: boolean
  created_at?: string | null
  updated_at?: string | null
}
