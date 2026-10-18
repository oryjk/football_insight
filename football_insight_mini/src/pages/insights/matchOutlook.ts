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
export const outlookCopy = {
  title: '赛前数据倾向',
  disclaimer: '基于历史比赛结果的统计模型，仅供球迷参考，不代表比赛结果。',
  empty: '暂无数据倾向',
}
