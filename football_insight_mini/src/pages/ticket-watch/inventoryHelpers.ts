import type { TicketWatchGroupedInventorySection, TicketWatchInventoryEntry, TicketWatchRegion } from '../../types/ticketWatch'
import { resolveTicketWatchBlockKey } from './ticketWatchSharedHelpers'

export interface TicketWatchBoardTopBlock {
  block_key: string
  block_name: string
  price: string
  occurrences: number
  latest_time: string
}

export interface TicketWatchBoardTopInterestBlock {
  block_key: string
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

export function groupInventoryByPrice(
  regions: TicketWatchRegion[],
  inventory: TicketWatchInventoryEntry[],
): TicketWatchGroupedInventorySection[] {
  const inventoryMap = new Map<string, TicketWatchInventoryEntry>()
  inventory.forEach((item) => inventoryMap.set(resolveTicketWatchBlockKey(item), item))

  const grouped = new Map<string, TicketWatchGroupedInventorySection>()

  regions.forEach((region) => {
    const blockKey = resolveTicketWatchBlockKey(region)
    const matched = inventoryMap.get(blockKey)
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
      block_key: blockKey,
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
      items: section.items.sort((left, right) => (
        left.block_name.localeCompare(right.block_name, 'zh-CN') ||
        resolveTicketWatchBlockKey(left).localeCompare(resolveTicketWatchBlockKey(right))
      )),
    }))
    .sort((left, right) => Number(left.price) - Number(right.price))
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
          block_key: resolveTicketWatchBlockKey(item),
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
          block_key: resolveTicketWatchBlockKey(item),
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
