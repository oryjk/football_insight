import { request } from '../utils/request'

export const ACTIVITY_PAGE_KEYS = [
  'home',
  'rankings',
  'insights',
  'ticket_watch',
  'user',
  'membership_purchase',
] as const

export type ActivityPageKey = (typeof ACTIVITY_PAGE_KEYS)[number]

export function buildActivityPageViewUrl(): string {
  return '/activity/page-view'
}

export function recordPageActivity(pageKey: ActivityPageKey): Promise<void> {
  return request<void>({
    url: buildActivityPageViewUrl(),
    method: 'POST',
    auth: true,
    data: { page_key: pageKey },
  })
}
