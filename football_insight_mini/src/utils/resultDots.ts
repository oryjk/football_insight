import type { MatchOutcome } from '../types/insight'

export type ResultDotTone = MatchOutcome

export interface ResultDotItem {
  id: string | number
  tone: ResultDotTone
  ariaLabel?: string
}

export const RESULT_DOT_LABELS: Record<ResultDotTone, string> = { win: '胜', draw: '平', loss: '负' }

/** Keep valid results (the newest `limit` when given), oldest to newest, without inventing matches. */
export function buildOutcomeDots(input: readonly unknown[] | undefined, limit?: number): ResultDotItem[] {
  const outcomes = (input ?? []).filter(
    (value): value is MatchOutcome => value === 'win' || value === 'draw' || value === 'loss',
  )
  return (limit === undefined ? outcomes : outcomes.slice(-limit)).map((tone, index) => ({ id: index, tone }))
}
