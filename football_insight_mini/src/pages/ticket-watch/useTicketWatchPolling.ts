import { watch, onScopeDispose } from 'vue'

import type { TicketWatchLoadReason } from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

export function useTicketWatchPolling(
  state: TicketWatchState,
  loadCurrentBoard: (reason: TicketWatchLoadReason) => Promise<void>,
) {
  const {
    activeTab,
    systemConfigUnderReview,
    isMonitoringActive,
    freshnessNowMs,
    pollIntervalSeconds,
    membershipBenefitsLocked,
  } = state
  let pollTimer: ReturnType<typeof setInterval> | null = null
  let freshnessTimer: ReturnType<typeof setInterval> | null = null
  let pageVisible = true
  function startPolling(): void {
    if (
      !pageVisible ||
      activeTab.value !== 'current' ||
      !isMonitoringActive.value
    ) {
      return
    }

    stopPolling()
    pollTimer = setInterval(() => {
      void loadCurrentBoard('poll')
    }, pollIntervalSeconds.value * 1000)
  }

  function stopPolling(): void {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  function startFreshnessClock(): void {
    stopFreshnessClock()
    pageVisible = true
    freshnessNowMs.value = Date.now()
    freshnessTimer = setInterval(() => {
      freshnessNowMs.value = Date.now()
    }, 15 * 1000)
  }

  function stopFreshnessClock(): void {
    pageVisible = false
    if (freshnessTimer) {
      clearInterval(freshnessTimer)
      freshnessTimer = null
    }
  }

  async function handleStartMonitoring(): Promise<void> {
    if (isMonitoringActive.value) {
      return
    }

    isMonitoringActive.value = true
    await loadCurrentBoard('poll')
    startPolling()
  }

  function handleStopMonitoring(): void {
    if (!isMonitoringActive.value) {
      return
    }

    isMonitoringActive.value = false
    stopPolling()
  }

  async function toggleMonitoring(): Promise<void> {
    if (systemConfigUnderReview.value) {
      return
    }

    if (membershipBenefitsLocked.value) {
      uni.showToast({
        title: '当前账号已取关公众号，会员权益已暂停',
        icon: 'none',
        duration: 2200,
      })
      return
    }

    if (isMonitoringActive.value) {
      handleStopMonitoring()
      return
    }

    await handleStartMonitoring()
  }

  watch(activeTab, (tab) => {
    if (tab === 'current' && isMonitoringActive.value) startPolling()
    else stopPolling()
  })
  watch(pollIntervalSeconds, () => {
    if (activeTab.value === 'current' && isMonitoringActive.value)
      startPolling()
  })
  onScopeDispose(() => {
    stopPolling()
    stopFreshnessClock()
  })

  return {
    startPolling,
    stopPolling,
    startFreshnessClock,
    stopFreshnessClock,
    handleStopMonitoring,
    toggleMonitoring,
  }
}
