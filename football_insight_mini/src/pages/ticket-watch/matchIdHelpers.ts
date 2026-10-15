export type TicketMatchIdSheetState = 'locked' | 'unlocked'

export function resolveMatchIdSheetState(
  entitlement: { unlocked: boolean } | null | undefined,
): TicketMatchIdSheetState {
  return entitlement?.unlocked ? 'unlocked' : 'locked'
}

export function buildMatchIdSourceLabel(via: 'membership' | 'purchase' | null | undefined): string {
  if (via === 'membership') {
    return '会员权益'
  }

  if (via === 'purchase') {
    return '单场解锁'
  }

  return ''
}

export function buildTicketWatchMatchLabel(
  match: {
    round_number: number
    home_team_name: string
    away_team_name: string
    match_date: string
  } | null | undefined,
): string {
  if (!match) {
    return ''
  }

  return `第${match.round_number}轮 ${match.home_team_name} VS ${match.away_team_name} · ${match.match_date}`
}
