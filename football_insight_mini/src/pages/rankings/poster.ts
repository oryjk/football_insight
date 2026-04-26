import type { StandingsTable, StandingsTableEntry } from '../../types/insight'

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

function getPosterPointsLabel(table: StandingsTable): string {
  return table.slug === 'standings_without_penalty' ? '理论积分' : '积分'
}

function formatAdjustment(value: number): string {
  if (value > 0) {
    return `+${value}`
  }

  return `${value}`
}
