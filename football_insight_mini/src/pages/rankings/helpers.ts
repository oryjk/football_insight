import type { StandingsTable } from '../../types/insight'

export function resolveStandingsTablePriority(slug: string): number {
  if (slug === 'standings_with_penalty') {
    return 0
  }

  if (slug === 'standings_without_penalty') {
    return 1
  }

  return 2
}

export function sortStandingsPreviewTables(tables: StandingsTable[]): StandingsTable[] {
  return [...tables].sort(
    (left, right) => resolveStandingsTablePriority(left.slug) - resolveStandingsTablePriority(right.slug),
  )
}

export function buildStandingsPreviewSummary(table: StandingsTable): string {
  if (table.slug === 'standings_without_penalty') {
    return `榜首是 ${table.entries[0]?.team_name ?? '暂无数据'}，按理论积分重新排序。`
  }

  const impactedCount = table.entries.filter((item) => item.points_adjustment !== 0).length
  return impactedCount > 0
    ? `${impactedCount} 支球队当前存在积分调整，查看完整实际积分榜。`
    : '当前所有球队积分与理论积分一致。'
}

export function buildStandingsPosterSubtitle(options: {
  season: number
  round: number | null
  table: Pick<StandingsTable, 'label'>
}): string {
  return options.round
    ? `${options.season} ${options.table.label} · 第 ${options.round} 轮`
    : `${options.season} ${options.table.label}`
}

export function truncateStandingsPosterTeamName(name: string): string {
  return name.length > 10 ? name.slice(0, 9) + '…' : name
}
