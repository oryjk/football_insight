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
  opponentName: string
  isHomeTeam: boolean
  displayStatus: MatchCard['status']
  resultLabel: '胜' | '平' | '负' | '进行中' | '未开赛'
  resultTone: 'win' | 'draw' | 'loss' | 'live' | 'scheduled'
  scoreText: string
}

function resolveKickoffSortValue(matchDate: string, matchTime: string): number {
  const timeValue = matchTime || '00:00'
  const normalized = `${matchDate}T${timeValue}:00+08:00`
  const timestamp = Date.parse(normalized)
  return Number.isNaN(timestamp) ? 0 : timestamp
}

export function resolveTeamSeasonMatches(
  team: { team_id: number; team_name: string },
  matches: MatchCard[],
  nowIso = new Date().toISOString(),
): TeamSeasonMatch[] {
  return matches
    .filter((match) =>
      match.home_team_id === team.team_id
      || match.away_team_id === team.team_id
      || match.home_team_name === team.team_name
      || match.away_team_name === team.team_name,
    )
    .sort((left, right) => {
      const leftStatus = resolveMatchDisplayStatus(left, nowIso)
      const rightStatus = resolveMatchDisplayStatus(right, nowIso)
      const order = { finished: 0, live: 1, scheduled: 2 } as const
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
      const homeScore = Number(match.home_score)
      const awayScore = Number(match.away_score)
      const scoreText = `${match.home_score} : ${match.away_score}`

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
          opponentName: isHomeTeam ? match.away_team_name : match.home_team_name,
          isHomeTeam,
          displayStatus,
          resultLabel: '进行中',
          resultTone: 'live',
          scoreText,
        }
      }

      if (displayStatus === 'scheduled' || Number.isNaN(homeScore) || Number.isNaN(awayScore)) {
        return {
          matchId: match.match_id,
          roundNumber: match.round_number,
          matchDate: match.match_date,
          matchTime: match.match_time,
          homeTeamName: match.home_team_name,
          awayTeamName: match.away_team_name,
          homeScore: match.home_score,
          awayScore: match.away_score,
          opponentName: isHomeTeam ? match.away_team_name : match.home_team_name,
          isHomeTeam,
          displayStatus,
          resultLabel: '未开赛',
          resultTone: 'scheduled',
          scoreText: 'VS',
        }
      }

      const teamScore = isHomeTeam ? homeScore : awayScore
      const opponentScore = isHomeTeam ? awayScore : homeScore
      const resultTone = teamScore > opponentScore ? 'win' : teamScore < opponentScore ? 'loss' : 'draw'

      return {
        matchId: match.match_id,
        roundNumber: match.round_number,
        matchDate: match.match_date,
        matchTime: match.match_time,
        homeTeamName: match.home_team_name,
        awayTeamName: match.away_team_name,
        homeScore: match.home_score,
        awayScore: match.away_score,
        opponentName: isHomeTeam ? match.away_team_name : match.home_team_name,
        isHomeTeam,
        displayStatus,
        resultLabel: resultTone === 'win' ? '胜' : resultTone === 'loss' ? '负' : '平',
        resultTone,
        scoreText,
      }
    })
}
