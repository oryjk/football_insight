import {
  getTicketWatchBlockInterests,
  toggleTicketWatchBlockInterest,
} from '../../api/ticketWatch'

import type {
  TicketWatchGroupedInventoryItem,
  TicketWatchMatchSummary,
} from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'

import {
  applyBlockInterestToSections,
  applyBlockInterestsToSections,
} from './helpers'
import type { TicketWatchState } from './useTicketWatchState'
import type { TicketWatchBoardMode } from './types'
import { resolveInventoryBlockKey } from './presentation'

export function useTicketWatchInterests(
  state: TicketWatchState,
  loadCurrentTrackedInterests: (matchId: number) => Promise<void>,
) {
  const {
    currentSections,
    historySections,
    historySectionsCache,
    interestToggleLoading,
    pendingInterestSelection,
  } = state
  async function refreshMatchInterests(
    matchId: number,
    mode: TicketWatchBoardMode,
  ): Promise<void> {
    const interests = await getTicketWatchBlockInterests(matchId)

    if (mode === 'current') {
      currentSections.value = applyBlockInterestsToSections(
        currentSections.value,
        interests,
      )
      return
    }

    const nextSections = applyBlockInterestsToSections(
      historySections.value,
      interests,
    )
    historySections.value = nextSections
    historySectionsCache.value = {
      ...historySectionsCache.value,
      [matchId]: nextSections,
    }
  }

  function buildInterestToggleKey(matchId: number, blockName: string): string {
    return `${matchId}:${blockName}`
  }

  function isInterestToggleLoading(
    matchId: number,
    blockName: string,
  ): boolean {
    return Boolean(
      interestToggleLoading.value[buildInterestToggleKey(matchId, blockName)],
    )
  }

  async function handleBlockInterestToggle(
    match: TicketWatchMatchSummary,
    item: TicketWatchGroupedInventoryItem,
    mode: TicketWatchBoardMode,
  ): Promise<void> {
    if (mode !== 'current') {
      return
    }

    if (!item.viewer_interested) {
      pendingInterestSelection.value = {
        match,
        item,
        mode,
        blockName: item.block_name,
      }
      return
    }

    await submitBlockInterestToggle(match, item, mode)
  }

  async function submitBlockInterestToggle(
    match: TicketWatchMatchSummary,
    item: TicketWatchGroupedInventoryItem,
    mode: TicketWatchBoardMode,
  ): Promise<void> {
    const toggleKey = buildInterestToggleKey(
      match.match_id,
      resolveInventoryBlockKey(item),
    )
    if (interestToggleLoading.value[toggleKey]) {
      return
    }

    interestToggleLoading.value = {
      ...interestToggleLoading.value,
      [toggleKey]: true,
    }

    try {
      const updatedInterest = await toggleTicketWatchBlockInterest(
        match.match_id,
        item.block_name,
      )

      if (mode === 'current') {
        currentSections.value = applyBlockInterestToSections(
          currentSections.value,
          updatedInterest,
        )
        await loadCurrentTrackedInterests(match.match_id)
      } else {
        const nextSections = applyBlockInterestToSections(
          historySections.value,
          updatedInterest,
        )
        historySections.value = nextSections
        historySectionsCache.value = {
          ...historySectionsCache.value,
          [match.match_id]: nextSections,
        }
      }

      await refreshMatchInterests(match.match_id, mode)

      uni.showToast({
        title: updatedInterest.viewer_interested
          ? `已标记想抢 ${item.block_name}`
          : `已取消 ${item.block_name}`,
        icon: 'none',
      })
    } catch (error) {
      uni.showToast({
        title: extractApiErrorMessage(error, '标记想抢区域失败，请稍后重试。'),
        icon: 'none',
      })
    } finally {
      const nextState = { ...interestToggleLoading.value }
      delete nextState[toggleKey]
      interestToggleLoading.value = nextState
    }
  }

  function closeInterestConfirm(): void {
    pendingInterestSelection.value = null
  }

  async function confirmPendingInterestSelection(): Promise<void> {
    const selection = pendingInterestSelection.value
    if (!selection) {
      return
    }

    pendingInterestSelection.value = null
    await submitBlockInterestToggle(
      selection.match,
      selection.item,
      selection.mode,
    )
  }

  return {
    isInterestToggleLoading,
    handleBlockInterestToggle,
    closeInterestConfirm,
    confirmPendingInterestSelection,
  }
}
