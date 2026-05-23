import type { StandingsTable, StandingsTableEntry } from '../../types/insight'

export type StandingsRankingMode = 'with_penalty' | 'without_penalty'

export interface StandingsRankingEntry {
  rank_no: number
  team_id: number
  team_name: string
  score_value: string
  avatar_storage_url: string | null
}

export interface StandingsPosterColumn {
  label: string
  x: number
  align?: 'left' | 'center'
}

export interface StandingsPosterMetric {
  value: string
  x: number
  highlight?: boolean
  compact?: boolean
}

export interface StandingsPosterTeamLayout {
  logoX: number
  logoSize: number
  nameX: number
}

export interface StandingsFallbackMetric {
  label: string
  value: string
}

export function buildStandingsPosterShareTitle(season: number, table: Pick<StandingsTable, 'label'>): string {
  return `${season} 中超${table.label}`
}

export function buildStandingsPosterSharePath(table: Pick<StandingsTable, 'slug'>): string {
  return `/pages/rankings/index?mode=rankings&openPoster=1&table=${encodeURIComponent(table.slug)}`
}

export function buildStandingsRankingEntries(
  tables: StandingsTable[],
  mode: StandingsRankingMode,
): StandingsRankingEntry[] {
  const table = resolveStandingsRankingTable(tables, mode)
  if (!table) {
    return []
  }

  return table.entries.map((entry) => ({
    rank_no: entry.rank_no,
    team_id: entry.team_id,
    team_name: entry.team_name,
    score_value: String(getDisplayedPoints(table, entry)),
    avatar_storage_url: entry.avatar_storage_url,
  }))
}

export function buildStandingsPosterColumns(table: StandingsTable): StandingsPosterColumn[] {
  return [
    { label: '排名', x: 88 },
    { label: '球队', x: 200 },
    { label: getPosterPointsLabel(table), x: 818 },
    { label: '进球', x: 904 },
    { label: '失球', x: 980 },
  ]
}

export function buildStandingsPosterMetrics(
  table: StandingsTable,
  entry: StandingsTableEntry,
): StandingsPosterMetric[] {
  const metrics: StandingsPosterMetric[] = [
    {
      value: String(getDisplayedPoints(table, entry)),
      x: 818,
    },
    {
      value: String(entry.goals_for),
      x: 904,
    },
    {
      value: String(entry.goals_against),
      x: 980,
    },
  ]

  if (table.slug === 'standings_with_penalty' && entry.points_adjustment !== 0) {
    metrics.unshift({
      value: formatAdjustment(entry.points_adjustment),
      x: 760,
      highlight: true,
      compact: true,
    })
  }

  return metrics
}

export function buildStandingsFallbackMetrics(
  table: StandingsTable,
  entry: StandingsTableEntry,
): StandingsFallbackMetric[] {
  return [
    {
      label: getPosterPointsLabel(table),
      value: String(getDisplayedPoints(table, entry)),
    },
    {
      label: '进球',
      value: String(entry.goals_for),
    },
    {
      label: '失球',
      value: String(entry.goals_against),
    },
  ]
}

export function buildStandingsPosterTeamLayout(hasLogo: boolean): StandingsPosterTeamLayout {
  if (!hasLogo) {
    return {
      logoX: 200,
      logoSize: 0,
      nameX: 200,
    }
  }

  return {
    logoX: 200,
    logoSize: 28,
    nameX: 240,
  }
}

function getDisplayedPoints(table: StandingsTable, entry: StandingsTableEntry): number {
  return table.slug === 'standings_without_penalty' ? entry.points_without_penalty : entry.points
}

function resolveStandingsRankingTable(
  tables: StandingsTable[],
  mode: StandingsRankingMode,
): StandingsTable | null {
  const preferredSlug = mode === 'without_penalty' ? 'standings_without_penalty' : 'standings_with_penalty'
  return tables.find((table) => table.slug === preferredSlug)
    ?? tables.find((table) => table.slug === 'standings')
    ?? tables[0]
    ?? null
}

function getPosterPointsLabel(table: StandingsTable): string {
  return table.slug === 'standings_without_penalty' ? '理论积分' : '积分'
}

function formatAdjustment(value: number): string {
  if (value > 0) {
    return `+${value}`
  }

  return `${value}`
}
