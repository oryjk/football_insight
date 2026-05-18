import type {
  CancelMatchedSeatSwapPayload,
  SeatSwapCurrentResponse,
  UpsertSeatSwapRequestPayload,
} from '../types/seatSwap'
import { request } from '../utils/request'

export function getCurrentSeatSwap(): Promise<SeatSwapCurrentResponse> {
  return request<SeatSwapCurrentResponse>({
    url: '/seat-swap/current',
    auth: true,
  })
}

export function upsertMySeatSwapRequest(payload: UpsertSeatSwapRequestPayload): Promise<void> {
  return request<void>({
    url: '/seat-swap/my-request',
    method: 'PUT',
    auth: true,
    data: payload,
  })
}

export function deleteMySeatSwapRequest(): Promise<void> {
  return request<void>({
    url: '/seat-swap/my-request',
    method: 'DELETE',
    auth: true,
  })
}

export function confirmSeatSwapCandidate(targetRequestId: string): Promise<void> {
  return request<void>({
    url: `/seat-swap/matches/${targetRequestId}/confirm`,
    method: 'POST',
    auth: true,
  })
}

export function cancelMatchedSeatSwap(
  targetRequestId: string,
  payload: CancelMatchedSeatSwapPayload,
): Promise<void> {
  return request<void>({
    url: `/seat-swap/matches/${targetRequestId}/cancel`,
    method: 'POST',
    auth: true,
    data: payload,
  })
}
