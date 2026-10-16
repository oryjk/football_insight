import type { DigitalPaymentParams } from '../types/payment'
import { request } from '../utils/request'

export interface MatchIdEntitlement {
  unlocked: boolean
  via: 'membership' | 'purchase' | null
  effective_tier: string
}

export interface CreateMatchIdOrderResponse {
  order_no: string
  wx_pay_params: DigitalPaymentParams
}

export function getMatchIdEntitlement(matchId: number): Promise<MatchIdEntitlement> {
  return request<MatchIdEntitlement>({
    url: `/match-id/entitlement?match_id=${matchId}`,
    auth: true,
  })
}

export function createMatchIdOrder(matchId: number, attemptId?: string): Promise<CreateMatchIdOrderResponse> {
  return request<CreateMatchIdOrderResponse>({
    url: '/match-id/order',
    ...(attemptId ? { headers: { 'X-Payment-Attempt-Id': attemptId } } : {}),
    method: 'POST',
    auth: true,
    data: {
      match_id: matchId,
    },
  })
}
