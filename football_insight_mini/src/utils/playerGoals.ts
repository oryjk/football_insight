import type { OverviewPlayer, PlayerMatchGoals } from '../types/insight'

export type PlayerGoalEntry = Pick<OverviewPlayer,
  'rank_no' | 'player_id' | 'player_name' | 'team_name' | 'score_value' | 'avatar_storage_url' | 'recent_goals'>

export function normalizeRecentGoals(rows?: PlayerMatchGoals[]): PlayerMatchGoals[] {
  return (rows ?? [])
    .filter(row => Number.isInteger(row.goals) && row.goals >= 0 && row.goals <= 30
      && Number.isInteger(row.minutes_played) && row.minutes_played > 0)
    .slice()
    .sort((a, b) => a.match_date.localeCompare(b.match_date)
      || a.match_time.localeCompare(b.match_time) || a.match_id - b.match_id)
    .slice(-10)
}

export function goalChartScale(series: Array<PlayerMatchGoals[] | undefined>): number {
  return Math.max(3, ...series.flatMap(rows => normalizeRecentGoals(rows).map(row => row.goals)))
}

export function buildGoalMatchDetail(name: string, match: PlayerMatchGoals) {
  return {
    title: `${name} · ${match.goals} 球${match.goals === 3 ? ' · 帽子戏法' : ''}`,
    content: `${match.match_date} ${match.match_time}\n第 ${match.round_number} 轮 · ${match.home_team_name} vs ${match.away_team_name}\n出场 ${match.minutes_played} 分钟`,
  }
}
