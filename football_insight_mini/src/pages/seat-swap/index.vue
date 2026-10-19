<template>
  <page-meta :page-style="navigationPageStyle + pageLockStyle" />
  <view class="page-root">
    <FiBrandNav />
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page">
      <SeatSwapSkeleton v-if="loading" />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
        <button class="ghost-action" @tap="loadPage">重试</button>
      </view>

      <view v-else class="page-content">
        <view>
          <SeatSwapMatchCard
            :match-title="matchTitle"
            :match-summary="matchSummary"
            :candidates-count="candidatesCount"
          />
        </view>

        <SeatSwapNotice v-if="!currentView?.available" text="换座撮合只在成都蓉城当前比赛开放。" />
        <SeatSwapNotice
          v-else-if="!isLoggedIn"
          text="未登录可浏览脱敏意向;发布、确认和查看联系方式需要登录。"
          action-label="去登录"
          @action="goToUserPage"
        />

        <view class="seat-map-section">
          <SeatSwapRegionPanel
            :mode="mainMapMode"
            :regions="regions"
            :badges="regionBadgeCounts"
            :filter-key="browsingFilterKey"
            :current-key="currentView?.my_request?.current_region_key || ''"
            :desired-keys="myDesiredKeys"
            :has-my-request="!!currentView?.my_request"
            @region-tap="handleMainMapTap"
          />
        </view>

        <SeatSwapPoolList
          :groups="seatSwapRegionGroups"
          :expanded-keys="expandedRegionKeys"
          :filter-key="browsingFilterKey"
          :filter-name="browsingFilterName"
          :filtered-candidates="filteredCandidates"
          :total-count="candidatesCount"
          :my-desired-keys="myDesiredKeys"
          :is-logged-in="isLoggedIn"
          :my-request-id="currentView?.my_request?.request_id ?? null"
          @toggle-group="toggleRegionGroup"
          @clear-filter="clearFilter"
          @confirm="confirmCandidate"
          @cancel-confirmation="cancelCandidateConfirmation"
          @matched-cancel="openMatchedCancelForCandidate"
        />
      </view>
    </view>

    <SeatSwapDock
      v-if="!loading && !errorMessage && currentView?.available"
      :has-my-request="!!currentView?.my_request"
      :is-logged-in="isLoggedIn"
      :current-seat-label="myRequestSeatLabel"
      :desired-summary="myDesiredSummary"
      @cta="handleDockCtaTap"
      @manage="openManageSheet"
    />

    <SeatSwapPublishSheet
      v-model:visible="publishSheetVisible"
      :regions="regions"
      :my-request="currentView?.my_request ?? null"
      :preset-candidate="pendingConfirmTarget"
      :submitting="submitting"
      @submit="submitForm"
    />

    <SeatSwapManageSheet
      v-model:visible="manageSheetVisible"
      v-model:cancel-reason="cancelReason"
      :request="currentView?.my_request"
      :status-label="myRequestStatusLabel"
      :desired-summary="myDesiredSummary"
      @close="closeManageSheet"
      @edit="openEditFromManage"
      @delete="deleteRequest"
      @submit-matched-cancel="submitMatchedCancel"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import SeatSwapManageSheet from './components/SeatSwapManageSheet.vue'
import SeatSwapMatchCard from './components/SeatSwapMatchCard.vue'
import SeatSwapNotice from './components/SeatSwapNotice.vue'
import SeatSwapSkeleton from './components/SeatSwapSkeleton.vue'
import SeatSwapRegionPanel from './components/SeatSwapRegionPanel.vue'
import SeatSwapPoolList from './components/SeatSwapPoolList.vue'
import SeatSwapDock from './components/SeatSwapDock.vue'
import SeatSwapPublishSheet from './components/SeatSwapPublishSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import type { SeatSwapCandidate } from '../../types/seatSwap'
import { buildSharePath } from '../../utils/shareSource'
import { buildSeatSwapRegionAnchorId } from './helpers'
import { useSeatSwapActions } from './useSeatSwapActions'
import { useSeatSwapBrowse } from './useSeatSwapBrowse'
import { useSeatSwapPage } from './useSeatSwapPage'

onShareAppMessage(() => ({
  title: '球迷换座位，看看有没有合适的位置',
  path: buildSharePath('/pages/seat-swap/index', 'swap'),
}))

const { navigationPageStyle } = useNavigationLayout()

const USE_SEAT_SWAP_MOCK_LARGE_DATA =
  import.meta.env.DEV && String(import.meta.env.VITE_SEAT_SWAP_MOCK_LARGE_DATA || '').trim() === '1'

// 登录标记共享自 appSession，退出/换号后自动更新，不再读本地存储。
const isLoggedIn = useAppSession().hasAccessToken
const submitting = ref(false)
const publishSheetVisible = ref(false)
const manageSheetVisible = ref(false)
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const pendingMatchedCancelTargetId = ref('')
const miniProgramNoticeEnabled = ref(false)
const cancelReason = ref('')

const anySheetOpen = computed(() => publishSheetVisible.value || manageSheetVisible.value)
const pageLockStyle = computed(() => anySheetOpen.value ? 'overflow: hidden;' : '')

const { loading, errorMessage, currentView, regions, loadPage } = useSeatSwapPage({
  useMock: USE_SEAT_SWAP_MOCK_LARGE_DATA,
  onLoaded: () => browse.syncExpandedRegionKeys(),
})
const browse = useSeatSwapBrowse({ currentView, regions, sheetOpen: anySheetOpen })
const {
  browsingFilterKey,
  expandedRegionKeys,
  matchTitle,
  matchSummary,
  candidatesCount,
  seatSwapRegionGroups,
  filteredCandidates,
  browsingFilterName,
  regionBadgeCounts,
  myDesiredKeys,
  myDesiredSummary,
  myRequestSeatLabel,
  myRequestStatusLabel,
  mainMapMode,
  toggleRegionGroup,
  clearFilter,
} = browse

const { submitForm, deleteRequest, confirmCandidate, cancelCandidateConfirmation, submitMatchedCancel } = useSeatSwapActions({
  submitting,
  currentView,
  publishSheetVisible,
  pendingConfirmTarget,
  manageSheetVisible,
  miniProgramNoticeEnabled,
  pendingMatchedCancelTargetId,
  cancelReason,
  loadPage,
  openPublishSheetForCandidate,
})

function handleMainMapTap(key: string): void {
  const outcome = browse.applyMapTap(key, false)
  if (outcome.scrollTo) void scrollToSeatSwapRegion(outcome.scrollTo)
}

async function scrollToSeatSwapRegion(regionKey: string): Promise<void> {
  await new Promise<void>((resolve) => {
    nextTick(() => {
      uni.pageScrollTo({
        selector: `#${buildSeatSwapRegionAnchorId(regionKey)}`,
        duration: 220,
        success: () => resolve(),
        fail: () => resolve(),
      })
    })
  })
}

function openPublishSheet(): void {
  pendingConfirmTarget.value = null
  publishSheetVisible.value = true
}

function openPublishSheetForCandidate(candidate: SeatSwapCandidate): void {
  pendingConfirmTarget.value = candidate
  publishSheetVisible.value = true
}

function openManageSheet(): void {
  cancelReason.value = ''
  pendingMatchedCancelTargetId.value = ''
  manageSheetVisible.value = true
}

function closeManageSheet(): void {
  manageSheetVisible.value = false
  pendingMatchedCancelTargetId.value = ''
}

function openEditFromManage(): void {
  manageSheetVisible.value = false
  openPublishSheet()
}

function handleDockCtaTap(): void {
  if (!isLoggedIn.value) {
    goToUserPage()
    return
  }
  openPublishSheet()
}

function openMatchedCancelForCandidate(requestId: string): void {
  pendingMatchedCancelTargetId.value = requestId
  cancelReason.value = ''
  manageSheetVisible.value = true
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

onShow(() => {
  void loadPage()
})
</script>

<style scoped>
.seat-map-section { position: sticky; top: var(--fi-brand-nav-height); z-index: 5; margin: 0 calc(-1 * var(--fi-space-24)); }
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

/* 背景遮罩（spec 第 7 节）：照片区 32vh 为半透明白，以下为页面底色，纯色替代原渐变。 */
.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}

.page-bg-fade::before,
.page-bg-fade::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
}

.page-bg-fade::before {
  top: 0;
  height: 32vh;
  background: rgba(var(--fi-primitive-white-rgb), 0.5);
}

.page-bg-fade::after {
  top: 32vh;
  bottom: 0;
  background: var(--fi-color-page-soft);
}

.page {
  gap: var(--fi-space-card-gap);
  flex-direction: column;
  display: flex;
  position: relative;
  z-index: 1;
  min-height: calc(100vh - var(--fi-brand-nav-height));
  padding-top: calc(var(--fi-brand-nav-height) + var(--fi-space-28));
  padding-right: var(--fi-space-24);
  padding-bottom: 240rpx;
  padding-left: var(--fi-space-24);
  box-sizing: border-box;
}

.page-content {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
  animation: seat-swap-content-in 200ms var(--fi-motion-ease-out) both;
}

@keyframes seat-swap-content-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.state-card {
  padding: var(--fi-space-28);
  border: var(--fi-border-card);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-surface-frosted);
  box-shadow: var(--fi-shadow-soft);
  text-align: center;
}

.state-card--error {
  color: var(--fi-color-primary-deep);
}

.ghost-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-12) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.ghost-action::after {
  border: 0;
}
</style>
