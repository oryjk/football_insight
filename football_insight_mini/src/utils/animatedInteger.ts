export function buildIntegerSequence(start: number, target: number): number[] {
  const from = Math.round(start)
  const to = Math.round(target)

  if (from === to) {
    return [to]
  }

  const step = from < to ? 1 : -1
  const values: number[] = []

  for (let current = from; current !== to; current += step) {
    values.push(current)
  }

  values.push(to)

  if (Math.abs(to - from) >= 2) {
    values.push(to - step)
    values.push(to)
  }

  return values
}
