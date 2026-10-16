import type { TicketWatchMatchSummary } from '../../types/ticketWatch'

export type TicketWatchLoadReason = 'initial' | 'poll'

export type TicketWatchHistoryLoadReason = 'initial' | 'selection'

export interface TicketWatchCurrentLoadStrategy {
  showBlockingLoading: boolean
  clearErrorBeforeLoad: boolean
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

export function selectRefluxStatsMatches(
  matches: TicketWatchMatchSummary[],
  primaryTeamName = '成都蓉城',
): TicketWatchMatchSummary[] {
  return matches.filter((match) =>
    match.include_in_reflux_stats !== false
    && (match.home_team_name === primaryTeamName || match.away_team_name === primaryTeamName),
  )
}

/** One displayed sample for both history tabs; unknown totals remain load candidates only. */
export function selectHistoryMatchesWithData(
  matches: TicketWatchMatchSummary[],
  totalsByMatchId: Record<number, number | undefined>,
  primaryTeamName = '成都蓉城',
): TicketWatchMatchSummary[] {
  return selectRefluxStatsMatches(matches, primaryTeamName).filter((match) => {
    const total = totalsByMatchId[match.match_id]
    return typeof total === 'number' && Number.isFinite(total) && total > 0
  })
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

export function requireInventorySince(saleStartAt?: string | null): string {
  const since = buildInventorySince(saleStartAt)
  if (!since) {
    throw new Error('sale_start_at is required to build inventory since; refusing full inventory query')
  }

  return since
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
