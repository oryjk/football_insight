import { computed, getCurrentInstance, nextTick, onScopeDispose, watch } from 'vue'
import { getTicketWatchShareCode } from '../../api/shareCodes'
import { exportCanvasToTempFile, flushCanvas, materializePosterCode, savePosterWithPermission } from '../../utils/canvasPoster'
import type { TicketWatchState } from './useTicketWatchState'
import { TICKET_POSTER, buildTicketPosterContent } from './sharePoster'
import { drawTicketPoster } from './sharePosterDrawing'
import { createTicketPosterFlow, type TicketPosterDependencies } from './ticketPosterFlow'

export function useTicketWatchSharePoster(state: TicketWatchState, goToLogin: () => void, dependencies: Partial<TicketPosterDependencies> = {}) {
  const instance = getCurrentInstance()
  const content = computed(() => {
    if (state.systemConfigUnderReview.value || state.membershipBenefitsLocked.value) return null
    const tab = state.activeTab.value
    if (tab !== 'current' && tab !== 'history') return null
    if (tab === 'current' ? state.currentErrorMessage.value : state.historyErrorMessage.value) return null
    if (tab === 'history' && state.selectedHistoryMatchId.value !== state.displayedHistoryMatch.value?.match_id) return null
    return buildTicketPosterContent({
      mode: tab,
      match: tab === 'current' ? state.currentMatch.value : state.displayedHistoryMatch.value,
      sections: tab === 'current' ? state.currentSections.value : state.historySections.value,
      loadedAtMs: tab === 'current' ? state.currentBoardLoadedAtMs.value : null,
    })
  })
  const loading = computed(() => state.activeTab.value === 'current'
    ? state.currentLoading.value || state.currentBoardRequestInFlight.value
    : state.historyLoading.value || state.historySelectionLoading.value || state.historyBoardRequestInFlight.value)
  const flow = createTicketPosterFlow({
    getCode: dependencies.getCode ?? getTicketWatchShareCode,
    render: dependencies.render ?? (async (snapshot, code, isCurrent) => {
      await nextTick()
      if (!isCurrent()) return ''
      const image = await materializePosterCode(code)
      try {
        const info = await new Promise<UniApp.GetImageInfoSuccessData>((resolve, reject) => uni.getImageInfo({ src: image.path, success: resolve, fail: reject }))
        if (!isCurrent()) return ''
        const context = uni.createCanvasContext(TICKET_POSTER.canvasId, instance)
        drawTicketPoster(context, snapshot, info.path)
        await flushCanvas(context)
        if (!isCurrent()) return ''
        const path = await exportCanvasToTempFile({ ...TICKET_POSTER, fileType: 'png', instance })
        return isCurrent() ? path : ''
      } finally { image.cleanup() }
    }),
  })
  // Same-match polling must not cancel a slow export or erase an already shared snapshot.
  // Only identity/access/validity changes invalidate it; generate captures fresh content once.
  const contextKey = computed(() => {
    const tab = state.activeTab.value
    const match = tab === 'current' ? state.currentMatch.value : state.displayedHistoryMatch.value
    return JSON.stringify([
      tab, state.selectedTeam.value, match?.match_id, match?.sale_start_at,
      tab === 'history' ? state.selectedHistoryMatchId.value : null,
      state.currentUser.value?.id, state.membershipBenefitsLocked.value,
      state.systemConfigUnderReview.value, content.value !== null,
    ])
  })
  watch(contextKey, flow.reset, { flush: 'sync' })
  // Native image preview may hide the page. Keep an already exported preview available
  // on return, but a hidden page must not publish a still-running generation.
  watch(state.isPageVisible, visible => { if (!visible && flow.generating.value) flow.reset() }, { flush: 'sync' })
  onScopeDispose(flow.reset)
  async function generate(): Promise<void> {
    if (!state.isPageVisible.value || !content.value || loading.value || flow.generating.value) return
    if (!state.currentUser.value) {
      uni.showToast({ title: '请先登录后生成分享图', icon: 'none' })
      goToLogin()
      return
    }
    await flow.generate(content.value)
  }
  function preview(): void {
    if (state.isPageVisible.value && flow.imagePath.value && content.value) uni.previewImage({ urls: [flow.imagePath.value] })
  }
  function save(): void {
    const path = flow.imagePath.value
    if (state.isPageVisible.value && path && content.value && !loading.value) {
      void savePosterWithPermission(path, uni, () => state.isPageVisible.value && flow.imagePath.value === path && content.value !== null)
    }
  }
  return { ...flow, visible: computed(() => content.value !== null), loading, generate, preview, save }
}
