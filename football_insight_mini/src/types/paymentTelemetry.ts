export type PaymentScenario = 'membership' | 'match_id_unlock' | 'reflux_subscription'
export type ClientPaymentEvent = 'purchase_tapped' | 'agreement_prompted' | 'session_prepare_failed' | 'order_create_requested' | 'order_create_failed' | 'pay_sheet_invoked' | 'pay_cancelled' | 'pay_failed' | 'pay_success_client' | 'confirm_timeout'
export interface PaymentEventExtra {
  error_code?: string
  error_msg?: string
  detail?: { http_status?: number }
}
export interface PaymentEvent extends PaymentEventExtra {
  event: ClientPaymentEvent
  detail: Record<string, string | number>
}
export interface PaymentEventsPayload {
  attempt_id: string
  order_no: string | null
  scenario: PaymentScenario
  product_ref: string
  events: PaymentEvent[]
}
export interface PaymentAttempt {
  readonly attemptId: string
  report(event: ClientPaymentEvent, extra?: PaymentEventExtra): void
  bindOrder(orderNo: string): void
}
