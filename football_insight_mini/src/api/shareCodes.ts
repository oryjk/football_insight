import { request } from '../utils/request'
import type { ShareCodeImage } from '../types/shareCode'

export function getTicketWatchShareCode(): Promise<ShareCodeImage> {
  return request({ url: '/share-codes/ticket-watch', method: 'POST', auth: true })
}
