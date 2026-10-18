import type { TicketWatchGroupedInventorySection, TicketWatchMatchSummary } from '../../types/ticketWatch'
import { buildInventorySince } from './ticketWatchLoadHelpers'

export const TICKET_POSTER = { width: 750, height: 1080, canvasId: 'ticketWatchSharePoster' } as const
export interface TicketPosterRegion { name: string; price: string; occurrences: number }
export interface TicketPosterContent {
  mode: 'current' | 'history'
  teams: string
  matchLabel: string
  totalOccurrences: number
  regionCount: number
  regions: TicketPosterRegion[]
  cutoffLabel: string | null
  disclaimer: string
}
export interface TicketPosterInput {
  mode: 'current' | 'history'
  match: TicketWatchMatchSummary | null
  /** Already queried with requireInventorySince(sale_start_at), as in the displayed board. */
  sections: TicketWatchGroupedInventorySection[]
  loadedAtMs?: number | null
  viewer?: { membership_tier?: string | null } | null
}

/**
 * Only the basic inventory grid is public for V1/guests. It has no tier condition in
 * TicketInventoryPanel. Recent-reflux buckets (V6–V8), best windows, interest data,
 * timestamps per block and account fields are deliberately absent from this model.
 * Never pass a viewer's recent-reflux or best-window view model to the drawing code.
 */
export function buildTicketPosterContent(input: TicketPosterInput): TicketPosterContent | null {
  const { match, sections, mode } = input
  const since = buildInventorySince(match?.sale_start_at)
  if (!match || !since) return null
  if (mode === 'current' && (!Number.isFinite(input.loadedAtMs) || !input.loadedAtMs)) return null
  const regions = sections.flatMap(section => section.items)
    .filter(item => item.has_inventory && Number.isFinite(item.occurrences) && item.occurrences > 0
      && new Date(item.latest_time).getTime() >= new Date(since).getTime())
    .map(item => ({ name: item.block_name, price: item.price, occurrences: Math.floor(item.occurrences) }))
    .sort((a, b) => b.occurrences - a.occurrences || a.name.localeCompare(b.name))
  if (!regions.length) return null
  return {
    mode,
    teams: `${match.home_team_name} VS ${match.away_team_name}`,
    matchLabel: `${match.match_date} ${match.match_time} · 第 ${match.round_number} 轮`,
    totalOccurrences: regions.reduce((sum, region) => sum + region.occurrences, 0),
    regionCount: regions.length,
    regions: regions.slice(0, 5),
    cutoffLabel: mode === 'current' ? formatTicketPosterCutoff(input.loadedAtMs!) : null,
    disclaimer: mode === 'current' ? '实时库存变化很快，请以小程序为准' : '历史回流记录，不代表当前仍有余票',
  }
}

/** Explicit Beijing time, independent of the device's timezone. */
export function formatTicketPosterCutoff(timestampMs: number): string {
  return new Date(timestampMs + 8 * 60 * 60 * 1000).toISOString().slice(0, 19).replace('T', ' ')
}
