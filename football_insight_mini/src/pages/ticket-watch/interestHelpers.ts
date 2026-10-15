import type { TicketWatchBlockInterest, TicketWatchGroupedInventorySection, TicketWatchTrackedInterest } from '../../types/ticketWatch'
import { resolveTicketWatchBlockKey } from './ticketWatchSharedHelpers'

export interface TicketWatchTrackedInterestSummary {
  total: number
  hitCount: number
  pendingCount: number
}

export type TicketWatchCollapsedSectionState = Record<string, boolean>

export function applyBlockInterestsToSections(
  sections: TicketWatchGroupedInventorySection[],
  interests: TicketWatchBlockInterest[],
): TicketWatchGroupedInventorySection[] {
  const interestMap = new Map(interests.map((item) => [resolveTicketWatchBlockKey(item), item]))

  return sections.map((section) => ({
    ...section,
    items: section.items.map((item) => {
      const interest = interestMap.get(resolveTicketWatchBlockKey(item))

      return {
        ...item,
        interested_user_count: interest?.interested_user_count ?? 0,
        viewer_interested: interest?.viewer_interested ?? false,
      }
    }),
  }))
}

export function applyBlockInterestToSections(
  sections: TicketWatchGroupedInventorySection[],
  interest: TicketWatchBlockInterest,
): TicketWatchGroupedInventorySection[] {
  const interestBlockKey = resolveTicketWatchBlockKey(interest)

  return sections.map((section) => ({
    ...section,
    items: section.items.map((item) => {
      if (resolveTicketWatchBlockKey(item) !== interestBlockKey) {
        return item
      }

      return {
        ...item,
        interested_user_count: interest.interested_user_count,
        viewer_interested: interest.viewer_interested,
      }
    }),
  }))
}

export function resolveBlockInterestHeatLevel(interestedUserCount: number): number {
  if (interestedUserCount <= 0) {
    return 0
  }

  const cappedCount = Math.min(interestedUserCount, 50)
  return Math.max(1, Math.min(4, Math.ceil(cappedCount / 12.5)))
}

export function buildTrackedInterestSummary(
  trackedInterests: TicketWatchTrackedInterest[],
): TicketWatchTrackedInterestSummary {
  const total = trackedInterests.length
  const hitCount = trackedInterests.filter((item) => Boolean(item.first_inventory_at)).length

  return {
    total,
    hitCount,
    pendingCount: Math.max(0, total - hitCount),
  }
}

export function isTicketWatchSectionCollapsed(
  state: TicketWatchCollapsedSectionState,
  key: string,
): boolean {
  return Boolean(state[key])
}

export function toggleTicketWatchSectionCollapsed(
  state: TicketWatchCollapsedSectionState,
  key: string,
): TicketWatchCollapsedSectionState {
  return {
    ...state,
    [key]: !isTicketWatchSectionCollapsed(state, key),
  }
}

export function formatTrackedInterestTime(value: string): string {
  if (!value) {
    return '--'
  }

  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return '--'
  }

  // Mini Program runtimes on real devices may not expose Intl, so format
  // Beijing time by shifting to UTC+8 and reading via UTC getters.
  const chinaTime = new Date(date.getTime() + 8 * 60 * 60 * 1000)
  const month = String(chinaTime.getUTCMonth() + 1).padStart(2, '0')
  const day = String(chinaTime.getUTCDate()).padStart(2, '0')
  const hours = String(chinaTime.getUTCHours()).padStart(2, '0')
  const minutes = String(chinaTime.getUTCMinutes()).padStart(2, '0')
  return `${month}-${day} ${hours}:${minutes}`
}

export function formatTrackedInterestWaitLabel(
  startedAt: string,
  firstInventoryAt?: string | null,
): string {
  if (!firstInventoryAt) {
    return '暂未等到回流'
  }

  const startedAtTime = new Date(startedAt).getTime()
  const firstInventoryTime = new Date(firstInventoryAt).getTime()

  if (!Number.isFinite(startedAtTime) || !Number.isFinite(firstInventoryTime)) {
    return '已等到回流'
  }

  const diffMinutes = Math.round((firstInventoryTime - startedAtTime) / 60000)

  if (diffMinutes <= 0) {
    return '等了不到 1 分钟'
  }

  return `等了 ${diffMinutes} 分钟`
}
