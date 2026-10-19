/** Use explicit Beijing time; avoid device timezone changing the displayed creation order. */
export function lineupDateLabel(value: string): string {
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return ''
  return new Date(date.getTime() + 8 * 60 * 60 * 1000).toISOString().slice(0, 16).replace('T', ' ')
}
