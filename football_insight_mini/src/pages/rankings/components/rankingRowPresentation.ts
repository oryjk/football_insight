// 前 3 名使用奖牌徽标和行高亮，其余名次直接展示数字。
export function resolveRankBadgeClass(rankNo: number): Record<string, boolean> {
  return {
    'ranking-row__rank--medal': rankNo <= 3,
    'ranking-row__rank--1': rankNo === 1,
    'ranking-row__rank--2': rankNo === 2,
    'ranking-row__rank--3': rankNo === 3,
  }
}

export function resolveRankRowClass(rankNo: number): Record<string, boolean> {
  return {
    'ranking-row--top-1': rankNo === 1,
    'ranking-row--top-2': rankNo === 2,
    'ranking-row--top-3': rankNo === 3,
  }
}

export function formatRankLabel(rankNo: number): string {
  return `${rankNo}`
}
