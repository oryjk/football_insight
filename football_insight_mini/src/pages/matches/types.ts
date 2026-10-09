import type { MatchCard, RoundReference } from '../../types/insight'

export interface UpcomingSection {
  title: string
  roundNumber: number
  matches: MatchCard[]
}

export interface MatchRoundGroup {
  roundNumber: number
  items: MatchCard[]
}

export interface SeasonProgressRow {
  rounds: RoundReference[]
  fillWidth: string
}
