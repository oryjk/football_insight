<template>
  <page-meta :page-style="pageStyle" />
  <FiSecondaryNav @back="goBackFromSecondaryPage('/pages/insights/index')" @share="copyPageShareLink('/pages/ticket-watch/index')" />
  <view v-if="pageEntered" class="page-root">
    <FiPageBackdrop :image-url="pageBackgroundImageUrl" :fallback-image-url="phoenixStadiumBgImage" full-page />
    <view class="page" :class="{ 'board-content--history-stats': activeTab === 'history-stats' }">
      <template v-for="block in visibleBlocks" :key="block.key">
        <FiLayoutBlock v-if="block.key === 'header'" :background-image-url="block.background_image_url">
          <TicketWatchHeader :active-tab="activeTab" @tab="activeTab = $event" />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'current_match' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'current' && !currentLoading && !currentErrorMessage"
          :background-image-url="block.background_image_url"
        >
          <TicketCurrentMatchCard
            v-if="currentMatch"
            :match="currentMatch"
            :available-regions="currentAvailableRegionCount"
            :total-occurrences="currentTotalOccurrences"
            :monitoring="isMonitoringActive"
            :refreshing="currentBoardRequestInFlight"
            :poll-failing="currentBoardPollFailing"
            :interval-seconds="pollIntervalSeconds"
            :last-updated-at-ms="currentBoardLoadedAtMs"
            :ticking="isPageVisible"
            :subscribed="refluxSubscriptionSubscribed"
            :message="currentMessage"
            @monitor="toggleMonitoring"
            @subscribe="openRefluxSubscriptionSheet"
            @match-id="openMatchIdSheet"
          />
          <view v-else class="state-card state-card--empty"><text>{{ currentMessage }}</text></view>
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'history_selector' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'history' && !historyLoading && !historyErrorMessage"
          :background-image-url="block.background_image_url"
        >
          <view v-if="!visibleHistoryMatches.length" class="state-card">
            <text>{{ historyTrendLoading ? '正在加载历史回流数据…' : '暂无有回流数据的历史比赛。' }}</text>
          </view>
          <TicketHistorySelector v-else :matches="visibleHistoryMatches" :selected-match-id="selectedHistoryMatchId" @select="handleHistoryMatchSelect" />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'history_match' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'history' && !historyLoading && !historyErrorMessage && displayedHistoryMatch"
          :background-image-url="block.background_image_url"
        >
          <TicketHistoryMatchCard :match="displayedHistoryMatch" :switching="historySelectionLoading" :total-occurrences="historyTotalOccurrences" />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'focus_panel' && !systemConfigUnderReview && !membershipBenefitsLocked && ((activeTab === 'current' && !currentLoading && !currentErrorMessage && currentMatch && (currentFocusBlocks.length || currentInterestFocusBlocks.length)) || (activeTab === 'history' && !historyLoading && !historyErrorMessage && displayedHistoryMatch && (historyFocusBlocks.length || historyInterestFocusBlocks.length)))"
          :background-image-url="block.background_image_url"
        >
          <TicketFocusPanel
            v-if="activeTab === 'current'"
            :focus-blocks="currentFocusBlocks"
            :interest-blocks="currentInterestFocusBlocks"
            :collapsed="isCurrentSectionCollapsed('focus')"
            @toggle="toggleCurrentSectionCollapsed('focus')"
          />
          <TicketFocusPanel
            v-else
            :focus-blocks="historyFocusBlocks"
            :interest-blocks="historyInterestFocusBlocks"
            :collapsed="isHistorySectionCollapsed('focus')"
            history
            :switching="historySelectionLoading"
            @toggle="toggleHistorySectionCollapsed('focus')"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'tracked_interests' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'current' && !currentLoading && !currentErrorMessage && currentMatch && currentUser"
          :background-image-url="block.background_image_url"
        >
          <TicketTrackedInterests
            :interests="currentTrackedInterests"
            :summary="currentTrackedInterestSummary"
            :collapsed="isCurrentSectionCollapsed('tracked-interests')"
            @toggle="toggleCurrentSectionCollapsed('tracked-interests')"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'recent_reflux' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'current' && !currentLoading && !currentErrorMessage && currentMatch && currentRecentRefluxPanelMode !== 'hidden'"
          :background-image-url="block.background_image_url"
        >
          <TicketRecentRefluxPanel
            :mode="currentRecentRefluxPanelMode"
            :buckets="currentRecentRefluxBuckets"
            :membership-tier="viewerMembershipTier"
            :invite-hint="inviteUpgradeHint"
            @upgrade="handleRecentRefluxUpgrade"
            @invite="handleInviteUpgrade"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'best_windows' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'current' && !currentLoading && !currentErrorMessage && (bestWindowsLoading || bestWindowsData)"
          :background-image-url="block.background_image_url"
        >
          <TicketBestWindowsPanel
            :data="bestWindowsData"
            :loading="bestWindowsLoading"
            :visible="bestWindowsVisible"
            :invite-hint="inviteUpgradeHint"
            @upgrade="handleBestWindowsUpgrade"
            @invite="handleInviteUpgrade"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'inventory_stack' && !systemConfigUnderReview && !membershipBenefitsLocked && ((activeTab === 'current' && !currentLoading && !currentErrorMessage && currentMatch && prioritizedCurrentSections.length) || (activeTab === 'history' && !historyLoading && !historyErrorMessage && displayedHistoryMatch && prioritizedHistorySections.length))"
          :background-image-url="block.background_image_url"
        >
          <view v-if="activeTab === 'current' && currentMatch" class="inventory-stack">
            <TicketInventoryPanel
              v-for="section in prioritizedCurrentSections"
              :key="section.price"
              :section="section"
              :match-id="currentMatch.match_id"
              :busy-blocks="interestToggleLoading"
              :collapsed="isCurrentSectionCollapsed(`inventory:${section.price}`)"
              @toggle="toggleCurrentSectionCollapsed(`inventory:${section.price}`)"
              @interest="handleCurrentInterest"
            />
          </view>
          <view v-else-if="displayedHistoryMatch" class="inventory-stack">
            <TicketInventoryPanel
              v-for="section in prioritizedHistorySections"
              :key="section.price"
              :section="section"
              :match-id="displayedHistoryMatch.match_id"
              readonly
              :collapsed="isHistorySectionCollapsed(`inventory:${section.price}`)"
              @toggle="toggleHistorySectionCollapsed(`inventory:${section.price}`)"
            />
          </view>
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'supply_insights' && !systemConfigUnderReview && !membershipBenefitsLocked && ((activeTab === 'current' && !currentLoading && !currentErrorMessage && currentMatch) || (activeTab === 'history' && !historyLoading && !historyErrorMessage && displayedHistoryMatch))"
          :background-image-url="block.background_image_url"
        >
          <TicketSupplyInsights
            v-if="activeTab === 'current'"
            :stats="currentBoardStats"
            :decision-lines="currentDecisionLines"
            :insight-collapsed="isCurrentSectionCollapsed('insight')"
            :decision-collapsed="isCurrentSectionCollapsed('decision')"
            @toggle="toggleCurrentSectionCollapsed"
          />
          <TicketSupplyInsights
            v-else
            :stats="historyBoardStats"
            :decision-lines="historyDecisionLines"
            history
            :insight-collapsed="isHistorySectionCollapsed('insight')"
            :decision-collapsed="isHistorySectionCollapsed('decision')"
            @toggle="toggleHistorySectionCollapsed"
          />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'history_trend' && !systemConfigUnderReview && !membershipBenefitsLocked && activeTab === 'history-stats' && !historyLoading && !historyErrorMessage"
          :background-image-url="block.background_image_url"
        >
          <view v-if="!historyMatches.length" class="state-card"><text>当前没有历史比赛。</text></view>
          <TicketHistoryTrend v-else :loading="historyTrendLoading" :trend="historyRefluxTrend" @select="handleHistoryMatchSelect" />
        </FiLayoutBlock>

        <FiLayoutBlock
          v-else-if="block.key === 'share_poster' && posterVisible"
          :background-image-url="block.background_image_url"
        >
          <TicketSharePosterPanel
            :image-path="posterImagePath"
            :generating="posterGenerating"
            :loading="posterLoading"
            :error="posterError"
            @generate="generatePoster"
            @preview="previewPoster"
            @save="savePoster"
          />
        </FiLayoutBlock>
      </template>

      <view v-if="systemConfigUnderReview" class="state-card"><text>当前版本展示基础内容。</text></view>
      <TicketMembershipLock
        v-else-if="membershipBenefitsLocked"
        :binding-required="officialBindingRequired"
        @login="officialBindingRequired ? officialBind.open() : goToUserPage()"
      />
      <template v-else-if="activeTab === 'current'">
        <TicketBoardSkeleton v-if="currentLoading" mode="current" :show-tabs="false" />
        <view v-else-if="currentErrorMessage" class="state-card state-card--error"><text>{{ currentErrorMessage }}</text></view>
      </template>
      <template v-else-if="activeTab === 'history'">
        <TicketBoardSkeleton v-if="historyLoading" mode="history" />
        <view v-else-if="historyErrorMessage" class="state-card state-card--error"><text>{{ historyErrorMessage }}</text></view>
      </template>
      <template v-else-if="activeTab === 'history-stats'">
        <TicketBoardSkeleton v-if="historyLoading" mode="history-stats" />
        <view v-else-if="historyErrorMessage" class="state-card state-card--error"><text>{{ historyErrorMessage }}</text></view>
      </template>
    </view>
    <canvas canvas-id="ticketWatchSharePoster" class="share-poster-canvas" :style="posterCanvasStyle" />
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
      :match-id="matchIdDisplayValue"
      :match-label="matchIdMatchLabel"
      :state="matchIdSheetState"
      :via="matchIdEntitlement?.via ?? null"
      :watermark="currentUser?.display_name?.trim() || ''"
      @close="closeMatchIdSheet"
      @pay="payForMatchId"
      @upgrade="handleMatchIdSheetUpgrade"
    />
    <FiOfficialBindSheet
      :visible="officialBind.visible.value"
      :code="officialBind.code.value"
      :remaining-seconds="officialBind.remainingSeconds.value"
      :loading="officialBind.loading.value"
      :error="officialBind.error.value"
      @close="officialBind.close"
      @get-code="officialBind.getCode"
      @copy="officialBind.copyCode"
      @open-official="openUserOfficialAccount"
    />
  </view>
  <view v-else class="page page--entry-mask" />
</template>

<script setup lang="ts">
import FiOfficialBindSheet from '../../components/FiOfficialBindSheet.vue'
import { createOfficialBindingGate } from '../../composables/officialBindingGate'
import { openUserOfficialAccount } from '../../utils/officialAccountNavigation'

import { buildSharePath, buildShareQuery } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { usePageLayout } from '../../composables/usePageLayout'
import { computed, nextTick, ref, watch } from 'vue'
import FiLayoutBlock from '../../components/FiLayoutBlock.vue'
import FiPageBackdrop from '../../components/FiPageBackdrop.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import {
  onHide,
  onReady,
  onShareAppMessage,
  onShareTimeline,
  onShow,
  onUnload,
} from '@dcloudio/uni-app'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import { formatInviteUpgradeHint } from '../../utils/membershipRules'
import { useRuntimeConfig } from '../../stores/runtimeConfig'
import { beginPaywallVisit, reportPaywallImpression, reportPaywallTap } from '../../composables/usePaywallTelemetry'
import { useAppSession } from '../../stores/appSession'
import type { TicketWatchGroupedInventoryItem } from '../../types/ticketWatch'
import TicketBoardSkeleton from './components/TicketBoardSkeleton.vue'
import TicketBestWindowsPanel from './components/TicketBestWindowsPanel.vue'
import { useTicketWatchBestWindows } from './useTicketWatchBestWindows'
import { resolveBestWindowsData } from './bestWindowsHelpers'
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
import TicketSharePosterPanel from './components/TicketSharePosterPanel.vue'
import { useTicketWatchSharePoster } from './useTicketWatchSharePoster'
import { TICKET_POSTER } from './sharePoster'
import { useTicketWatchState } from './useTicketWatchState'
import { useTicketWatchBoards } from './useTicketWatchBoards'
import { useTicketWatchInterests } from './useTicketWatchInterests'
import { useTicketWatchPolling } from './useTicketWatchPolling'
import { useTicketWatchSubscription } from './useTicketWatchSubscription'
import { useTicketWatchMatchId } from './useTicketWatchMatchId'
import { createTicketWatchSessionSync } from './sessionSync'
import { syncVisibleHistorySelection } from './ticketWatchHistorySelection'

const { navigationPageStyle } = useNavigationLayout()
const { visibleBlocks, pageBackgroundImageUrl, refreshLayout } = usePageLayout('ticket_watch')

const state = useTicketWatchState()
const {
  imagePath: posterImagePath, generating: posterGenerating, error: posterError,
  visible: posterVisible, loading: posterLoading, generate: generatePoster,
  preview: previewPoster, save: savePoster,
} = useTicketWatchSharePoster(state, goToUserPage)
// Bitmap geometry is in pixels, matching the canvas drawing/export coordinates.
const posterCanvasStyle = { width: `${TICKET_POSTER.width}px`, height: `${TICKET_POSTER.height}px` }
const {
  activeTab,
  currentLoading,
  currentBoardLoadedAtMs,
  currentBoardPollFailing,
  currentBoardRequestInFlight,
  isPageVisible,
  pollIntervalSeconds,
  historyLoading,
  currentErrorMessage,
  historyErrorMessage,
  currentMessage,
  regions,
  currentMatch,
  historyMatches,
  visibleHistoryMatches,
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
  matchIdDisplayValue,
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
  membershipBenefitsLocked,
  isCurrentSectionCollapsed,
  isHistorySectionCollapsed,
  toggleCurrentSectionCollapsed,
  toggleHistorySectionCollapsed,
} = state
const pageStyle = computed(() => `${navigationPageStyle.value};${matchIdSheetVisible.value || officialBind.visible.value ? 'overflow: hidden;' : ''}`)
const session = useAppSession()
const bindingStateReady = ref(false)
const officialBind = createOfficialBindingGate(session, systemConfigUnderReview, bindingStateReady)
const officialBindingRequired = computed(() => currentUser.value?.official_wechat_binding_required === true)
const bestWindows = useTicketWatchBestWindows({
  revision: session.revision,
  underReview: systemConfigUnderReview,
})
const { data: bestWindowsRawData, loading: bestWindowsLoading } = bestWindows
const bestWindowsData = computed(() => resolveBestWindowsData(
  bestWindowsRawData.value, membershipBenefitsLocked.value, systemConfigUnderReview.value,
))
const bestWindowsVisible = computed(() =>
  isPageVisible.value && pageEntered.value && activeTab.value === 'current'
  && !systemConfigUnderReview.value && !membershipBenefitsLocked.value
  && !currentLoading.value && !currentErrorMessage.value
  && !bestWindowsLoading.value && bestWindowsData.value !== null,
)
const inviteUpgradeHint = computed(() =>
  formatInviteUpgradeHint(useRuntimeConfig().config.value?.membership_tier_rules, 'V6'),
)
function handleBestWindowsUpgrade(): void {
  if (!bestWindowsVisible.value || bestWindowsData.value?.access !== 'preview') return
  reportPaywallTap('ticket_watch_best_window_lock', { matchId: currentMatch.value?.match_id })
  goToMembershipPurchase()
}
watch([bestWindowsVisible, bestWindowsData], () => {
  if (bestWindowsVisible.value && bestWindowsData.value?.access === 'preview') {
    reportPaywallImpression('ticket_watch_best_window_lock', { matchId: currentMatch.value?.match_id })
  }
})
const {
  refreshRefluxSubscriptionStatus,
  openRefluxSubscriptionSheet,
  closeRefluxSubscriptionSheet,
  submitRefluxSubscriptionOrder,
} = useTicketWatchSubscription(state, goToUserPage, undefined, session)
const { closeMatchIdSheet, openMatchIdSheet, payForMatchId, resetMatchIdSessionState } =
  useTicketWatchMatchId(state, goToUserPage, undefined, session)
const {
  loadCurrentBoard,
  loadHistoryMatches,
  loadBoardForTab,
  loadViewerMembershipTier,
  loadCurrentTrackedInterests,
  resetCurrentBoard,
  resetHistoryBoard,
  handleHistoryMatchSelect,
} = useTicketWatchBoards(state, refreshRefluxSubscriptionStatus, undefined, session)
const {
  handleBlockInterestToggle,
  closeInterestConfirm,
  confirmPendingInterestSelection,
} = useTicketWatchInterests(state, loadCurrentTrackedInterests, session)
const {
  startPolling,
  stopPolling,
  startFreshnessClock,
  stopFreshnessClock,
  toggleMonitoring,
} = useTicketWatchPolling(state, loadCurrentBoard, session)

// 页面可见身份：onShow 递增并置可见，onHide/onUnload 失效。
// onShow 的 await 链与 sessionSync 恢复链据此决定 hide/unload 后是否继续后续读取；
// pageEntered 在 onHide 时保持 true（仅控制首屏渲染遮罩），不能当作可见身份。
let pageShowToken = 0
let pageVisible = false
function isTicketWatchPageVisible(): boolean {
  return pageVisible
}

// 账号切换/退出：立即清理旧账号的私有看板数据（兴趣/订阅/会员档位），
// 轮询停止由 useTicketWatchPolling 的 revision watch 负责；公共数据下次 onShow 以新会话重载。
// 后台静默恢复/登录发布已认证用户后，可见页面按当前 activeTab 重载被清空的看板，无需再次切 Tab。
const sessionSync = createTicketWatchSessionSync(state, session, {
  resetSessionScopedBoards: () => {
    resetCurrentBoard()
    resetHistoryBoard()
    currentUser.value = null
    viewerMembershipTier.value = 'V1'
    refluxSubscriptionSheetVisible.value = false
    refluxSubscriptionLoading.value = false
    refluxSubscriptionSubmitting.value = false
    refluxSubscriptionStatusLoading.value = false
    pendingInterestSelection.value = null
    // 解锁 sheet 状态与私有权益一并重置（旧下单/支付的 finally 被 revision 防护跳过）。
    resetMatchIdSessionState()
  },
  loadViewerMembershipTier,
  loadCurrentBoard,
  loadHistoryMatches,
  isPageVisible: isTicketWatchPageVisible,
})

watch([session.revision, session.currentUser], () => {
  sessionSync.handleSessionRevisionChange()
})

// 页内 tab 切换不会触发 onShow：会话恢复只重载可见 tab，另一块已被清空的
// 看板靠这里在进入时按当前会话补载；已加载的看板复用，隐藏页面不发起读取。
watch(activeTab, (tab) => {
  if (!isTicketWatchPageVisible()) {
    return
  }
  loadBoardForTab(tab)
})

watch([activeTab, visibleHistoryMatches, historyLoading, historyTrendLoading, isPageVisible], () => {
  void syncVisibleHistorySelection(state, handleHistoryMatchSelect)
})

function handleCurrentInterest(item: TicketWatchGroupedInventoryItem): void {
  if (currentMatch.value)
    void handleBlockInterestToggle(currentMatch.value, item, 'current')
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

// 邀请升级：邀请码在「我的」页，锁定卡片直接带用户过去。
function handleInviteUpgrade(): void {
  if (systemConfigUnderReview.value) return
  goToUserPage()
}

// 付费引导度量：只在原有 goToMembershipPurchase 之前补记点击，原有逻辑不变。
function handleRecentRefluxUpgrade(): void {
  if (!systemConfigUnderReview.value) {
    reportPaywallTap('ticket_watch_recent_reflux_lock', { matchId: currentMatch.value?.match_id })
  }
  goToMembershipPurchase()
}

function handleMatchIdSheetUpgrade(): void {
  if (!systemConfigUnderReview.value) {
    reportPaywallTap('ticket_watch_match_id_sheet', { matchId: currentMatch.value?.match_id })
  }
  goToMembershipPurchase()
}

function reportRecentRefluxLockImpression(): void {
  if (systemConfigUnderReview.value || !currentMatch.value || currentRecentRefluxPanelMode.value !== 'locked') return
  reportPaywallImpression('ticket_watch_recent_reflux_lock', { matchId: currentMatch.value.match_id })
}

watch([currentRecentRefluxPanelMode, currentMatch], reportRecentRefluxLockImpression)
watch([matchIdSheetVisible, matchIdSheetState], ([visible, sheetState]) => {
  if (visible && sheetState === 'locked' && !systemConfigUnderReview.value) {
    reportPaywallImpression('ticket_watch_match_id_sheet', { matchId: currentMatch.value?.match_id })
  }
})

function buildTicketSharePayload() {
  if (activeTab.value === 'history' && displayedHistoryMatch.value) {
    return {
      title: `${displayedHistoryMatch.value.home_team_name} VS ${displayedHistoryMatch.value.away_team_name} 回流复盘`,
      path: buildSharePath('/pages/ticket-watch/index', 'ticket_history'),
    }
  }

  if (currentMatch.value) {
    return {
      title: `${currentMatch.value.home_team_name} VS ${currentMatch.value.away_team_name} 回流看板`,
      path: buildSharePath('/pages/ticket-watch/index', 'ticket_current'),
    }
  }

  return {
    title: '回流看板：当前比赛盯实时，历史比赛看回流',
    path: buildSharePath('/pages/ticket-watch/index', 'ticket_current'),
  }
}

onShareAppMessage(buildTicketSharePayload)
onShareTimeline(() => ({
  title: buildTicketSharePayload().title,
  query: buildShareQuery(activeTab.value === 'history' && displayedHistoryMatch.value
    ? 'ticket_history' : 'ticket_current'),
}))
onShow(() => {
  bindingStateReady.value = false
  void officialBind.handlePageShow()
  reportPageActivity('ticket_watch')
  void refreshLayout()
  beginPaywallVisit()
  reportRecentRefluxLockImpression()
  pageVisible = true
  isPageVisible.value = true
  const showToken = ++pageShowToken
  void (async () => {
    // 捕获本次 onShow 的可见身份：hide/unload/再次 onShow 后不再继续后续读取。
    const isCurrentShow = () => pageVisible && showToken === pageShowToken
    pageEntered.value = true
    startFreshnessClock()
    systemConfigUnderReview.value = await loadSystemConfigUnderReview()
    if (!isCurrentShow()) {
      return
    }
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
    if (!isCurrentShow()) {
      return
    }
    bindingStateReady.value = true
    void bestWindows.show()
    if (membershipBenefitsLocked.value) {
      stopPolling()
      return
    }
    await Promise.all([loadCurrentBoard('initial'), loadHistoryMatches()])
    if (!isCurrentShow()) {
      return
    }
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
  officialBind.handlePageHide()
  bestWindows.hide()
  pageVisible = false
  isPageVisible.value = false
  stopPolling()
  stopFreshnessClock()
})

onUnload(() => {
  officialBind.dispose()
  bestWindows.hide()
  pageVisible = false
  isPageVisible.value = false
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

.page {
  position: relative;
  z-index: 1;
  min-height: 100vh;
  padding: calc(var(--fi-brand-nav-height) + var(--fi-space-20)) var(--fi-space-20) var(--fi-space-36);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
}

.page--entry-mask {
  padding: 0;
}

.inventory-stack {
  display: grid;
  gap: var(--fi-space-card-gap);
}

.board-content {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: var(--fi-space-card-gap);
}

.share-poster-canvas { position: fixed; left: -10000rpx; top: -10000rpx; pointer-events: none; }

.board-content--history-stats {
  /* Remaining viewport after page padding, the segmented track and their gap. */
  --fi-ticket-history-panel-height: calc(
    100vh - var(--fi-brand-nav-height) - var(--fi-space-20) - var(--fi-space-36)
    - var(--fi-space-card-gap) - var(--fi-space-64)
    - var(--fi-space-6) * 2 - var(--fi-border-width) * 2
  );
}
</style>
