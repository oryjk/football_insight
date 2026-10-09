<template>
  <view v-if="pageEntered" class="page-root">
    <image
      class="page-bg-img"
      :src="phoenixStadiumBgImage"
      mode="aspectFill"
      :webp="true"
    />
    <view class="page-bg-fade" />
    <view class="page">
      <TicketWatchHeader
        :selected-team="selectedTeam"
        :active-tab="activeTab"
        @team="switchTeam"
        @tab="activeTab = $event"
      />
      <view v-if="systemConfigUnderReview" class="state-card"
        ><text>当前版本展示基础内容。</text></view
      >
      <TicketMembershipLock
        v-else-if="membershipBenefitsLocked"
        @login="goToUserPage"
      />
      <view v-else-if="activeTab === 'current'" class="board-content">
        <TicketBoardSkeleton v-if="currentLoading" mode="current" />
        <view
          v-else-if="currentErrorMessage"
          class="state-card state-card--error"
          ><text>{{ currentErrorMessage }}</text></view
        >
        <template v-else>
          <TicketCurrentMatchCard
            v-if="currentMatch"
            :match="currentMatch"
            :available-regions="currentAvailableRegionCount"
            :total-occurrences="currentTotalOccurrences"
            :membership-label="membershipExclusiveLabel"
            :monitoring="isMonitoringActive"
            :subscribed="refluxSubscriptionSubscribed"
            :message="currentMessage"
            @monitor="toggleMonitoring"
            @subscribe="openRefluxSubscriptionSheet"
            @match-id="openMatchIdSheet"
          />
          <view v-else class="state-card state-card--empty"
            ><text>{{ currentMessage }}</text></view
          >
          <TicketRecentRefluxPanel
            v-if="currentMatch && currentRecentRefluxPanelMode !== 'hidden'"
            :mode="currentRecentRefluxPanelMode"
            :buckets="currentRecentRefluxBuckets"
            :membership-tier="viewerMembershipTier"
            @upgrade="goToMembershipPurchase"
          />
          <TicketFocusPanel
            v-if="
              currentMatch &&
              (currentFocusBlocks.length || currentInterestFocusBlocks.length)
            "
            :focus-blocks="currentFocusBlocks"
            :interest-blocks="currentInterestFocusBlocks"
            :collapsed="isCurrentSectionCollapsed('focus')"
            @toggle="toggleCurrentSectionCollapsed('focus')"
          />
          <TicketTrackedInterests
            v-if="currentMatch && currentUser"
            :interests="currentTrackedInterests"
            :summary="currentTrackedInterestSummary"
            :collapsed="isCurrentSectionCollapsed('tracked-interests')"
            @toggle="toggleCurrentSectionCollapsed('tracked-interests')"
          />
          <view
            v-if="currentMatch && prioritizedCurrentSections.length"
            class="inventory-stack"
          >
            <TicketInventoryPanel
              v-for="section in prioritizedCurrentSections"
              :key="section.price"
              :section="section"
              :match-id="currentMatch.match_id"
              :busy-blocks="interestToggleLoading"
              :collapsed="
                isCurrentSectionCollapsed(`inventory:${section.price}`)
              "
              @toggle="
                toggleCurrentSectionCollapsed(`inventory:${section.price}`)
              "
              @interest="handleCurrentInterest"
            />
          </view>
          <TicketSupplyInsights
            v-if="currentMatch"
            :stats="currentBoardStats"
            :decision-lines="currentDecisionLines"
            :insight-collapsed="isCurrentSectionCollapsed('insight')"
            :decision-collapsed="isCurrentSectionCollapsed('decision')"
            @toggle="toggleCurrentSectionCollapsed"
          />
        </template>
      </view>
      <view v-else-if="activeTab === 'history'" class="board-content">
        <TicketBoardSkeleton v-if="historyLoading" mode="history" />
        <view
          v-else-if="historyErrorMessage"
          class="state-card state-card--error"
          ><text>{{ historyErrorMessage }}</text></view
        >
        <template v-else>
          <TicketHistorySelector
            :matches="historyMatches"
            :selected-match-id="selectedHistoryMatchId"
            @select="handleHistoryMatchSelect"
          />
          <TicketHistoryMatchCard
            v-if="displayedHistoryMatch"
            :match="displayedHistoryMatch"
            :switching="historySelectionLoading"
            :total-occurrences="historyTotalOccurrences"
          />
          <TicketFocusPanel
            v-if="
              displayedHistoryMatch &&
              (historyFocusBlocks.length || historyInterestFocusBlocks.length)
            "
            :focus-blocks="historyFocusBlocks"
            :interest-blocks="historyInterestFocusBlocks"
            :collapsed="isHistorySectionCollapsed('focus')"
            history
            :switching="historySelectionLoading"
            @toggle="toggleHistorySectionCollapsed('focus')"
          />
          <view
            v-if="displayedHistoryMatch && prioritizedHistorySections.length"
            class="inventory-stack"
          >
            <TicketInventoryPanel
              v-for="section in prioritizedHistorySections"
              :key="section.price"
              :section="section"
              :match-id="displayedHistoryMatch.match_id"
              readonly
              :collapsed="
                isHistorySectionCollapsed(`inventory:${section.price}`)
              "
              @toggle="
                toggleHistorySectionCollapsed(`inventory:${section.price}`)
              "
            />
          </view>
          <TicketSupplyInsights
            v-if="displayedHistoryMatch"
            :stats="historyBoardStats"
            :decision-lines="historyDecisionLines"
            history
            :insight-collapsed="isHistorySectionCollapsed('insight')"
            :decision-collapsed="isHistorySectionCollapsed('decision')"
            @toggle="toggleHistorySectionCollapsed"
          />
        </template>
      </view>
      <view v-else-if="activeTab === 'history-stats'" class="board-content">
        <TicketBoardSkeleton v-if="historyLoading" mode="history-stats" />
        <view
          v-else-if="historyErrorMessage"
          class="state-card state-card--error"
          ><text>{{ historyErrorMessage }}</text></view
        >
        <view v-else-if="!historyMatches.length" class="state-card"
          ><text>当前没有历史比赛。</text></view
        >
        <TicketHistoryTrend
          v-else
          :loading="historyTrendLoading"
          :trend="historyRefluxTrend"
          @select="handleHistoryMatchSelect"
        />
      </view>
    </view>
    <TicketInterestConfirm
      v-if="pendingInterestSelection"
      :block-name="pendingInterestSelection.blockName"
      @close="closeInterestConfirm"
      @confirm="confirmPendingInterestSelection"
    />
    <TicketSubscriptionSheet
      v-else-if="refluxSubscriptionSheetVisible && currentMatch"
      :match="currentMatch"
      :subscribed="refluxSubscriptionSubscribed"
      :email="refluxSubscriptionEmail"
      :plans="refluxSubscriptionPlans"
      :selected-plan-code="selectedRefluxSubscriptionPlanCode"
      :submitting="refluxSubscriptionSubmitting"
      @close="closeRefluxSubscriptionSheet"
      @email="refluxSubscriptionEmail = $event"
      @plan="selectedRefluxSubscriptionPlanCode = $event"
      @submit="submitRefluxSubscriptionOrder"
    />
    <TicketMatchIdSheet
      :visible="matchIdSheetVisible"
      :match-id="currentMatch?.match_id ?? null"
      :match-label="matchIdMatchLabel"
      :state="matchIdSheetState"
      :via="matchIdEntitlement?.via ?? null"
      @close="closeMatchIdSheet"
      @pay="payForMatchId"
      @upgrade="goToMembershipPurchase"
    />
  </view>
  <view v-else class="page page--entry-mask" />
</template>

<script setup lang="ts">
import { nextTick } from 'vue'
import {
  onHide,
  onReady,
  onShareAppMessage,
  onShow,
  onUnload,
} from '@dcloudio/uni-app'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import type { TicketWatchGroupedInventoryItem } from '../../types/ticketWatch'
import type { TicketWatchTeam } from './types'
import TicketBoardSkeleton from './components/TicketBoardSkeleton.vue'
import TicketCurrentMatchCard from './components/TicketCurrentMatchCard.vue'
import TicketFocusPanel from './components/TicketFocusPanel.vue'
import TicketHistoryMatchCard from './components/TicketHistoryMatchCard.vue'
import TicketHistorySelector from './components/TicketHistorySelector.vue'
import TicketHistoryTrend from './components/TicketHistoryTrend.vue'
import TicketInterestConfirm from './components/TicketInterestConfirm.vue'
import TicketInventoryPanel from './components/TicketInventoryPanel.vue'
import TicketMatchIdSheet from './components/TicketMatchIdSheet.vue'
import TicketMembershipLock from './components/TicketMembershipLock.vue'
import TicketRecentRefluxPanel from './components/TicketRecentRefluxPanel.vue'
import TicketSubscriptionSheet from './components/TicketSubscriptionSheet.vue'
import TicketSupplyInsights from './components/TicketSupplyInsights.vue'
import TicketTrackedInterests from './components/TicketTrackedInterests.vue'
import TicketWatchHeader from './components/TicketWatchHeader.vue'
import { useTicketWatchState } from './useTicketWatchState'
import { useTicketWatchBoards } from './useTicketWatchBoards'
import { useTicketWatchInterests } from './useTicketWatchInterests'
import { useTicketWatchPolling } from './useTicketWatchPolling'
import { useTicketWatchSubscription } from './useTicketWatchSubscription'
import { useTicketWatchMatchId } from './useTicketWatchMatchId'

const state = useTicketWatchState()
const {
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
  historySelectionLoading,
  historyTrendLoading,
  pageEntered,
  interestToggleLoading,
  currentUser,
  viewerMembershipTier,
  systemConfigUnderReview,
  pendingInterestSelection,
  isMonitoringActive,
  refluxSubscriptionSheetVisible,
  refluxSubscriptionSubmitting,
  refluxSubscriptionPlans,
  selectedRefluxSubscriptionPlanCode,
  refluxSubscriptionEmail,
  refluxSubscriptionSubscribed,
  matchIdSheetVisible,
  matchIdSheetState,
  matchIdEntitlement,
  currentTrackedInterests,
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
  currentRecentRefluxPanelMode,
  currentTrackedInterestSummary,
  currentDecisionLines,
  historyDecisionLines,
  historyRefluxTrend,
  membershipExclusiveLabel,
  membershipBenefitsLocked,
  isCurrentSectionCollapsed,
  isHistorySectionCollapsed,
  toggleCurrentSectionCollapsed,
  toggleHistorySectionCollapsed,
} = state
const {
  refreshRefluxSubscriptionStatus,
  openRefluxSubscriptionSheet,
  closeRefluxSubscriptionSheet,
  submitRefluxSubscriptionOrder,
} = useTicketWatchSubscription(state, goToUserPage)
const { closeMatchIdSheet, openMatchIdSheet, payForMatchId } =
  useTicketWatchMatchId(state, goToUserPage)
const {
  loadCurrentBoard,
  loadHistoryMatches,
  loadViewerMembershipTier,
  loadCurrentTrackedInterests,
  resetCurrentBoard,
  resetHistoryBoard,
  handleHistoryMatchSelect,
} = useTicketWatchBoards(state, refreshRefluxSubscriptionStatus)
const {
  handleBlockInterestToggle,
  closeInterestConfirm,
  confirmPendingInterestSelection,
} = useTicketWatchInterests(state, loadCurrentTrackedInterests)
const {
  startPolling,
  stopPolling,
  startFreshnessClock,
  stopFreshnessClock,
  handleStopMonitoring,
  toggleMonitoring,
} = useTicketWatchPolling(state, loadCurrentBoard)

function handleCurrentInterest(item: TicketWatchGroupedInventoryItem): void {
  if (currentMatch.value)
    void handleBlockInterestToggle(currentMatch.value, item, 'current')
}

async function switchTeam(team: TicketWatchTeam): Promise<void> {
  if (selectedTeam.value === team) return
  selectedTeam.value = team
  handleStopMonitoring()
  resetCurrentBoard()
  resetHistoryBoard()
  await Promise.all([
    loadCurrentBoard('initial'),
    loadHistoryMatches('initial'),
  ])
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

function goToMembershipPurchase(): void {
  if (systemConfigUnderReview.value) {
    return
  }

  if (!currentUser.value?.has_wechat_binding) {
    goToUserPage()
    return
  }

  uni.navigateTo({ url: '/pages/membership-purchase/index' })
}

onShareAppMessage(() => {
  if (activeTab.value === 'history' && displayedHistoryMatch.value) {
    return {
      title: `${displayedHistoryMatch.value.home_team_name} VS ${displayedHistoryMatch.value.away_team_name} 回流复盘`,
      path: '/pages/ticket-watch/index',
    }
  }

  if (currentMatch.value) {
    return {
      title: `${currentMatch.value.home_team_name} VS ${currentMatch.value.away_team_name} 回流看板`,
      path: '/pages/ticket-watch/index',
    }
  }

  return {
    title: '回流看板：当前比赛盯实时，历史比赛看回流',
    path: '/pages/ticket-watch/index',
  }
})

onShow(() => {
  reportPageActivity('ticket_watch')
  void (async () => {
    pageEntered.value = true
    startFreshnessClock()
    systemConfigUnderReview.value = await loadSystemConfigUnderReview()
    if (systemConfigUnderReview.value) {
      stopPolling()
      currentLoading.value = false
      historyLoading.value = false
      currentUser.value = null
      viewerMembershipTier.value = 'V1'
      currentSections.value = []
      historySections.value = []
      return
    }
    await loadViewerMembershipTier()
    if (membershipBenefitsLocked.value) {
      stopPolling()
      return
    }
    await Promise.all([loadCurrentBoard('initial'), loadHistoryMatches()])
    if (isMonitoringActive.value) {
      startPolling()
    }
  })()
})

onReady(() => {
  void nextTick(() => {
    pageEntered.value = true
  })
})

onHide(() => {
  stopPolling()
  stopFreshnessClock()
})

onUnload(() => {
  stopPolling()
  stopFreshnessClock()
  pageEntered.value = false
})
</script>

<style scoped lang="css">
@import './ticket-watch-shared.css';
.page-root {
  position: relative;
  min-height: 100vh;
  background: var(--fi-color-page-soft);
}

.page-bg-img {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}

.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-page-bg-fade-background) 0%,
    var(--fi-color-ticket-page-bg-fade-background-2) 32%,
    var(--fi-color-page-soft) 58%,
    var(--fi-color-page-soft) 100%
  );
  pointer-events: none;
  z-index: 0;
}

.page {
  position: relative;
  z-index: 1;
  min-height: 100vh;
  padding: var(--fi-space-20) var(--fi-space-20) var(--fi-space-36);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
}

.page--entry-mask {
  padding: 0;
}

.inventory-stack {
  display: grid;
  gap: var(--fi-space-14);
}

.board-content {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: var(--fi-space-18);
}
</style>
