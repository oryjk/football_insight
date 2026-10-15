import type { OverviewStanding, StandingsTable } from '../../types/insight'
import { buildStandingsRankingEntries, type StandingsRankingMode } from './poster'

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

export interface StandingsPreviewFacts {
  leaderName: string
  leaderAvatar: string | null
  adjustedCount: number
}

/** The few facts the launcher card shows instead of a sentence of prose. */
export function buildStandingsPreviewFacts(table: StandingsTable): StandingsPreviewFacts {
  const leader = table.entries[0]
  return {
    leaderName: leader?.team_name ?? '暂无数据',
    leaderAvatar: leader?.avatar_storage_url ?? null,
    adjustedCount: table.entries.filter((item) => item.points_adjustment !== 0).length,
  }
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

export function buildStandingsTeams(tables: StandingsTable[], mode: StandingsRankingMode): OverviewStanding[] {
  return buildStandingsRankingEntries(tables, mode).map((entry) => ({
    rank_no: entry.rank_no,
    team_id: entry.team_id,
    team_name: entry.team_name,
    avatar_storage_url: entry.avatar_storage_url,
    points: Number(entry.score_value),
    recent_form: entry.recent_form,
  }))
}
