export interface CurrentUser {
  id: string
  display_name: string
  account_identifier: string
  invite_code?: string | null
  avatar_url: string | null
  has_wechat_binding: boolean
  membership_tier: string
  membership_expires_at?: string | null
  membership_benefits_enabled?: boolean
  ticket_watch_poll_interval_seconds?: number
  created_at: string
}

export interface AuthResponse {
  access_token: string
  expires_at: string
  user: CurrentUser
}

export interface MiniWechatBindingRequiredResponse {
  status: 'binding_required'
  bind_token: string
  expires_at: string
  display_name: string | null
  avatar_url: string | null
}

export interface MiniWechatAuthenticatedResponse extends AuthResponse {
  status: 'authenticated'
}

export type MiniWechatLoginResponse =
  | MiniWechatAuthenticatedResponse
  | MiniWechatBindingRequiredResponse

export interface RegisterPayload {
  invite_code: string
  referral_code?: string | null
  account_identifier: string
  password: string
}

export interface LoginPayload {
  account_identifier: string
  password: string
}

export interface ResetPasswordPayload {
  invite_code: string
  account_identifier: string
  password: string
}

export interface MiniWechatBindPayload {
  bind_token: string
  invite_code: string
  referral_code?: string | null
  display_name: string
  avatar_data_url: string
}
