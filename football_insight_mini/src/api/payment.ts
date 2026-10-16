import type { PaymentEventsPayload } from '../types/paymentTelemetry'
import { request } from '../utils/request'
import type { DigitalPaymentParams, MembershipProduct, MembershipProductOption } from '../types/payment'

export type { MembershipProduct, MembershipProductOption }

export interface CreateOrderResponse {
  order_no: string
  params: DigitalPaymentParams
}
export function getVirtualPaymentStatus(): Promise<{ available: boolean; message: string }> {
  return request({ url: '/payment/virtual/status' })
}
export function prepareVirtualPaymentSession(code: string): Promise<{ ready: boolean }> {
  return request({ url: '/payment/virtual/session', method: 'POST', auth: true, data: { code } })
}

export interface OrderStatusResponse {
  order_no: string
  status: string
  amount_cents: number
  paid_at: string | null
}

export function getMembershipProduct(): Promise<MembershipProduct> {
  return request<MembershipProduct>({
    url: '/payment/membership-product',
    auth: true,
    headers: { 'X-Client-Capabilities': 'catalog-v2' },
  })
}

export function createMembershipOrder(targetTier: string, attemptId?: string): Promise<CreateOrderResponse> {
  return request<CreateOrderResponse>({
    url: '/payment/membership/order',
    headers: {
      'X-Client-Capabilities': 'catalog-v2',
      ...(attemptId ? { 'X-Payment-Attempt-Id': attemptId } : {}),
    },
    method: 'POST',
    auth: true,
    data: {
      target_tier: targetTier,
    },
  })
}

export function getOrderStatus(orderNo: string): Promise<OrderStatusResponse> {
  return request<OrderStatusResponse>({
    url: `/payment/order/${orderNo}`,
    auth: true,
  })
}

export function reportPaymentEvents(payload: PaymentEventsPayload): Promise<void> {
  return request<void>({ url: '/payment/events', method: 'POST', auth: true, silentNetworkError: true, data: payload })
}
