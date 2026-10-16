import type { TicketWatchBestWindow, TicketWatchBestWindowsResponse } from '../../types/ticketWatch'

/** Keep membership suspension and review gates independent of the server's tier response. */
export function resolveBestWindowsData(
  data: TicketWatchBestWindowsResponse | null,
  benefitsLocked: boolean,
  underReview: boolean,
): TicketWatchBestWindowsResponse | null {
  if (underReview || data?.status !== 'ok' || !data.top_windows.length) return null
  if (benefitsLocked || data.access === 'preview') {
    // Even if the server sent a full response, a suspended member must not see the hour.
    const top = data.top_windows.slice(0, 1).map(row => ({ ...row, start_hour: null, end_hour: null }))
    return { ...data, access: 'preview', top_windows: top, hours: [] }
  }
  return data
}

export function buildBestWindowsBars(data: TicketWatchBestWindowsResponse, currentHour: number) {
  if (data.access !== 'full') return []
  const max = Math.max(0, ...data.hours.map(row => row.avg_events))
  const peaks = new Set(data.top_windows.slice(0, 3).map(row => row.start_hour))
  return data.hours.map(row => ({
    ...row,
    heightPercent: max > 0 && row.avg_events > 0 ? Math.max(4, row.avg_events / max * 100) : 0,
    isPeak: peaks.has(row.hour),
    isNow: row.hour === currentHour,
    label: [0, 6, 12, 18].includes(row.hour) ? String(row.hour) : '',
  }))
}

export const MASKED_BEST_WINDOW = '**:**–**:**'

export function formatBestWindow(window: TicketWatchBestWindow): string {
  if (window.start_hour === null || window.end_hour === null) return MASKED_BEST_WINDOW
  return `${String(window.start_hour).padStart(2, '0')}:00–${String(window.end_hour).padStart(2, '0')}:00`
}

/** Map a horizontal touch position over the chart to its hour column (0-23). */
export function hourFromChartX(x: number, left: number, width: number, hours = 24): number | null {
  if (!(width > 0) || !Number.isFinite(x) || !Number.isFinite(left)) return null
  const index = Math.floor((x - left) / width * hours)
  return Math.min(hours - 1, Math.max(0, index))
}

export function formatBestWindowsConclusion(windows: TicketWatchBestWindow[]): string {
  return `补票最集中在 ${windows.slice(0, 3).map(formatBestWindow).join('、')}`
}

export function formatShare(share: number): string {
  return `${Number((share * 100).toFixed(1))}%`
}

export function bestWindowsHourDetail(hour: number, sampleSize: number, hitMatches: number, share?: number): string {
  const base = `${hour} 点：近 ${sampleSize} 场里有 ${hitMatches} 场出现补票`
  return typeof share === 'number' && Number.isFinite(share) ? `${base}，占全天补票的 ${formatShare(share)}` : base
}
