const DAY_MS = 24 * 60 * 60 * 1000
const BEIJING_OFFSET_MS = 8 * 60 * 60 * 1000

function beijingDay(timestamp: number): number {
  return Math.floor((timestamp + BEIJING_OFFSET_MS) / DAY_MS)
}

export function telegraphCalendarLabel(publishedAt: string, nowMs = Date.now()): { label: string; isToday: boolean } {
  const publishedMs = Date.parse(publishedAt)
  if (!Number.isFinite(publishedMs) || !Number.isFinite(nowMs)) return { label: '', isToday: false }

  const dayDifference = beijingDay(nowMs) - beijingDay(publishedMs)
  if (dayDifference === 0) return { label: '今天', isToday: true }
  if (dayDifference === 1) return { label: '昨天', isToday: false }

  const published = new Date(publishedMs + BEIJING_OFFSET_MS)
  const now = new Date(nowMs + BEIJING_OFFSET_MS)
  const date = `${published.getUTCMonth() + 1}月${published.getUTCDate()}日`
  return {
    label: published.getUTCFullYear() === now.getUTCFullYear() ? date : `${published.getUTCFullYear()}年${date}`,
    isToday: false,
  }
}
