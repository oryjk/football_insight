import type { MatchCard } from '../../types/insight'

type MatchDisplayStatus = MatchCard['status']

export interface MatchTechStatRow {
  key: string
  label: string
  homeValue: string
  awayValue: string
  homeBarPercent: number
  awayBarPercent: number
}

export function resolveMatchDisplayStatus(
  match: Pick<MatchCard, 'status' | 'match_date' | 'match_time'>,
  nowIso = new Date().toISOString(),
): MatchDisplayStatus {
  if (match.status === 'finished' || match.status === 'live') {
    return match.status
  }

  const kickoffAt = parseMatchKickoff(match.match_date, match.match_time)
  const nowAt = new Date(nowIso).getTime()

  if (!kickoffAt || Number.isNaN(nowAt)) {
    return match.status
  }

  if (nowAt >= kickoffAt) {
    return 'live'
  }

  return 'scheduled'
}

export function buildCurrentRoundSectionTitle(
  matches: Array<Pick<MatchCard, 'status'>>,
): string {
  return matches.some((match) => match.status === 'live') ? '本轮进行中' : '本轮未开赛'
}

export function formatMatchScoreboardText(
  match: Pick<MatchCard, 'status' | 'match_date' | 'match_time' | 'home_score' | 'away_score'>,
  nowIso = new Date().toISOString(),
): string {
  const displayStatus = resolveMatchDisplayStatus(match, nowIso)

  if ((displayStatus === 'live' || displayStatus === 'finished') && hasMatchScores(match)) {
    return `${match.home_score} : ${match.away_score}`
  }

  return displayStatus === 'live' ? '进行中' : '未开赛'
}

export function shouldShowLiveStatusTag(
  match: Pick<MatchCard, 'status' | 'match_date' | 'match_time'>,
  nowIso = new Date().toISOString(),
): boolean {
  return resolveMatchDisplayStatus(match, nowIso) === 'live'
}

export function hasMatchTechStats(
  match: Pick<MatchCard, 'technical_stats' | 'home_corners' | 'away_corners'>,
): boolean {
  return resolveMatchTechStats(match).length > 0
}

export function resolveMatchTechStats(
  match: Pick<MatchCard, 'technical_stats' | 'home_corners' | 'away_corners'>,
): MatchTechStatRow[] {
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

function parseMatchKickoff(matchDate: string, matchTime: string): number | null {
  const date = matchDate?.trim()
  const time = matchTime?.trim()

  if (!date || !time) {
    return null
  }

  const normalizedTime = time.length === 5 ? `${time}:00` : time
  const kickoff = new Date(`${date}T${normalizedTime}+08:00`).getTime()

  return Number.isNaN(kickoff) ? null : kickoff
}

function hasMatchScores(
  match: Pick<MatchCard, 'home_score' | 'away_score'>,
): boolean {
  return match.home_score.trim().length > 0 && match.away_score.trim().length > 0
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
