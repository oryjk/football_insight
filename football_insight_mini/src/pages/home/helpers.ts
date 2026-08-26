import type {
  MatchCard,
  MatchTechnicalStat,
  OverviewMatch,
  OverviewPlayer,
  OverviewStanding,
  PlayerRankingCategory,
  RankingsViewResponse,
  RoundReference,
  StandingsTable,
} from '../../types/insight'
import type { SupportMatchDetail } from '../../types/support'
import { resolveMatchDisplayStatus } from '../matches/helpers'
import type { HomeBriefingMarqueeAccent } from '../../utils/homeBriefingMarquees'

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

export interface HomePulseLeadMatch {
  match_id: number
  round_number: number
  match_date: string
  match_time: string
  home_team_name: string
  home_score: string
  away_team_name: string
  away_score: string
  technical_stats: MatchTechnicalStat[]
  home_corners: number | null
  away_corners: number | null
  status: 'live' | 'finished'
}

export interface HomeGuideLeaders {
  topTeamNames: string[]
  topScorerNames: string[]
  source: 'live' | 'previous_round'
}

export interface HomePulseTechStat {
  key: string
  label: string
  homeValue: string
  awayValue: string
  homeBarPercent: number
  awayBarPercent: number
}

export interface HomeLoadPlan {
  critical: string[]
  deferred: string[]
  onDemand: string[]
}

export interface HomeTeamSeasonMatch {
  matchId: number
  roundNumber: number
  matchDate: string
  matchTime: string
  homeTeamName: string
  awayTeamName: string
  homeScore: string
  awayScore: string
  teamName: string
  teamAvatar: string | null
  opponentName: string
  opponentAvatar: string | null
  isHomeTeam: boolean
  displayStatus: MatchCard['status']
  resultLabel: '胜' | '平' | '负' | '进' | '未'
  resultTone: 'win' | 'draw' | 'loss' | 'live' | 'scheduled'
  scoreText: string
  venueLabel: '主' | '客'
}

function mapLiveMatchToPulseMatch(
  match: Pick<MatchCard, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners'>,
): HomePulseLeadMatch {
  return {
    match_id: match.match_id,
    round_number: match.round_number,
    match_date: match.match_date,
    match_time: match.match_time,
    home_team_name: match.home_team_name,
    home_score: match.home_score,
    away_team_name: match.away_team_name,
    away_score: match.away_score,
    technical_stats: match.technical_stats,
    home_corners: match.home_corners,
    away_corners: match.away_corners,
    status: 'live',
  }
}

function mergeRecentMatchWithLiveMatch(
  match: Pick<OverviewMatch, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners'>,
  liveMatch: Pick<MatchCard, 'technical_stats' | 'home_corners' | 'away_corners'> | undefined,
): HomePulseLeadMatch {
  return {
    ...match,
    technical_stats: liveMatch?.technical_stats ?? match.technical_stats ?? [],
    home_corners: liveMatch?.home_corners ?? match.home_corners ?? null,
    away_corners: liveMatch?.away_corners ?? match.away_corners ?? null,
    status: 'finished',
  }
}

export function resolveHomeHasAuthToken(token: string | null): boolean {
  return typeof token === 'string' && token.trim().length > 0
}

export function shouldShowHomeSupportLoading(options: {
  loading: boolean
  hasCachedProfile: boolean
}): boolean {
  return options.loading && !options.hasCachedProfile
}

export function isHomeAuthExpiredMessage(message: string): boolean {
  const normalized = message.trim().toLowerCase()
  return normalized.includes('401')
    || normalized.includes('登录')
    || normalized.includes('unauthorized')
    || normalized.includes('not logged in')
    || normalized.includes('invalid token')
    || normalized.includes('token expired')
}

export function resolveHomeLoadPlan(hasAuthToken: boolean): HomeLoadPlan {
  return {
    critical: ['overview'],
    deferred: hasAuthToken
      ? ['rankings', 'matches', 'rounds', 'public-config', 'support-profile', 'support-teams']
      : ['rankings', 'matches', 'rounds', 'public-config'],
    onDemand: hasAuthToken ? ['auth-me'] : ['auth-me', 'support-teams'],
  }
}

export function resolveHomeSupportWindowShortLabel(
  match: Pick<SupportMatchDetail, 'support_window_status'> | null,
): string {
  if (!match) {
    return ''
  }

  if (match.support_window_status !== 'closed') {
    return 'VS'
  }

  return '已关闭'
}

export function resolveHomeSupportNextMatchLabel(
  match: Pick<SupportMatchDetail, 'support_window_status' | 'status'> | null,
): string {
  if (!match) {
    return ''
  }

  if (match.support_window_status !== 'closed') {
    return '随时进入助力页'
  }

  return match.status === 'finished' ? '比赛已完赛' : '比赛已开始'
}

export function resolveHomeSupportTeamRankLabel(rankNo: number | null | undefined): string {
  return rankNo && rankNo > 0 ? `积分榜第 ${rankNo}` : '排名待同步'
}

export function resolveHomeSupportMatchWeekdayLabel(matchDate: string | null | undefined): string {
  const normalizedDate = matchDate?.trim()
  if (!normalizedDate) {
    return ''
  }

  const dateMatch = /^(\d{4})-(\d{2})-(\d{2})$/.exec(normalizedDate)
  if (!dateMatch) {
    return ''
  }

  const year = Number(dateMatch[1])
  const monthIndex = Number(dateMatch[2]) - 1
  const day = Number(dateMatch[3])
  const parsedDate = new Date(year, monthIndex, day)

  if (
    parsedDate.getFullYear() !== year
    || parsedDate.getMonth() !== monthIndex
    || parsedDate.getDate() !== day
  ) {
    return ''
  }

  return ['周日', '周一', '周二', '周三', '周四', '周五', '周六'][parsedDate.getDay()]
}

export function resolveHomePulseLeadMatch(
  liveMatches: Array<Pick<MatchCard, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners' | 'status'>>,
  recentMatches: Array<Pick<OverviewMatch, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners'>>,
  nowIso = new Date().toISOString(),
): HomePulseLeadMatch | null {
  return resolveHomePulseMatches(liveMatches, recentMatches, nowIso)[0] ?? null
}

export function resolveHomePulseMatches(
  liveMatches: Array<Pick<MatchCard, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners' | 'status'>>,
  recentMatches: Array<Pick<OverviewMatch, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners'>>,
  nowIso = new Date().toISOString(),
): HomePulseLeadMatch[] {
  const activeLiveMatches = liveMatches.filter((match) =>
    resolveMatchDisplayStatus(match, nowIso) === 'live',
  )
  if (activeLiveMatches.length) {
    return activeLiveMatches.map(mapLiveMatchToPulseMatch)
  }

  const recentMatch = recentMatches[0]
  if (!recentMatch) {
    return []
  }

  const liveMatchMap = new Map(liveMatches.map((match) => [match.match_id, match]))

  return recentMatches
    .slice(0, 3)
    .map((match) => mergeRecentMatchWithLiveMatch(match, liveMatchMap.get(match.match_id)))
}

export function formatHomePulseCornerText(
  match: Pick<HomePulseLeadMatch, 'home_corners' | 'away_corners'>,
): string {
  if (match.home_corners === null || match.away_corners === null) {
    return ''
  }

  return `角球 ${match.home_corners} : ${match.away_corners}`
}

export function resolveHomePulseTechStats(
  match: Pick<HomePulseLeadMatch, 'technical_stats' | 'home_corners' | 'away_corners'>,
): HomePulseTechStat[] {
  if (match.technical_stats.length) {
    return match.technical_stats.map((stat) => ({
      key: stat.slug,
      label: stat.label,
      homeValue: formatTechnicalStatValue(stat.home_value, stat.unit),
      awayValue: formatTechnicalStatValue(stat.away_value, stat.unit),
      homeBarPercent: resolveTechBarPercent(stat.home_value, stat.away_value),
      awayBarPercent: resolveTechBarPercent(stat.away_value, stat.home_value),
    }))
  }

  if (match.home_corners === null || match.away_corners === null) {
    return []
  }

  return [{
    key: 'corners',
    label: '角球',
    homeValue: String(match.home_corners),
    awayValue: String(match.away_corners),
    homeBarPercent: resolveTechBarPercent(match.home_corners, match.away_corners),
    awayBarPercent: resolveTechBarPercent(match.away_corners, match.home_corners),
  }]
}

function formatTechnicalStatValue(value: number, unit: string | null): string {
  return unit ? `${value}${unit}` : String(value)
}

function resolveTechBarPercent(value: number, oppositeValue: number): number {
  const maxValue = Math.max(value, oppositeValue)
  if (maxValue === 0) {
    return 0
  }

  const percent = Math.round((value / maxValue) * 100)
  if (value === 0) {
    return 0
  }

  return Math.max(percent, 14)
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

export function resolveHomeTeamSeasonMatches(
  team: Pick<OverviewStanding, 'team_id' | 'team_name'>,
  matches: Array<Pick<MatchCard, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'status' | 'home_team_id' | 'home_team_name' | 'home_score' | 'away_team_id' | 'away_team_name' | 'away_score' | 'home_team_avatar' | 'away_team_avatar'>>,
  nowIso = new Date().toISOString(),
): HomeTeamSeasonMatch[] {
  return matches
    .filter((match) =>
      match.home_team_id === team.team_id
      || match.away_team_id === team.team_id
      || match.home_team_name === team.team_name
      || match.away_team_name === team.team_name,
    )
    .sort((left, right) => {
      const leftStatusRank = resolveSeasonMatchSortRank(resolveMatchDisplayStatus(left, nowIso))
      const rightStatusRank = resolveSeasonMatchSortRank(resolveMatchDisplayStatus(right, nowIso))
      if (leftStatusRank !== rightStatusRank) {
        return leftStatusRank - rightStatusRank
      }

      const leftAt = resolveKickoffSortValue(left.match_date, left.match_time)
      const rightAt = resolveKickoffSortValue(right.match_date, right.match_time)
      return leftAt - rightAt
    })
    .map((match) => {
      const isHomeTeam = match.home_team_id === team.team_id || match.home_team_name === team.team_name
      const displayStatus = resolveMatchDisplayStatus(match, nowIso)
      const homeScoreNum = Number(match.home_score)
      const awayScoreNum = Number(match.away_score)
      const hasScores = !Number.isNaN(homeScoreNum) && !Number.isNaN(awayScoreNum)

      const leftScore = isHomeTeam ? match.home_score : match.away_score
      const rightScore = isHomeTeam ? match.away_score : match.home_score
      const scoreText = displayStatus === 'scheduled' || !hasScores ? 'VS' : `${leftScore} : ${rightScore}`

      const teamScoreNum = isHomeTeam ? homeScoreNum : awayScoreNum
      const opponentScoreNum = isHomeTeam ? awayScoreNum : homeScoreNum

      const opponentName = isHomeTeam ? match.away_team_name : match.home_team_name
      const venueLabel = isHomeTeam ? '主' : '客'

      if (displayStatus === 'live') {
        return {
          matchId: match.match_id,
          roundNumber: match.round_number,
          matchDate: match.match_date,
          matchTime: match.match_time,
          homeTeamName: match.home_team_name,
          awayTeamName: match.away_team_name,
          homeScore: match.home_score,
          awayScore: match.away_score,
          teamName: team.team_name,
          teamAvatar: isHomeTeam ? match.home_team_avatar : match.away_team_avatar,
          opponentName,
          opponentAvatar: isHomeTeam ? match.away_team_avatar : match.home_team_avatar,
          isHomeTeam,
          displayStatus,
          resultLabel: '进',
          resultTone: 'live',
          scoreText,
          venueLabel,
        }
      }

      if (displayStatus === 'scheduled' || !hasScores) {
        return {
          matchId: match.match_id,
          roundNumber: match.round_number,
          matchDate: match.match_date,
          matchTime: match.match_time,
          homeTeamName: match.home_team_name,
          awayTeamName: match.away_team_name,
          homeScore: match.home_score,
          awayScore: match.away_score,
          teamName: team.team_name,
          teamAvatar: isHomeTeam ? match.home_team_avatar : match.away_team_avatar,
          opponentName,
          opponentAvatar: isHomeTeam ? match.away_team_avatar : match.home_team_avatar,
          isHomeTeam,
          displayStatus,
          resultLabel: '未',
          resultTone: 'scheduled',
          scoreText,
          venueLabel,
        }
      }

      const resultTone = teamScoreNum > opponentScoreNum ? 'win' : teamScoreNum < opponentScoreNum ? 'loss' : 'draw'

      return {
        matchId: match.match_id,
        roundNumber: match.round_number,
        matchDate: match.match_date,
        matchTime: match.match_time,
        homeTeamName: match.home_team_name,
        awayTeamName: match.away_team_name,
        homeScore: match.home_score,
        awayScore: match.away_score,
        teamName: team.team_name,
        teamAvatar: isHomeTeam ? match.home_team_avatar : match.away_team_avatar,
        opponentName,
        opponentAvatar: isHomeTeam ? match.away_team_avatar : match.home_team_avatar,
        isHomeTeam,
        displayStatus,
        resultLabel: resultTone === 'win' ? '胜' : resultTone === 'loss' ? '负' : '平',
        resultTone,
        scoreText,
        venueLabel,
      }
    })
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

function resolveKickoffSortValue(matchDate: string, matchTime: string): number {
  const normalizedTime = matchTime?.trim().length === 5 ? `${matchTime}:00` : matchTime
  const kickoffAt = new Date(`${matchDate}T${normalizedTime}+08:00`).getTime()
  return Number.isNaN(kickoffAt) ? 0 : kickoffAt
}

function resolveSeasonMatchSortRank(status: MatchCard['status']): number {
  if (status === 'finished') {
    return 0
  }

  if (status === 'live') {
    return 1
  }

  return 2
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

export function formatHomeTeamSeasonRecord(matches: Array<Pick<HomeTeamSeasonMatch, 'resultTone'>>): string {
  const counts = { win: 0, draw: 0, loss: 0 }
  for (const match of matches) {
    if (match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss') {
      counts[match.resultTone] += 1
    }
  }

  return `${counts.win}胜 ${counts.draw}平 ${counts.loss}负`
}

export function formatHomeEntityNames(names: string[]): string {
  return names.join('、')
}
