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

export interface StandingsPodiumEntry {
  rank: number
  teamId: number
  teamName: string
  avatar: string | null
}

export interface StandingsPreviewFacts {
  podium: StandingsPodiumEntry[]
  adjustedCount: number
}

/** What the launcher card shows instead of prose: the top three and how many teams carry a points adjustment. */
export function buildStandingsPreviewFacts(table: StandingsTable): StandingsPreviewFacts {
  return {
    podium: table.entries.slice(0, 3).map((entry, index) => ({
      rank: index + 1,
      teamId: entry.team_id,
      teamName: entry.team_name,
      avatar: entry.avatar_storage_url,
    })),
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
