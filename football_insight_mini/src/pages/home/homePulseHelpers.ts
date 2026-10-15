import type { MatchCard, MatchTechnicalStat, OverviewMatch } from '../../types/insight'
import { LIVE_INFERENCE_WINDOW_MS, parseMatchKickoff, resolveMatchDisplayStatus } from '../../utils/matchDisplayStatus'

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

export interface HomePulseTechStat {
  key: string
  label: string
  homeValue: string
  awayValue: string
  homeBarPercent: number
  awayBarPercent: number
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

function mapFinishedMatchToPulseMatch(
  match: Pick<MatchCard, 'match_id' | 'round_number' | 'match_date' | 'match_time' | 'home_team_name' | 'home_score' | 'away_team_name' | 'away_score' | 'technical_stats' | 'home_corners' | 'away_corners'>,
): HomePulseLeadMatch {
  return { ...mapLiveMatchToPulseMatch(match), status: 'finished' }
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

  // 无进行中比赛：展示最近一轮已完赛的全部场次（live 列表即最新一轮），
  // 首屏 live 列表未返回时退回概览里的最近完赛。
  const finishedRound = liveMatches
    .filter((match) => match.status === 'finished')
    .map((match) => mapFinishedMatchToPulseMatch(match))
  const finished = finishedRound.length
    ? finishedRound
    : recentMatches
        .filter((match) => match.round_number === recentMatches[0]?.round_number)
        .map((match) => mergeRecentMatchWithLiveMatch(match, undefined))

  return isHomePulseResultFresh(finished, nowIso) ? finished : []
}

/** 最近一轮最晚一场完赛超过 24 小时后，不再作为"实时"信息展示。 */
export const HOME_PULSE_RESULT_FRESH_WINDOW_MS = 24 * 60 * 60 * 1000

function isHomePulseResultFresh(
  matches: Array<Pick<HomePulseLeadMatch, 'match_date' | 'match_time'>>,
  nowIso: string,
): boolean {
  const nowAt = new Date(nowIso).getTime()
  const kickoffs = matches
    .map((match) => parseMatchKickoff(match.match_date, match.match_time))
    .filter((value): value is number => value !== null)

  if (!kickoffs.length || Number.isNaN(nowAt)) {
    return false
  }

  return nowAt <= Math.max(...kickoffs) + LIVE_INFERENCE_WINDOW_MS + HOME_PULSE_RESULT_FRESH_WINDOW_MS
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
