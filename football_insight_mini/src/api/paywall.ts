import { request } from '../utils/request'
import type { PaywallEventsPayload } from '../types/paywall'

/** Measurement only: failures are silent and never change page behavior. */
export function reportPaywallEvents(payload: PaywallEventsPayload): Promise<void> {
  return request<void>({ url: '/activity/paywall-events', method: 'POST', auth: true, silentNetworkError: true, data: payload })
}
