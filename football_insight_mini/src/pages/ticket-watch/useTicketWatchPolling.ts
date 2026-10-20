import { watch, onScopeDispose } from 'vue'

import { useAppSession, type AppSession } from '../../stores/appSession'

import type { TicketWatchLoadReason } from './helpers'
import type { TicketWatchState } from './useTicketWatchState'

export function useTicketWatchPolling(
  state: TicketWatchState,
  loadCurrentBoard: (reason: TicketWatchLoadReason) => Promise<void>,
  session: AppSession = useAppSession(),
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
        title: state.currentUser.value?.official_wechat_binding_required === true
          ? '请先绑定公众号后使用会员功能'
          : '当前账号已取关公众号，会员权益已暂停',
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
  // 账号切换/退出：立即停止旧账号轮询；在途的旧请求完成后也不会重启
  // （startPolling 以 isMonitoringActive 为门槛，私有数据由页面 revision watch 清理）。
  watch(session.revision, () => {
    handleStopMonitoring()
    stopPolling()
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
