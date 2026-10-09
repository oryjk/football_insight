import type { MatchCard, RoundReference } from '../../types/insight'
import { buildCurrentRoundSectionTitle, resolveMatchDisplayStatus } from './helpers'
import type { MatchRoundGroup, SeasonProgressRow, UpcomingSection } from './types'

export function groupFinishedMatches(matches: MatchCard[], nowIso: string): MatchRoundGroup[] {
  const groups = new Map<number, MatchCard[]>()

  matches
    .filter((match) => resolveMatchDisplayStatus(match, nowIso) === 'finished')
    .forEach((match) => {
      const list = groups.get(match.round_number) ?? []
      list.push(match)
      groups.set(match.round_number, list)
    })

  return Array.from(groups.entries())
    .sort((left, right) => right[0] - left[0])
    .map(([roundNumber, items]) => ({
      roundNumber,
      items,
    }))
}

export function buildSeasonProgressRows(rounds: RoundReference[]): SeasonProgressRow[] {
  const perRow = 15
  const rows: { rounds: RoundReference[]; fillWidth: string }[] = []
  for (let i = 0; i < rounds.length; i += perRow) {
    const rowRounds = rounds.slice(i, i + perRow)
    const completedInRow = rowRounds.filter((r) => r.status === 'completed' || r.status === 'current').length
    const fillWidth = rowRounds.length > 0 ? `${(completedInRow / rowRounds.length) * 100}%` : '0%'
    rows.push({ rounds: rowRounds, fillWidth })
  }
  return rows
}

export function buildUpcomingSections(rounds: RoundReference[], matches: MatchCard[], nowIso: string): UpcomingSection[] {
  const matchesForRound = (roundNumber: number) => matches.filter((match) => match.round_number === roundNumber)
  const currentRound = rounds.find((round) => round.status === 'current') ?? null

  if (!currentRound) {
    const firstUpcoming = rounds.find((round) => round.status === 'upcoming') ?? null
    if (!firstUpcoming) {
      return []
    }

    return [
      {
        title: '下一轮',
        roundNumber: firstUpcoming.round_number,
        matches: matchesForRound(firstUpcoming.round_number).filter((match) => resolveMatchDisplayStatus(match, nowIso) !== 'finished'),
      },
    ].filter((section) => section.matches.length > 0)
  }

  const pendingCurrentRoundMatches = matchesForRound(currentRound.round_number)
    .filter((match) => resolveMatchDisplayStatus(match, nowIso) !== 'finished')
  if (pendingCurrentRoundMatches.length > 0) {
    return [
      {
        title: buildCurrentRoundSectionTitle(
          pendingCurrentRoundMatches.map((match) => ({ status: resolveMatchDisplayStatus(match, nowIso) })),
        ),
        roundNumber: currentRound.round_number,
        matches: pendingCurrentRoundMatches,
      },
    ]
  }

  const nextRound = rounds.find((round) => round.round_number > currentRound.round_number) ?? null
  if (!nextRound) {
    return []
  }

  return [
    {
      title: '下一轮',
      roundNumber: nextRound.round_number,
      matches: matchesForRound(nextRound.round_number).filter((match) => resolveMatchDisplayStatus(match, nowIso) !== 'finished'),
    },
  ].filter((section) => section.matches.length > 0)
}

export function mergeMatchLists(...lists: MatchCard[][]): MatchCard[] {
  const merged = new Map<number, MatchCard>()
  for (const list of lists) {
    for (const match of list) merged.set(match.match_id, match)
  }
  return Array.from(merged.values())
}
