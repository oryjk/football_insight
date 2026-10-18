import type { OutlookRecord } from '../../types/matchOutlook'
export function percentages(probabilities: number[]): number[] {
  const sum = probabilities.reduce((a, b) => a + b, 0)
  if (
    probabilities.length !== 3 ||
    !(sum > 0) ||
    probabilities.some((v) => !Number.isFinite(v) || v < 0)
  )
    throw Error('概率资料暂不可用')
  const raw = probabilities.map((value) => (value / sum) * 100),
    result = raw.map(Math.floor)
  const order = raw
    .map((value, index) => ({ index, remainder: value - result[index]! }))
    .sort((a, b) => b.remainder - a.remainder || a.index - b.index)
  const missing = 100 - result.reduce((a, b) => a + b, 0)
  for (let i = 0; i < missing; i++) result[order[i]!.index]!++
  return result
}
export function confidenceText(tau: number | null): string {
  if (tau === null) return ''
  if (tau < 0.8) return '本赛季联赛较均衡，预测把握度偏低'
  if (tau < 1.05) return '当前数据差异有限，预测仍有不确定性'
  return '当前数据差异较明显，预测仍有不确定性'
}
export const outlookCopy = {
  title: '赛前数据倾向',
  disclaimer: '基于历史比赛结果的统计模型，仅供球迷参考，不代表比赛结果。',
  early: '赛季初，参考价值有限',
  empty: '暂无数据倾向',
  record: '公开赛季战绩',
}
export function recordSummary(record: OutlookRecord): string {
  if (!record.matches || record.rps_improvement === null)
    return '尚无公开计分场次'
  const direction = record.rps_improvement >= 0 ? '改善' : '变差'
  return `命中 ${record.hits}/${record.matches} · RPS较历史频率基线${direction} ${Math.abs(record.rps_improvement).toFixed(4)}`
}
