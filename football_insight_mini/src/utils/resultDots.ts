import type { MatchOutcome } from '../types/insight'

export type ResultDotTone = MatchOutcome | 'upcoming'

export interface ResultDotItem {
  id: string | number
  tone: ResultDotTone
  ariaLabel?: string
}

export const RESULT_DOT_LABELS: Record<ResultDotTone, string> = { win: '胜', draw: '平', loss: '负', upcoming: '未赛' }

/** 中超单赛季每队 30 轮，用于把未赛轮次补成灰色圆点。 */
export const CSL_SEASON_TOTAL_ROUNDS = 30

/** Keep valid results (the newest `limit` when given), oldest to newest, without inventing matches. */
export function buildOutcomeDots(input: readonly unknown[] | undefined, limit?: number): ResultDotItem[] {
  const outcomes = (input ?? []).filter(
    (value): value is MatchOutcome => value === 'win' || value === 'draw' || value === 'loss',
  )
  return (limit === undefined ? outcomes : outcomes.slice(-limit)).map((tone, index) => ({ id: index, tone }))
}

/** Played results followed by upcoming placeholders, so a full season always renders `total` dots. */
export function buildSeasonDots(input: readonly unknown[] | undefined, total = CSL_SEASON_TOTAL_ROUNDS): ResultDotItem[] {
  return padUpcomingDots(buildOutcomeDots(input, total), total)
}

/** Append upcoming placeholders to already-built dots so the row always shows `total` entries. */
export function padUpcomingDots(dots: ResultDotItem[], total = CSL_SEASON_TOTAL_ROUNDS): ResultDotItem[] {
  const upcomingCount = Math.max(0, total - dots.length)
  return [
    ...dots,
    ...Array.from({ length: upcomingCount }, (_, index) => ({ id: `upcoming-${index}`, tone: 'upcoming' as const })),
  ]
}
