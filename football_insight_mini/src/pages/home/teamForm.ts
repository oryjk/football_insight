import type { MatchOutcome } from '../../types/insight'

const labels: Record<MatchOutcome, string> = { win: '胜', draw: '平', loss: '负' }
const levels: Record<MatchOutcome, number> = { win: 20, draw: 50, loss: 80 }

export function buildHomeTeamForm(input: readonly unknown[] | undefined) {
  const results = (input ?? [])
    .filter((value): value is MatchOutcome => value === 'win' || value === 'draw' || value === 'loss')
    .slice(-6)
  // Keep short records on the right, with the same spacing as a full six-game record.
  const points = results.map((result, index) => ({
    result,
    top: levels[result],
    left: ((6 - results.length + index) / 5) * 100,
    latest: index === results.length - 1,
  }))
  const segments = points.slice(1).map((point, index) => {
    const previous = points[index]!
    return {
      left: previous.left,
      width: point.left - previous.left,
      top: Math.min(point.top, previous.top),
      height: Math.abs(point.top - previous.top),
      direction: point.top === previous.top ? 'flat' : point.top < previous.top ? 'up' : 'down',
      latest: point.latest,
    }
  })
  return {
    points,
    segments,
    latestLabel: results.length ? labels[results[results.length - 1]!] : '',
    description: results.length
      ? `近${results.length}场，从旧到新：${results.map(result => labels[result]).join('、')}`
      : '暂无已完赛战绩',
  }
}
