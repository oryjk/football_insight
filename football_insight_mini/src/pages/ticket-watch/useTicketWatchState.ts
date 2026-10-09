import { computed, ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import type { MatchIdEntitlement } from '../../api/matchIdUnlock'
import type {
  TicketWatchGroupedInventorySection,
  TicketWatchMatchSummary,
  TicketWatchRegion,
  TicketWatchTrackedInterest,
  RefluxSubscriptionPlan,
} from '../../types/ticketWatch'
import { resolveMembershipBenefitsLocked } from '../../utils/membershipBenefits'
import { normalizeTicketWatchPollIntervalSeconds } from '../../utils/membershipRules'
import {
  buildHistoryRefluxTrend,
  buildRecentRefluxBuckets,
  buildTrackedInterestSummary,
  formatTicketWatchMembershipBadgeTier,
  isTicketWatchSectionCollapsed,
  prioritizeInventorySections,
  resolveRecentRefluxPanelMode,
  buildTicketWatchMatchLabel,
  selectRefluxStatsMatches,
  summarizeInventoryBoard,
  type TicketWatchCollapsedSectionState,
  toggleTicketWatchSectionCollapsed,
} from './helpers'
import { buildBoardDecisionLines } from './presentation'

import type {
  TicketWatchTab,
  TicketWatchTeam,
  PendingInterestSelection,
} from './types'

export function useTicketWatchState() {
  const activeTab = ref<TicketWatchTab>('current')
  const selectedTeam = ref<TicketWatchTeam>('chengdurongcheng')
  const currentLoading = ref(true)
  const historyLoading = ref(true)
  const currentErrorMessage = ref('')
  const historyErrorMessage = ref('')
  const currentMessage = ref('暂无当前比赛。')
  const regions = ref<TicketWatchRegion[]>([])
  const currentMatch = ref<TicketWatchMatchSummary | null>(null)
  const historyMatches = ref<TicketWatchMatchSummary[]>([])
  const selectedHistoryMatchId = ref<number | null>(null)
  const currentSections = ref<TicketWatchGroupedInventorySection[]>([])
  const historySections = ref<TicketWatchGroupedInventorySection[]>([])
  const historyTrendTotals = ref<Record<number, number>>({})
  const hasLoadedCurrentBoard = ref(false)
  const hasLoadedHistoryBoard = ref(false)
  const currentBoardRequestInFlight = ref(false)
  const historyBoardRequestInFlight = ref(false)
  const historySelectionLoading = ref(false)
  const historyTrendLoading = ref(false)
  const displayedHistoryMatchId = ref<number | null>(null)
  const historySectionsCache = ref<
    Record<number, TicketWatchGroupedInventorySection[]>
  >({})
  const pageEntered = ref(true)
  const interestToggleLoading = ref<Record<string, boolean>>({})
  const currentUser = ref<CurrentUser | null>(null)
  const viewerMembershipTier = ref('V1')
  const systemConfigUnderReview = ref(false)
  const pendingInterestSelection = ref<PendingInterestSelection | null>(null)
  const isMonitoringActive = ref(false)
  const refluxSubscriptionSheetVisible = ref(false)
  const refluxSubscriptionLoading = ref(false)
  const refluxSubscriptionSubmitting = ref(false)
  const refluxSubscriptionPlans = ref<RefluxSubscriptionPlan[]>([])
  const selectedRefluxSubscriptionPlanCode = ref('')
  const refluxSubscriptionEmail = ref('')
  const refluxSubscriptionSubscribed = ref(false)
  const refluxSubscriptionStatusLoading = ref(false)
  const matchIdSheetVisible = ref(false)
  const matchIdSheetState = ref<'loading' | 'locked' | 'unlocked' | 'paying'>(
    'loading',
  )
  const matchIdEntitlement = ref<MatchIdEntitlement | null>(null)
  // 微信支付仅小程序端可用；用运行时标记而非函数内 #ifdef 裸 return，
  // 避免 TS 把后续代码判为不可达而丢失类型收窄。
  const canRequestWxPayment = ref(false)
  // #ifdef MP-WEIXIN
  canRequestWxPayment.value = true
  // #endif
  const currentTrackedInterests = ref<TicketWatchTrackedInterest[]>([])
  const currentCollapsedSections = ref<TicketWatchCollapsedSectionState>({})
  const historyCollapsedSections = ref<TicketWatchCollapsedSectionState>({})
  const freshnessNowMs = ref(Date.now())

  const matchIdMatchLabel = computed(() =>
    buildTicketWatchMatchLabel(currentMatch.value),
  )

  const displayedHistoryMatch = computed(() => {
    if (displayedHistoryMatchId.value === null) {
      return null
    }

    return (
      historyMatches.value.find(
        (match) => match.match_id === displayedHistoryMatchId.value,
      ) ?? null
    )
  })

  const currentAvailableRegionCount = computed(() =>
    currentSections.value.reduce(
      (sum, section) => sum + section.available_region_count,
      0,
    ),
  )

  const currentTotalOccurrences = computed(() =>
    currentSections.value.reduce(
      (sum, section) => sum + section.total_occurrences,
      0,
    ),
  )

  const historyTotalOccurrences = computed(() =>
    historySections.value.reduce(
      (sum, section) => sum + section.total_occurrences,
      0,
    ),
  )

  const currentBoardStats = computed(() =>
    summarizeInventoryBoard(currentSections.value),
  )
  const historyBoardStats = computed(() =>
    summarizeInventoryBoard(historySections.value),
  )
  const prioritizedCurrentSections = computed(() =>
    prioritizeInventorySections(currentSections.value),
  )
  const prioritizedHistorySections = computed(() =>
    prioritizeInventorySections(historySections.value),
  )
  const currentFocusBlocks = computed(() =>
    currentBoardStats.value.topBlocks.slice(0, 3),
  )
  const historyFocusBlocks = computed(() =>
    historyBoardStats.value.topBlocks.slice(0, 3),
  )
  const currentInterestFocusBlocks = computed(() =>
    currentBoardStats.value.topInterestBlocks.slice(0, 3),
  )
  const historyInterestFocusBlocks = computed(() =>
    historyBoardStats.value.topInterestBlocks.slice(0, 3),
  )
  const currentRecentRefluxBuckets = computed(() =>
    buildRecentRefluxBuckets(
      currentSections.value,
      new Date(freshnessNowMs.value).toISOString(),
    ),
  )
  const currentHasRecentReflux = computed(() =>
    currentRecentRefluxBuckets.value.some((bucket) => bucket.items.length > 0),
  )
  const currentRecentRefluxPanelMode = computed(() =>
    resolveRecentRefluxPanelMode(
      currentHasRecentReflux.value,
      viewerMembershipTier.value,
    ),
  )
  const currentTrackedInterestSummary = computed(() =>
    buildTrackedInterestSummary(currentTrackedInterests.value),
  )
  const currentDecisionLines = computed(() =>
    buildBoardDecisionLines(currentBoardStats.value, 'current'),
  )
  const historyDecisionLines = computed(() =>
    buildBoardDecisionLines(historyBoardStats.value, 'history'),
  )
  const historyStatsMatches = computed(() =>
    selectRefluxStatsMatches(historyMatches.value),
  )
  const historyRefluxTrend = computed(() =>
    buildHistoryRefluxTrend(
      historyStatsMatches.value,
      historyTrendTotals.value,
      selectedHistoryMatchId.value,
      selectedTeam.value === 'yunnanyukun' ? '云南玉昆' : '成都蓉城',
    ),
  )
  const pollIntervalSeconds = computed(() =>
    normalizeTicketWatchPollIntervalSeconds(
      currentUser.value?.ticket_watch_poll_interval_seconds,
    ),
  )
  const membershipBadgeTier = computed(() =>
    formatTicketWatchMembershipBadgeTier(viewerMembershipTier.value),
  )
  const membershipExclusiveLabel = computed(
    () => `${membershipBadgeTier.value}专享`,
  )
  const membershipBenefitsLocked = computed(() =>
    resolveMembershipBenefitsLocked(currentUser.value),
  )

  function buildCurrentSectionCollapseKey(section: string): string {
    return `current:${currentMatch.value?.match_id ?? 0}:${section}`
  }

  function buildHistorySectionCollapseKey(section: string): string {
    return `history:${displayedHistoryMatch.value?.match_id ?? 0}:${section}`
  }

  function isCurrentSectionCollapsed(section: string): boolean {
    return isTicketWatchSectionCollapsed(
      currentCollapsedSections.value,
      buildCurrentSectionCollapseKey(section),
    )
  }

  function isHistorySectionCollapsed(section: string): boolean {
    return isTicketWatchSectionCollapsed(
      historyCollapsedSections.value,
      buildHistorySectionCollapseKey(section),
    )
  }

  function toggleCurrentSectionCollapsed(section: string): void {
    currentCollapsedSections.value = toggleTicketWatchSectionCollapsed(
      currentCollapsedSections.value,
      buildCurrentSectionCollapseKey(section),
    )
  }

  function toggleHistorySectionCollapsed(section: string): void {
    historyCollapsedSections.value = toggleTicketWatchSectionCollapsed(
      historyCollapsedSections.value,
      buildHistorySectionCollapseKey(section),
    )
  }

  return {
    activeTab,
    selectedTeam,
    currentLoading,
    historyLoading,
    currentErrorMessage,
    historyErrorMessage,
    currentMessage,
    regions,
    currentMatch,
    historyMatches,
    selectedHistoryMatchId,
    currentSections,
    historySections,
    historyTrendTotals,
    hasLoadedCurrentBoard,
    hasLoadedHistoryBoard,
    currentBoardRequestInFlight,
    historyBoardRequestInFlight,
    historySelectionLoading,
    historyTrendLoading,
    displayedHistoryMatchId,
    historySectionsCache,
    pageEntered,
    interestToggleLoading,
    currentUser,
    viewerMembershipTier,
    systemConfigUnderReview,
    pendingInterestSelection,
    isMonitoringActive,
    refluxSubscriptionSheetVisible,
    refluxSubscriptionLoading,
    refluxSubscriptionSubmitting,
    refluxSubscriptionPlans,
    selectedRefluxSubscriptionPlanCode,
    refluxSubscriptionEmail,
    refluxSubscriptionSubscribed,
    refluxSubscriptionStatusLoading,
    matchIdSheetVisible,
    matchIdSheetState,
    matchIdEntitlement,
    canRequestWxPayment,
    currentTrackedInterests,
    currentCollapsedSections,
    historyCollapsedSections,
    freshnessNowMs,
    matchIdMatchLabel,
    displayedHistoryMatch,
    currentAvailableRegionCount,
    currentTotalOccurrences,
    historyTotalOccurrences,
    currentBoardStats,
    historyBoardStats,
    prioritizedCurrentSections,
    prioritizedHistorySections,
    currentFocusBlocks,
    historyFocusBlocks,
    currentInterestFocusBlocks,
    historyInterestFocusBlocks,
    currentRecentRefluxBuckets,
    currentHasRecentReflux,
    currentRecentRefluxPanelMode,
    currentTrackedInterestSummary,
    currentDecisionLines,
    historyDecisionLines,
    historyStatsMatches,
    historyRefluxTrend,
    pollIntervalSeconds,
    membershipBadgeTier,
    membershipExclusiveLabel,
    membershipBenefitsLocked,
    isCurrentSectionCollapsed,
    isHistorySectionCollapsed,
    toggleCurrentSectionCollapsed,
    toggleHistorySectionCollapsed,
  }
}

export type TicketWatchState = ReturnType<typeof useTicketWatchState>
