import type { OverviewPlayer, OverviewStanding, PlayerRankingCategory, RankingsViewResponse, RoundReference, StandingsTable } from '../../types/insight'
import type { HomeBriefingMarqueeAccent } from '../../utils/homeBriefingMarquees'
import type { HomePulseLeadMatch } from './homePulseHelpers'

export type HomeHeroGuide =
  | { mode: 'team-and-scorer-with-match'; topTeamName: string; topScorerName: string }
  | { mode: 'team-and-scorer-with-live-match'; topTeamName: string; topScorerName: string }
  | { mode: 'team-and-scorer'; topTeamName: string; topScorerName: string }
  | { mode: 'fallback' }

export interface HomeBriefingItem {
  accent: HomeBriefingMarqueeAccent
  label: string
  value: string
  subValue: string | null
  entities: Array<{ name: string; caption: string; avatar: string | null }>
  metricValue: string
  metricLabel: string
  avatars: Array<{ name: string; src: string | null }>
}

export interface HomeGuideLeaders {
  topTeamNames: string[]
  topScorerNames: string[]
  source: 'live' | 'previous_round'
}

export function resolveHomeGuideReferenceRoundNumber(
  rounds: Array<Pick<RoundReference, 'round_number' | 'status' | 'completed_matches' | 'total_matches'>>,
): number | null {
  const currentRound = rounds.find((round) =>
    round.status === 'current' && round.completed_matches < round.total_matches,
  )

  if (!currentRound) {
    return null
  }

  const completedRounds = rounds
    .filter((round) =>
      round.status === 'completed' && round.round_number < currentRound.round_number,
    )
    .sort((left, right) => right.round_number - left.round_number)

  return completedRounds[0]?.round_number ?? null
}

export function resolveHomeGuideLeaders(options: {
  rounds: Array<Pick<RoundReference, 'round_number' | 'status' | 'completed_matches' | 'total_matches'>>
  liveStandings: Array<Pick<OverviewStanding, 'team_name' | 'points'>>
  liveScorers: Array<Pick<OverviewPlayer, 'player_name' | 'score_value'>>
  referenceRankings: RankingsViewResponse | null
}): HomeGuideLeaders {
  const referenceRoundNumber = resolveHomeGuideReferenceRoundNumber(options.rounds)
  if (referenceRoundNumber !== null && options.referenceRankings) {
    const topTeamNames = resolveTopTeamNamesFromRankings(options.referenceRankings)
    const topScorerNames = resolveTopScorerNamesFromRankings(options.referenceRankings)

    if (topTeamNames.length || topScorerNames.length) {
      return {
        topTeamNames,
        topScorerNames,
        source: 'previous_round',
      }
    }
  }

  return {
    topTeamNames: resolveTopTeamNamesFromLive(options.liveStandings),
    topScorerNames: resolveTopScorerNamesFromLive(options.liveScorers),
    source: 'live',
  }
}

export function resolveHomeGuideNote(source: HomeGuideLeaders['source']): string {
  return source === 'previous_round'
    ? '当前轮次尚未全部结束，榜首和射手判断先按上一轮结清结果来看。'
    : ''
}

function resolveTopTeamNamesFromLive(
  standings: Array<Pick<OverviewStanding, 'team_name' | 'points'>>,
): string[] {
  const topPoints = standings[0]?.points
  if (topPoints === undefined) {
    return []
  }

  return standings
    .filter((team) => team.points === topPoints)
    .map((team) => team.team_name)
}

function resolveTopScorerNamesFromLive(
  scorers: Array<Pick<OverviewPlayer, 'player_name' | 'score_value'>>,
): string[] {
  const topScoreValue = scorers[0]?.score_value
  if (topScoreValue === undefined) {
    return []
  }

  return scorers
    .filter((player) => player.score_value === topScoreValue)
    .map((player) => player.player_name)
}

function resolveTopTeamNamesFromRankings(rankings: RankingsViewResponse): string[] {
  const table = pickPrimaryStandingsTable(rankings.standings_tables)
  const topPoints = table?.entries[0]?.points
  if (!table || topPoints === undefined) {
    return []
  }

  return table.entries
    .filter((entry) => entry.points === topPoints)
    .map((entry) => entry.team_name)
}

function resolveTopScorerNamesFromRankings(rankings: RankingsViewResponse): string[] {
  const category = pickGoalScorerCategory(rankings.player_categories)
  const topScoreValue = category?.entries[0]?.score_value
  if (!category || topScoreValue === undefined) {
    return []
  }

  return category.entries
    .filter((entry) => entry.score_value === topScoreValue)
    .map((entry) => entry.player_name)
}

function pickPrimaryStandingsTable(tables: StandingsTable[]): StandingsTable | null {
  return tables.find((table) => table.slug === 'standings') ?? tables[0] ?? null
}

function pickGoalScorerCategory(
  categories: PlayerRankingCategory[],
): PlayerRankingCategory | null {
  return categories.find((category) => category.slug === 'goals') ?? categories[0] ?? null
}

export function buildHomeHeroGuide(options: {
  guideLeaders: HomeGuideLeaders
  leadMatch: Pick<HomePulseLeadMatch, 'status'> | null
}): HomeHeroGuide {
  const topTeamNames = formatHomeEntityNames(options.guideLeaders.topTeamNames)
  const topScorerNames = formatHomeEntityNames(options.guideLeaders.topScorerNames)

  if (topTeamNames && topScorerNames && options.leadMatch) {
    if (options.leadMatch.status === 'live') {
      return {
        mode: 'team-and-scorer-with-live-match',
        topTeamName: topTeamNames,
        topScorerName: topScorerNames,
      }
    }

    return {
      mode: 'team-and-scorer-with-match',
      topTeamName: topTeamNames,
      topScorerName: topScorerNames,
    }
  }

  if (topTeamNames && topScorerNames) {
    return {
      mode: 'team-and-scorer',
      topTeamName: topTeamNames,
      topScorerName: topScorerNames,
    }
  }

  return { mode: 'fallback' }
}

export function buildHomeBriefingItems(options: {
  topTeam: Pick<OverviewStanding, 'team_name' | 'points'> | null
  leadingTeams: Array<Pick<OverviewStanding, 'team_name' | 'points' | 'avatar_storage_url'>>
  topScorer: Pick<OverviewPlayer, 'player_name' | 'score_value'> | null
  leadingScorers: Array<Pick<OverviewPlayer, 'player_name' | 'team_name' | 'score_value' | 'avatar_storage_url'>>
  topAssist: Pick<OverviewPlayer, 'player_name' | 'score_value'> | null
  leadingAssists: Array<Pick<OverviewPlayer, 'player_name' | 'team_name' | 'score_value' | 'avatar_storage_url'>>
}): HomeBriefingItem[] {
  const items: Array<HomeBriefingItem | null> = [
    options.topTeam
      ? {
          accent: 'leader',
          label: '榜首风向',
          value: formatHomeEntityNames(options.leadingTeams.map((team) => team.team_name)),
          subValue: null,
          entities: [],
          metricValue: String(options.topTeam.points),
          metricLabel: options.leadingTeams.length > 1 ? '分并列榜首' : '分领跑',
          avatars: options.leadingTeams.map((team) => ({
            name: team.team_name,
            src: team.avatar_storage_url,
          })),
        }
      : null,
    options.topScorer
      ? {
          accent: 'scorer',
          label: '射手头条',
          value: formatHomeEntityNames(options.leadingScorers.map((player) => player.player_name)),
          subValue: formatHomeEntityNames(options.leadingScorers.map((player) => player.team_name)),
          entities: options.leadingScorers.map((player) => ({
            name: player.player_name,
            caption: player.team_name,
            avatar: player.avatar_storage_url,
          })),
          metricValue: String(options.topScorer.score_value),
          metricLabel: options.leadingScorers.length > 1 ? '球并列头名' : '球',
          avatars: options.leadingScorers.map((player) => ({
            name: player.player_name,
            src: player.avatar_storage_url,
          })),
        }
      : null,
    options.topAssist
      ? {
          accent: 'assist',
          label: '助攻头条',
          value: formatHomeEntityNames(options.leadingAssists.map((player) => player.player_name)),
          subValue: formatHomeEntityNames(options.leadingAssists.map((player) => player.team_name)),
          entities: options.leadingAssists.map((player) => ({
            name: player.player_name,
            caption: player.team_name,
            avatar: player.avatar_storage_url,
          })),
          metricValue: String(options.topAssist.score_value),
          metricLabel: options.leadingAssists.length > 1 ? '次并列头名' : '次助攻',
          avatars: options.leadingAssists.map((player) => ({
            name: player.player_name,
            src: player.avatar_storage_url,
          })),
        }
      : null,
  ]

  return items.filter((item): item is HomeBriefingItem => item !== null)
}

export function buildHomeHeadlineBody(options: {
  summary: string | null
  topScorer: Pick<OverviewPlayer, 'player_name'> | null
}): string {
  if (options.summary) {
    return options.summary
  }

  if (options.topScorer) {
    return `${options.topScorer.player_name}暂时领跑赛季累计射手榜，最近赛果和榜单头部值得继续跟踪。`
  }

  return '当前可查看积分榜头部、最近赛果和榜单焦点。'
}

export function buildHomeWatchPoints(options: {
  bullets: string[]
  leadMatch: Pick<HomePulseLeadMatch, 'status' | 'home_team_name' | 'away_team_name'> | null
  topTeam: Pick<OverviewStanding, 'team_name'> | null
  topScorer: Pick<OverviewPlayer, 'player_name'> | null
  secondScorer: Pick<OverviewPlayer, 'player_name'> | null
}): string[] {
  if (options.bullets.length) {
    return options.bullets
  }

  const items: string[] = []

  if (options.leadMatch) {
    items.push(
      options.leadMatch.status === 'live'
        ? `当前进行中焦点是 ${options.leadMatch.home_team_name} 对阵 ${options.leadMatch.away_team_name}`
        : `最新完赛焦点是 ${options.leadMatch.home_team_name} 对阵 ${options.leadMatch.away_team_name}`,
    )
  }

  if (options.topTeam) {
    items.push(`${options.topTeam.team_name} 继续占据积分榜头部位置`)
  }

  if (options.topScorer && options.secondScorer) {
    items.push(`${options.topScorer.player_name} 与 ${options.secondScorer.player_name} 正在争夺赛季累计射手榜头部`)
  }

  return items
}

export function formatHomeEntityNames(names: string[]): string {
  return names.join('、')
}
