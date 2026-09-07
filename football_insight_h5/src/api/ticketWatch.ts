import { request } from '../lib/request'
import type { TicketWatchRegion } from '../types/ticketWatch'

export function getTicketWatchRegions(): Promise<TicketWatchRegion[]> {
  return request<TicketWatchRegion[]>({
    url: '/ticket-watch/regions',
  })
}
