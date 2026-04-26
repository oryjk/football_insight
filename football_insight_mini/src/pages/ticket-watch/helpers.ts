import type {
  TicketWatchBlockInterest,
  TicketWatchGroupedInventorySection,
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
  TicketWatchRegion,
  TicketWatchTrackedInterest,
} from '../../types/ticketWatch'
export {
  formatTicketWatchPollIntervalLabel,
  isMembershipTierAtLeast,
  normalizeTicketWatchPollIntervalSeconds,
} from '../../utils/membershipRules'
import { isMembershipTierAtLeast } from '../../utils/membershipRules'

export type TicketWatchLoadReason = 'initial' | 'poll'
export type TicketWatchHistoryLoadReason = 'initial' | 'selection'

export interface TicketWatchCurrentLoadStrategy {
  showBlockingLoading: boolean
  clearErrorBeforeLoad: boolean
}

export interface TicketWatchBoardTopBlock {
  block_name: string
  price: string
  occurrences: number
  latest_time: string
}

export interface TicketWatchBoardTopInterestBlock {
  block_name: string
  price: string
  interested_user_count: number
}

export interface TicketWatchBoardStats {
  priceBandCount: number
  totalRegionCount: number
  activeRegionCount: number
  activeRegionRatio: number
  totalOccurrences: number
  hottestPrice: TicketWatchGroupedInventorySection | null
  cheapestActivePrice: TicketWatchGroupedInventorySection | null
  topBlocks: TicketWatchBoardTopBlock[]
  topInterestBlocks: TicketWatchBoardTopInterestBlock[]
}

export interface TicketWatchTrackedInterestSummary {
  total: number
  hitCount: number
  pendingCount: number
}

export type TicketWatchCollapsedSectionState = Record<string, boolean>
export type TicketWatchInventoryRefluxFreshness = 'strong' | 'weak' | 'none'
export type TicketWatchRecentRefluxBucketKey = 'within3' | 'within10' | 'within30'
export type TicketWatchRecentRefluxPanelMode = 'hidden' | 'locked' | 'unlocked'

export interface TicketWatchRecentRefluxItem {
  block_name: string
  price: string
  occurrences: number
  latest_time: string
  minutes_ago: number
}

export interface TicketWatchRecentRefluxBucket {
  key: TicketWatchRecentRefluxBucketKey
  title: string
  subtitle: string
  items: TicketWatchRecentRefluxItem[]
}

export function formatTicketWatchMembershipBadgeTier(membershipTier?: string | null): string {
  const normalizedTier = membershipTier?.trim().toUpperCase() || 'V1'
  return `${normalizedTier} 会员`
}

export function resolveCurrentBoardLoadStrategy(
  hasLoadedOnce: boolean,
  reason: TicketWatchLoadReason,
): TicketWatchCurrentLoadStrategy {
  if (reason === 'poll') {
    return {
      showBlockingLoading: false,
      clearErrorBeforeLoad: false,
    }
  }

  return {
    showBlockingLoading: !hasLoadedOnce,
    clearErrorBeforeLoad: true,
  }
}

export function resolveHistoryBoardLoadStrategy(
  hasLoadedOnce: boolean,
  reason: TicketWatchHistoryLoadReason,
): TicketWatchCurrentLoadStrategy {
  if (reason === 'selection') {
    return {
      showBlockingLoading: false,
      clearErrorBeforeLoad: false,
    }
  }

  return {
    showBlockingLoading: !hasLoadedOnce,
    clearErrorBeforeLoad: true,
  }
}

export function selectCompletedMatches(
  matches: TicketWatchMatchSummary[],
  nowIso = new Date().toISOString(),
): TicketWatchMatchSummary[] {
  const now = new Date(nowIso).getTime()

  return matches
    .filter((match) => new Date(match.kickoff_at).getTime() < now)
    .sort((left, right) => new Date(right.kickoff_at).getTime() - new Date(left.kickoff_at).getTime())
}

export function groupInventoryByPrice(
  regions: TicketWatchRegion[],
  inventory: TicketWatchInventoryEntry[],
): TicketWatchGroupedInventorySection[] {
  const inventoryMap = new Map<string, TicketWatchInventoryEntry>()
  inventory.forEach((item) => inventoryMap.set(item.block_name, item))

  const grouped = new Map<string, TicketWatchGroupedInventorySection>()

  regions.forEach((region) => {
    const matched = inventoryMap.get(region.block_name)
    const section = grouped.get(region.price) ?? {
      price: region.price,
      region_count: 0,
      available_region_count: 0,
      total_occurrences: 0,
      items: [],
    }

    section.region_count += 1
    if (matched) {
      section.available_region_count += 1
      section.total_occurrences += matched.occurrences
    }

    section.items.push({
      block_name: region.block_name,
      price: region.price,
      occurrences: matched?.occurrences ?? 0,
      latest_time: matched?.latest_time ?? '',
      has_inventory: Boolean(matched && matched.occurrences > 0),
      interested_user_count: 0,
      viewer_interested: false,
    })

    grouped.set(region.price, section)
  })

  return Array.from(grouped.values())
    .map((section) => ({
      ...section,
      items: section.items.sort((left, right) => left.block_name.localeCompare(right.block_name, 'zh-CN')),
    }))
    .sort((left, right) => Number(left.price) - Number(right.price))
}

export function applyBlockInterestsToSections(
  sections: TicketWatchGroupedInventorySection[],
  interests: TicketWatchBlockInterest[],
): TicketWatchGroupedInventorySection[] {
  const interestMap = new Map(interests.map((item) => [item.block_name, item]))

  return sections.map((section) => ({
    ...section,
    items: section.items.map((item) => {
      const interest = interestMap.get(item.block_name)

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
  return sections.map((section) => ({
    ...section,
    items: section.items.map((item) => {
      if (item.block_name !== interest.block_name) {
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

export function summarizeInventoryBoard(
  sections: TicketWatchGroupedInventorySection[],
): TicketWatchBoardStats {
  const totalRegionCount = sections.reduce((sum, section) => sum + section.region_count, 0)
  const activeRegionCount = sections.reduce((sum, section) => sum + section.available_region_count, 0)
  const totalOccurrences = sections.reduce((sum, section) => sum + section.total_occurrences, 0)
  const activeSections = sections.filter((section) => section.available_region_count > 0)
  const activeNonVipSections = activeSections.filter((section) => !isVipInventorySection(section))

  const topBlocks = sections
    .flatMap((section) =>
      section.items
        .filter((item) => item.has_inventory && !isVipInventoryBlock(item.block_name))
        .map((item) => ({
          block_name: item.block_name,
          price: item.price,
          occurrences: item.occurrences,
          latest_time: item.latest_time,
        })),
    )
    .sort((left, right) => {
      if (right.occurrences !== left.occurrences) {
        return right.occurrences - left.occurrences
      }

      return right.latest_time.localeCompare(left.latest_time)
    })
    .slice(0, 3)

  const topInterestBlocks = sections
    .flatMap((section) =>
      section.items
        .filter((item) => item.interested_user_count > 0 && !isVipInventoryBlock(item.block_name))
        .map((item) => ({
          block_name: item.block_name,
          price: item.price,
          interested_user_count: item.interested_user_count,
        })),
    )
    .sort((left, right) => {
      if (right.interested_user_count !== left.interested_user_count) {
        return right.interested_user_count - left.interested_user_count
      }

      return Number(left.price) - Number(right.price)
    })
    .slice(0, 3)

  const hottestPriceSource = activeNonVipSections.length > 0 ? activeNonVipSections : activeSections

  const hottestPrice = hottestPriceSource.reduce<TicketWatchGroupedInventorySection | null>((selected, section) => {
    if (!selected) {
      return section
    }

    if (section.total_occurrences !== selected.total_occurrences) {
      return section.total_occurrences > selected.total_occurrences ? section : selected
    }

    return Number(section.price) < Number(selected.price) ? section : selected
  }, null)

  const cheapestActivePrice = activeSections.reduce<TicketWatchGroupedInventorySection | null>((selected, section) => {
    if (!selected) {
      return section
    }

    return Number(section.price) < Number(selected.price) ? section : selected
  }, null)

  return {
    priceBandCount: sections.length,
    totalRegionCount,
    activeRegionCount,
    activeRegionRatio: totalRegionCount > 0 ? activeRegionCount / totalRegionCount : 0,
    totalOccurrences,
    hottestPrice,
    cheapestActivePrice,
    topBlocks,
    topInterestBlocks,
  }
}

export function buildInventorySince(saleStartAt?: string | null): string | null {
  if (!saleStartAt) {
    return null
  }

  const offsetMinutes = extractOffsetMinutes(saleStartAt)
  const date = new Date(saleStartAt)
  if (Number.isNaN(date.getTime())) {
    return null
  }

  date.setMinutes(date.getMinutes() + 10)
  if (offsetMinutes === null) {
    return date.toISOString()
  }

  return formatDateWithOffset(date, offsetMinutes)
}

export function resolveInventoryHeatLevel(occurrences: number, maxOccurrences: number): number {
  if (occurrences <= 0 || maxOccurrences <= 0) {
    return 0
  }

  const ratio = occurrences / maxOccurrences

  if (ratio >= 0.75) {
    return 4
  }

  if (ratio >= 0.45) {
    return 3
  }

  if (ratio >= 0.2) {
    return 2
  }

  return 1
}

export function resolveInventoryRefluxFreshness(
  latestTime: string,
  nowIso = new Date().toISOString(),
): TicketWatchInventoryRefluxFreshness {
  if (!latestTime) {
    return 'none'
  }

  const latestTimestamp = new Date(latestTime).getTime()
  const nowTimestamp = new Date(nowIso).getTime()

  if (!Number.isFinite(latestTimestamp) || !Number.isFinite(nowTimestamp)) {
    return 'none'
  }

  const diffMinutes = (nowTimestamp - latestTimestamp) / 60000

  if (diffMinutes < 0) {
    return 'none'
  }

  if (diffMinutes <= 3) {
    return 'strong'
  }

  if (diffMinutes <= 10) {
    return 'weak'
  }

  return 'none'
}

export function buildRecentRefluxBuckets(
  sections: TicketWatchGroupedInventorySection[],
  nowIso = new Date().toISOString(),
): TicketWatchRecentRefluxBucket[] {
  const nowTimestamp = new Date(nowIso).getTime()
  const buckets: TicketWatchRecentRefluxBucket[] = [
    {
      key: 'within3',
      title: '3 分钟内',
      subtitle: '刚出现',
      items: [],
    },
    {
      key: 'within10',
      title: '10 分钟内',
      subtitle: '仍很近',
      items: [],
    },
    {
      key: 'within30',
      title: '30 分钟内',
      subtitle: '可回看',
      items: [],
    },
  ]

  if (!Number.isFinite(nowTimestamp)) {
    return buckets
  }

  sections
    .flatMap((section) => section.items)
    .filter((item) => item.has_inventory && item.latest_time)
    .forEach((item) => {
      const latestTimestamp = new Date(item.latest_time).getTime()

      if (!Number.isFinite(latestTimestamp)) {
        return
      }

      const diffMinutes = (nowTimestamp - latestTimestamp) / 60000

      if (diffMinutes < 0 || diffMinutes > 30) {
        return
      }

      const bucket = diffMinutes <= 3
        ? buckets[0]
        : diffMinutes <= 10
          ? buckets[1]
          : buckets[2]

      bucket.items.push({
        block_name: item.block_name,
        price: item.price,
        occurrences: item.occurrences,
        latest_time: item.latest_time,
        minutes_ago: Math.floor(diffMinutes),
      })
    })

  return buckets.map((bucket) => ({
    ...bucket,
    items: bucket.items.sort((left, right) => {
      const latestDiff = new Date(right.latest_time).getTime() - new Date(left.latest_time).getTime()

      if (latestDiff !== 0) {
        return latestDiff
      }

      return right.occurrences - left.occurrences
    }),
  }))
}

export function resolveRecentRefluxPanelMode(
  hasRecentReflux: boolean,
  membershipTier: string | null | undefined,
): TicketWatchRecentRefluxPanelMode {
  void hasRecentReflux

  if (isMembershipTierAtLeast(membershipTier, 'V6')) {
    return 'unlocked'
  }

  return 'locked'
}

export function resolveRecentRefluxBucketRequiredTier(
  bucketKey: TicketWatchRecentRefluxBucketKey,
): 'V6' | 'V7' | 'V8' {
  if (bucketKey === 'within3') {
    return 'V8'
  }

  if (bucketKey === 'within10') {
    return 'V7'
  }

  return 'V6'
}

export function isRecentRefluxBucketUnlocked(
  bucketKey: TicketWatchRecentRefluxBucketKey,
  membershipTier: string | null | undefined,
): boolean {
  return isMembershipTierAtLeast(membershipTier, resolveRecentRefluxBucketRequiredTier(bucketKey))
}

export function isVipInventoryBlock(blockName: string): boolean {
  return blockName.trim().toUpperCase().startsWith('VIP')
}

export function getInventoryEmptyStateLabel(blockName: string): string {
  return isVipInventoryBlock(blockName) ? 'VIP 暂未统计' : '无回流'
}

export function formatInventorySectionSummary(section: TicketWatchGroupedInventorySection): string {
  const isVipSection = section.items.length > 0 && section.items.every((item) => isVipInventoryBlock(item.block_name))

  if (isVipSection && section.available_region_count === 0 && section.total_occurrences === 0) {
    return 'VIP 暂未统计'
  }

  return `${section.available_region_count} 区有回流 · 共 ${section.total_occurrences} 张`
}

export function isVipInventorySection(section: TicketWatchGroupedInventorySection): boolean {
  if (section.items.length > 0) {
    return section.items.every((item) => isVipInventoryBlock(item.block_name))
  }

  return Number(section.price) >= 1000
}

export type TicketPriceTone = 'vip' | 's' | 'a' | 'b' | 'c' | 'd' | 'e' | 'neutral'

export function resolveInventoryPriceTone(price: string): TicketPriceTone {
  const numericPrice = Number(price)

  if (!Number.isFinite(numericPrice)) {
    return 'neutral'
  }

  if (numericPrice >= 1000) {
    return 'vip'
  }

  if (numericPrice >= 400) {
    return 's'
  }

  if (numericPrice >= 220) {
    return 'a'
  }

  if (numericPrice >= 180) {
    return 'b'
  }

  if (numericPrice >= 150) {
    return 'c'
  }

  if (numericPrice >= 120) {
    return 'd'
  }

  if (numericPrice >= 100) {
    return 'e'
  }

  return 'neutral'
}

export function prioritizeInventorySections(
  sections: TicketWatchGroupedInventorySection[],
): TicketWatchGroupedInventorySection[] {
  return [...sections].sort((left, right) => {
    const leftActive = left.available_region_count > 0 ? 1 : 0
    const rightActive = right.available_region_count > 0 ? 1 : 0

    if (leftActive !== rightActive) {
      return rightActive - leftActive
    }

    if (left.total_occurrences !== right.total_occurrences) {
      return right.total_occurrences - left.total_occurrences
    }

    if (left.available_region_count !== right.available_region_count) {
      return right.available_region_count - left.available_region_count
    }

    return Number(left.price) - Number(right.price)
  })
}

function extractOffsetMinutes(value: string): number | null {
  if (value.endsWith('Z')) {
    return 0
  }

  const matched = value.match(/([+-])(\d{2}):(\d{2})$/)
  if (!matched) {
    return null
  }

  const sign = matched[1] === '-' ? -1 : 1
  const hours = Number(matched[2])
  const minutes = Number(matched[3])
  return sign * (hours * 60 + minutes)
}

function formatDateWithOffset(date: Date, offsetMinutes: number): string {
  const shifted = new Date(date.getTime() + offsetMinutes * 60 * 1000)
  const sign = offsetMinutes < 0 ? '-' : '+'
  const absoluteMinutes = Math.abs(offsetMinutes)
  const offsetHours = `${Math.floor(absoluteMinutes / 60)}`.padStart(2, '0')
  const offsetRemainder = `${absoluteMinutes % 60}`.padStart(2, '0')

  const year = shifted.getUTCFullYear()
  const month = `${shifted.getUTCMonth() + 1}`.padStart(2, '0')
  const day = `${shifted.getUTCDate()}`.padStart(2, '0')
  const hours = `${shifted.getUTCHours()}`.padStart(2, '0')
  const minutes = `${shifted.getUTCMinutes()}`.padStart(2, '0')
  const seconds = `${shifted.getUTCSeconds()}`.padStart(2, '0')

  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}${sign}${offsetHours}:${offsetRemainder}`
}
