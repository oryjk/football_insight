import type { TicketWatchState } from './useTicketWatchState'

export async function syncVisibleHistorySelection(
  state: TicketWatchState,
  selectMatch: (matchId: number) => Promise<void>,
): Promise<void> {
  if (
    !state.isPageVisible.value
    || state.activeTab.value !== 'history'
    || !state.hasLoadedHistoryBoard.value
    || state.historyLoading.value
    || state.historyTrendLoading.value
    || state.systemConfigUnderReview.value
    || state.membershipBenefitsLocked.value
  ) return

  const matches = state.visibleHistoryMatches.value
  if (!matches.length || matches.some(match => match.match_id === state.selectedHistoryMatchId.value)) return

  await selectMatch(matches[0].match_id)
}
