export interface MembershipProduct {
  price_cents: number
  title: string
  subtitle: string
  description: string
  products?: MembershipProductOption[]
}

export interface MembershipProductOption {
  target_tier: string
  price_cents: number
  original_price_cents?: number
  pay_price_cents?: number
  upgrade_fee_cents?: number
  title: string
  subtitle: string
  description: string
  membership_duration_months?: number | null
  expiry_rule?: string
  duration_label?: string
  group_key?: string
  is_pass?: boolean
}

export interface VirtualPayParams {
  mode: 'short_series_goods'
  signData: string
  paySig: string
  signature: string
}

/** Stale ordinary responses are rejected before charging. */
export type DigitalPaymentParams = VirtualPayParams | {
  timeStamp: string
  nonceStr: string
  package: string
  signType: string
  paySign: string
}
