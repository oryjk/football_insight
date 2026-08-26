import type { MatchCard } from '../types/insight'
import { resolveMatchDisplayStatus } from '../pages/matches/helpers'

export interface TeamSeasonMatch {
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
  resultLabel: '胜' | '平' | '负' | '进' | '未' | '延'
  resultTone: 'win' | 'draw' | 'loss' | 'live' | 'scheduled' | 'postponed'
  scoreText: string
  venueLabel: '主' | '客'
  focusKind: 'latest-finished' | 'next-scheduled' | null
}

function resolveKickoffSortValue(matchDate: string, matchTime: string): number {
  const timeValue = matchTime || '00:00'
  const normalized = `${matchDate}T${timeValue}:00+08:00`
  const timestamp = Date.parse(normalized)
  return Number.isNaN(timestamp) ? 0 : timestamp
}

export function buildTeamSeasonMatchRowId(matchId: number): string {
  return `team-season-match-${matchId}`
}

export function resolveTeamSeasonMatches(
  team: { team_id: number; team_name: string },
  matches: MatchCard[],
  nowIso = new Date().toISOString(),
): TeamSeasonMatch[] {
  const teamMatches = matches
    .filter((match) =>
      match.home_team_id === team.team_id
      || match.away_team_id === team.team_id
      || match.home_team_name === team.team_name
      || match.away_team_name === team.team_name,
    )

  const latestFinishedMatchId = teamMatches
    .filter((match) => resolveMatchDisplayStatus(match, nowIso) === 'finished')
    .sort((left, right) =>
      resolveKickoffSortValue(right.match_date, right.match_time)
      - resolveKickoffSortValue(left.match_date, left.match_time),
    )[0]?.match_id ?? null

  const nextScheduledMatchId = teamMatches
    .filter((match) => resolveMatchDisplayStatus(match, nowIso) === 'scheduled')
    .sort((left, right) =>
      resolveKickoffSortValue(left.match_date, left.match_time)
      - resolveKickoffSortValue(right.match_date, right.match_time),
    )[0]?.match_id ?? null

  return teamMatches
    .sort((left, right) => {
      const leftStatus = resolveMatchDisplayStatus(left, nowIso)
      const rightStatus = resolveMatchDisplayStatus(right, nowIso)
      const order = { finished: 0, live: 1, postponed: 2, scheduled: 3 } as const
      const leftPriority = order[leftStatus as keyof typeof order] ?? 3
      const rightPriority = order[rightStatus as keyof typeof order] ?? 3

      if (leftPriority !== rightPriority) {
        return leftPriority - rightPriority
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
      const scoreText = displayStatus === 'postponed'
        ? '延期'
        : displayStatus === 'scheduled' || !hasScores ? 'VS' : `${leftScore} : ${rightScore}`

      const teamScoreNum = isHomeTeam ? homeScoreNum : awayScoreNum
      const opponentScoreNum = isHomeTeam ? awayScoreNum : homeScoreNum

      const opponentName = isHomeTeam ? match.away_team_name : match.home_team_name
      const venueLabel = isHomeTeam ? '主' : '客'
      const focusKind = match.match_id === latestFinishedMatchId
        ? 'latest-finished'
        : match.match_id === nextScheduledMatchId ? 'next-scheduled' : null

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
          focusKind,
        }
      }

      if (displayStatus === 'postponed') {
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
          resultLabel: '延',
          resultTone: 'postponed',
          scoreText,
          venueLabel,
          focusKind,
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
          focusKind,
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
        focusKind,
      }
    })
}

/** 赛季战绩文案（x胜 y平 z负）；live/scheduled 不计入。 */
export function formatTeamSeasonRecord(matches: Array<Pick<TeamSeasonMatch, 'resultTone'>>): string {
  const counts = { win: 0, draw: 0, loss: 0 }
  for (const match of matches) {
    if (match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss') {
      counts[match.resultTone] += 1
    }
  }

  return `${counts.win}胜 ${counts.draw}平 ${counts.loss}负`
}
