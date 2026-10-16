import type { TicketWatchBestWindow, TicketWatchBestWindowsResponse } from '../../types/ticketWatch'

/** Keep membership suspension and review gates independent of the server's tier response. */
export function resolveBestWindowsData(
  data: TicketWatchBestWindowsResponse | null,
  benefitsLocked: boolean,
  underReview: boolean,
): TicketWatchBestWindowsResponse | null {
  if (underReview || data?.status !== 'ok' || !data.top_windows.length) return null
  if (benefitsLocked || data.access === 'preview') {
    return { ...data, access: 'preview', top_windows: data.top_windows.slice(0, 1), hours: [] }
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

export function formatBestWindow(window: TicketWatchBestWindow): string {
  return `${String(window.start_hour).padStart(2, '0')}:00–${String(window.end_hour).padStart(2, '0')}:00`
}

export function formatBestWindowsConclusion(windows: TicketWatchBestWindow[]): string {
  return `补票最集中在 ${windows.slice(0, 3).map(formatBestWindow).join('、')}`
}

export function bestWindowsHourDetail(hour: number, sampleSize: number, hitMatches: number): string {
  return `${hour} 点：近 ${sampleSize} 场里有 ${hitMatches} 场出现补票`
}
