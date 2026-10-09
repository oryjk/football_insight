import type {
  TicketWatchGroupedInventoryItem,
  TicketWatchMatchSummary,
} from '../../types/ticketWatch'
import {
  formatTrackedInterestTime,
  resolveRecentRefluxBucketRequiredTier,
  resolveInventoryPriceTone,
  type TicketWatchBoardStats,
  type TicketWatchHistoryRefluxTrendPoint,
  type TicketWatchRecentRefluxBucketKey,
} from './helpers'

export function resolveFallbackMatchId(
  match: Pick<TicketWatchMatchSummary, 'match_id' | 'external_match_id'> | null,
): number | null {
  if (!match?.external_match_id) {
    return null
  }

  const parsed = Number.parseInt(match.external_match_id, 10)

  if (!Number.isFinite(parsed) || parsed <= 0 || parsed === match.match_id) {
    return null
  }

  return parsed
}

export function sumInventoryOccurrences(
  inventory: Array<{ occurrences: number }>,
): number {
  return inventory.reduce((sum, item) => sum + item.occurrences, 0)
}

export function resolveInventoryBlockKey(
  item: Pick<TicketWatchGroupedInventoryItem, 'block_key' | 'block_name'>,
): string {
  return item.block_key?.trim() || item.block_name
}

export function formatLatestTime(value: string): string {
  return formatTrackedInterestTime(value)
}

export function formatRecentRefluxMinuteLabel(minutesAgo: number): string {
  if (minutesAgo <= 0) {
    return '刚刚'
  }

  return `${minutesAgo}分钟前`
}

export function formatRecentRefluxBucketRequiredTier(
  bucketKey: TicketWatchRecentRefluxBucketKey,
): string {
  return resolveRecentRefluxBucketRequiredTier(bucketKey)
}

export function formatHistoryTrendBarWidth(
  point: TicketWatchHistoryRefluxTrendPoint,
): string {
  if (!point.is_loaded || point.total_occurrences <= 0) {
    return '0%'
  }

  return `${Math.max(point.bar_percent, 8)}%`
}

export function resolvePriceToneClass(price: string): string {
  return `price-tone--${resolveInventoryPriceTone(price)}`
}

export function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`
}

export function formatPrice(price: string): string {
  return `¥${price}`
}

export function formatCoverage(stats: TicketWatchBoardStats): string {
  return `${stats.activeRegionCount}/${stats.totalRegionCount}`
}

export function formatHotPrice(stats: TicketWatchBoardStats): string {
  return stats.hottestPrice ? formatPrice(stats.hottestPrice.price) : '--'
}

export function formatHotPriceMeta(stats: TicketWatchBoardStats): string {
  if (!stats.hottestPrice) {
    return '暂无热点'
  }

  return `${stats.hottestPrice.available_region_count} 区 / ${stats.hottestPrice.total_occurrences} 张`
}

export function buildBoardInsightSummary(
  stats: TicketWatchBoardStats,
  mode: 'current' | 'history',
): string {
  if (!stats.totalRegionCount) {
    return mode === 'current'
      ? '当前还没有足够的分区回流信号，先等下一波回流。'
      : '这场历史比赛暂时没有足够样本，先不要把它当成固定规律。'
  }

  const hottestPriceLabel = stats.hottestPrice
    ? formatPrice(stats.hottestPrice.price)
    : '暂无热点价位'
  const modeLabel = mode === 'current' ? '当前供给面' : '这场比赛当时的供给面'

  return (
    `${modeLabel}里，${stats.activeRegionCount}/${stats.totalRegionCount} 个区域出现过回流，覆盖率 ${formatPercent(stats.activeRegionRatio)}。${hottestPriceLabel} 是最活跃的价格带，累计回流 ${stats.totalOccurrences} 张。` +
    ' 统计口径只看开售 10 分钟后的回流。'
  )
}

export function buildBoardDecisionLines(
  stats: TicketWatchBoardStats,
  mode: 'current' | 'history',
): string[] {
  if (!stats.totalRegionCount) {
    return [
      mode === 'current'
        ? '先观察，不要盲刷，当前还没有足够的分区回流信号。'
        : '这场比赛样本不足，先别把它当成下次抢票的稳定参考。',
    ]
  }

  const lines: string[] = []

  if (stats.hottestPrice) {
    lines.push(
      `${mode === 'current' ? '先盯' : '复盘先看'} ${formatPrice(stats.hottestPrice.price)}：${stats.hottestPrice.available_region_count} 区有回流，共 ${stats.hottestPrice.total_occurrences} 张回流。`,
    )
  }

  if (stats.topBlocks.length) {
    lines.push(
      `${mode === 'current' ? '重点刷新' : '历史高频区'} ${stats.topBlocks.map((item) => item.block_name).join('、')}：这些区域的回流最密。`,
    )
  }

  if (stats.cheapestActivePrice) {
    if (
      stats.hottestPrice &&
      stats.cheapestActivePrice.price === stats.hottestPrice.price
    ) {
      lines.push(
        `${formatPrice(stats.cheapestActivePrice.price)} 同时也是当前最低仍有回流的价位，预算优先就从这里进。`,
      )
    } else {
      lines.push(
        `${mode === 'current' ? '想压预算' : '想找低价经验'}，先看 ${formatPrice(stats.cheapestActivePrice.price)}：这是最低仍有回流的价位。`,
      )
    }
  }

  return lines.slice(0, 3)
}
