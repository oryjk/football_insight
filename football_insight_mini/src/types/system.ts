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
  home_briefing_marquees?: HomeBriefingMarquees
  membership_tier_rules?: MembershipTierRule[]
  /** 首页品牌栏副标题；null 或缺省表示使用内置默认文案。 */
  brand_subtitle?: string | null
}

export interface MiniReviewStatus {
  project_code: string
  version: string
  /** 兼容字段，等同 viewer_is_reviewing。 */
  is_reviewing: boolean
  /** 登记库里该版本的真实审核状态，不受审核豁免影响（旧后端无此字段）。 */
  registry_is_reviewing?: boolean
  /** 当前请求者被当作审核中处理，决定入口显隐（旧后端无此字段）。 */
  viewer_is_reviewing?: boolean
  /** 当前请求者是否因审核豁免名单而看到非审核态（旧后端无此字段）。 */
  viewer_is_exempt?: boolean
  status_text: string
}

export interface SystemConfig {
  mini_program_app_id: string
  mini_program_version: string
  is_under_review: boolean
  matched: boolean
  created_at?: string | null
  updated_at?: string | null
}
