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
