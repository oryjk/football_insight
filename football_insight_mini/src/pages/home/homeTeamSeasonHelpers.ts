import type { MatchCard, OverviewStanding } from '../../types/insight'
import { resolveMatchDisplayStatus } from '../../utils/matchDisplayStatus'

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

export function formatHomeTeamSeasonRecord(matches: Array<Pick<HomeTeamSeasonMatch, 'resultTone'>>): string {
  const counts = { win: 0, draw: 0, loss: 0 }
  for (const match of matches) {
    if (match.resultTone === 'win' || match.resultTone === 'draw' || match.resultTone === 'loss') {
      counts[match.resultTone] += 1
    }
  }

  return `${counts.win}胜 ${counts.draw}平 ${counts.loss}负`
}
