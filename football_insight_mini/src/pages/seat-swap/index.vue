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
        <SeatSwapNotice v-if="!currentView?.available" text="换座撮合只在成都蓉城当前比赛开放。" />
        <SeatSwapNotice
          v-else-if="!isLoggedIn"
          text="未登录可浏览脱敏意向;发布、确认和查看联系方式需要登录。"
          action-label="去登录"
          @action="goToUserPage"
        />

        <view id="seat-swap-map-anchor" class="seat-map-section">
          <SeatSwapRegionPanel
            :mode="mainMapMode"
            :regions="regions"
            :badges="regionBadgeCounts"
            :filter-key="browsingFilterKey"
            :current-key="currentView?.my_request?.current_region_key || ''"
            :desired-keys="myDesiredKeys"
            :has-my-request="!!currentView?.my_request"
            :headline="matchTitle"
            :headline-meta="matchSummary"
            :badge="`${candidatesCount} 条意向`"
            @region-tap="handleMainMapTap"
          />
        </view>

        <view :id="SEAT_SWAP_POOL_ANCHOR_ID">
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
            :my-request="currentView?.my_request ?? null"
            @toggle-group="toggleRegionGroup"
            @clear-filter="clearFilter"
            @confirm="confirmCandidate"
            @view="openCandidateView"
            @cancel-confirmation="cancelCandidateConfirmation"
            @preview-seat="openSeatPreview"
          />
        </view>
      </view>
    </view>

    <SeatSwapDock
      v-if="!loading && !errorMessage && currentView?.available"
      :has-my-request="!!currentView?.my_request"
      :is-logged-in="isLoggedIn"
      :current-seat-label="myRequestSeatLabel"
      :desired-summary="myDesiredSummary"
      :interested-count="peerConfirmedCount"
      @cta="handleDockCtaTap"
      @manage="openManageSheet"
    />

    <SeatSwapMapFab
      v-if="!loading && !errorMessage"
      :visible="mapFabVisible"
      :label="mapFabLabel.text"
      :clearable="mapFabLabel.clearable"
      @open="openMapSheet"
      @clear="clearFilter"
    />

    <SeatSwapMapSheet
      v-model:visible="mapSheetVisible"
      :mode="mainMapMode"
      :regions="regions"
      :badges="regionBadgeCounts"
      :filter-key="browsingFilterKey"
      :current-key="currentView?.my_request?.current_region_key || ''"
      :desired-keys="myDesiredKeys"
      :has-my-request="!!currentView?.my_request"
      @region-tap="handleSheetMapTap"
    />

    <SeatSwapSeatSheet
      v-model:visible="seatSheetVisible"
      :regions="regions"
      :candidate="seatPreviewCandidate"
    />

    <SeatSwapPublishSheet
      v-model:visible="publishSheetVisible"
      :regions="regions"
      :my-request="currentView?.my_request ?? null"
      :preset-candidate="pendingConfirmTarget"
      :submitting="submitting"
      @submit="submitForm"
    />

    <SeatSwapApplicantSheet
      v-model:visible="applicantSheetVisible"
      :applicants="peerConfirmedCandidates"
      @close="applicantSheetVisible = false"
      @confirm="confirmApplicant"
    />

    <SeatSwapMatchedSheet
      v-model:visible="matchedSheetVisible"
      :candidate="viewingMatchedCandidate"
      @close="matchedSheetVisible = false"
      @cancel-match="cancelMatchFromSheet"
    />

    <SeatSwapManageSheet
      v-model:visible="manageSheetVisible"
      v-model:cancel-reason="cancelReason"
      :request="currentView?.my_request"
      :applicants="peerConfirmedCandidates"
      :status-label="myRequestStatusLabel"
      :desired-summary="myDesiredSummary"
      @close="closeManageSheet"
      @confirm-applicant="confirmApplicant"
      @edit="openEditFromManage"
      @delete="deleteRequest"
      @submit-matched-cancel="submitMatchedCancel"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { onHide, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import SeatSwapApplicantSheet from './components/SeatSwapApplicantSheet.vue'
import SeatSwapMatchedSheet from './components/SeatSwapMatchedSheet.vue'
import SeatSwapManageSheet from './components/SeatSwapManageSheet.vue'
import SeatSwapNotice from './components/SeatSwapNotice.vue'
import SeatSwapSkeleton from './components/SeatSwapSkeleton.vue'
import SeatSwapRegionPanel from './components/SeatSwapRegionPanel.vue'
import SeatSwapPoolList from './components/SeatSwapPoolList.vue'
import SeatSwapDock from './components/SeatSwapDock.vue'
import SeatSwapPublishSheet from './components/SeatSwapPublishSheet.vue'
import SeatSwapMapFab from './components/SeatSwapMapFab.vue'
import SeatSwapMapSheet from './components/SeatSwapMapSheet.vue'
import SeatSwapSeatSheet from './components/SeatSwapSeatSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import type { SeatSwapCandidate } from '../../types/seatSwap'
import { buildSharePath } from '../../utils/shareSource'
import { SEAT_SWAP_POOL_ANCHOR_ID, isSeatSwapMapRectInViewport } from './helpers'
import { useSeatSwapActions } from './useSeatSwapActions'
import { useSeatSwapBrowse } from './useSeatSwapBrowse'
import { useSeatSwapPage } from './useSeatSwapPage'

onShareAppMessage(() => ({
  title: '球迷换座位，看看有没有合适的位置',
  path: buildSharePath('/pages/seat-swap/index', 'swap'),
}))

const { navigationPageStyle, navigationHeightPx } = useNavigationLayout()

const USE_SEAT_SWAP_MOCK_LARGE_DATA =
  import.meta.env.DEV && String(import.meta.env.VITE_SEAT_SWAP_MOCK_LARGE_DATA || '').trim() === '1'

// 登录标记共享自 appSession，退出/换号后自动更新，不再读本地存储。
const isLoggedIn = useAppSession().hasAccessToken
const submitting = ref(false)
const publishSheetVisible = ref(false)
const manageSheetVisible = ref(false)
const applicantSheetVisible = ref(false)
const matchedSheetVisible = ref(false)
const viewingMatchedId = ref('')
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const pendingMatchedCancelTargetId = ref('')
const miniProgramNoticeEnabled = ref(false)
const cancelReason = ref('')

const mapSheetVisible = ref(false)
const seatSheetVisible = ref(false)
const seatPreviewId = ref('')
// 任一弹层打开：锁页面滚动，并隐藏地图浮窗。
const anySheetOpen = computed(() =>
  publishSheetVisible.value || manageSheetVisible.value || applicantSheetVisible.value || matchedSheetVisible.value || mapSheetVisible.value || seatSheetVisible.value,
)
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
  peerConfirmedCount,
  myDesiredKeys,
  myDesiredSummary,
  myRequestSeatLabel,
  myRequestStatusLabel,
  mainMapMode,
  mapFabLabel,
  mapFabVisible,
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
  if (outcome.scrollTo) void scrollToAnchor(outcome.scrollTo)
}

const SHEET_CLOSE_MS = 220 // 与 FiBottomSheet 关闭动画一致
const REGION_HIGHLIGHT_MS = 120
const SCROLL_MS = 300
const MAP_VISIBILITY_DEBOUNCE_MS = 150

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

// 换座卡缩略图点开：弹层里用完整球场图高亮该意向的当前与目标座位。
const seatPreviewCandidate = computed(() =>
  currentView.value?.candidates.find((candidate) => candidate.request_id === seatPreviewId.value) ?? null,
)

function openSeatPreview(requestId: string): void {
  seatPreviewId.value = requestId
  seatSheetVisible.value = true
}

function openMapSheet(): void {
  mapSheetVisible.value = true
}

// 弹层里点有意向的分区：高亮 → 关闭弹层 → 关闭动画结束后再滚动，串行执行；关闭期间忽略连点。
let mapTapBusy = false
async function handleSheetMapTap(key: string): Promise<void> {
  if (mapTapBusy) return
  const outcome = browse.applyMapTap(key, true)
  if (!outcome.closeSheet) return
  mapTapBusy = true
  try {
    await wait(REGION_HIGHLIGHT_MS)
    mapSheetVisible.value = false
    await wait(SHEET_CLOSE_MS)
    if (outcome.scrollTo) await scrollToAnchor(outcome.scrollTo)
  } finally {
    mapTapBusy = false
  }
}

// 地图是否在视口内：每次加载成功后重新挂载 observer（加载中地图节点尚未渲染）。
const pageInstance = getCurrentInstance()
let mapObserver: UniNamespace.IntersectionObserver | null = null
let mapVisibilityTimer: ReturnType<typeof setTimeout> | null = null

function detachMapObserver(): void {
  mapObserver?.disconnect()
  mapObserver = null
  if (mapVisibilityTimer) clearTimeout(mapVisibilityTimer)
  mapVisibilityTimer = null
}

function attachMapObserver(): void {
  detachMapObserver()
  browse.mapInViewport.value = true
  if (!pageInstance || !currentView.value?.available) return
  const observer = createPageIntersectionObserver()
  if (!observer) return
  mapObserver = observer
  // 扣除被品牌导航遮挡的顶部区域
  observer.relativeToViewport({ top: -navigationHeightPx.value })
  observer.observe('#seat-swap-map-anchor', (res) => {
    const visible = (res.intersectionRatio ?? 0) > 0
    if (mapVisibilityTimer) clearTimeout(mapVisibilityTimer)
    mapVisibilityTimer = setTimeout(() => {
      browse.mapInViewport.value = visible
    }, MAP_VISIBILITY_DEBOUNCE_MS)
  })
  // IntersectionObserver 只在比例变化时回调：挂载时地图若已在屏幕外不会触发，主动量一次作为初值。
  seedMapVisibility()
}

function createPageIntersectionObserver(): UniNamespace.IntersectionObserver | null {
  const proxy = pageInstance?.proxy as unknown as {
    createIntersectionObserver?: () => UniNamespace.IntersectionObserver
  } | null | undefined
  if (!proxy) return null
  // #ifdef MP-WEIXIN
  // 小程序：uni.createIntersectionObserver 直接透传给 wx，不识别 Vue proxy；用 proxy 上转发到页面实例的方法。
  return proxy.createIntersectionObserver?.() ?? null
  // #endif
  // #ifndef MP-WEIXIN
  return uni.createIntersectionObserver(proxy)
  // #endif
}

function seedMapVisibility(): void {
  if (!pageInstance) return
  let windowHeight = 0
  try {
    windowHeight = uni.getWindowInfo ? uni.getWindowInfo().windowHeight : uni.getSystemInfoSync().windowHeight
  } catch {
    return
  }
  uni.createSelectorQuery()
    .in(pageInstance)
    .select('#seat-swap-map-anchor')
    .boundingClientRect((rect) => {
      const box = rect && !Array.isArray(rect) && typeof rect.top === 'number' && typeof rect.bottom === 'number'
        ? { top: rect.top, bottom: rect.bottom }
        : null
      browse.mapInViewport.value = isSeatSwapMapRectInViewport(box, navigationHeightPx.value, windowHeight)
    })
    .exec()
}

watch(loading, (isLoading) => {
  if (!isLoading && !errorMessage.value) {
    void nextTick(attachMapObserver)
  } else if (isLoading) {
    detachMapObserver()
  }
}, { flush: 'post' })

onHide(detachMapObserver)
onUnload(detachMapObserver)
onBeforeUnmount(detachMapObserver)

const SCROLL_GAP_PX = 12

// 滚到换座池顶部，扣除固定品牌导航，避免标题被遮住。
async function scrollToAnchor(anchorId: string): Promise<void> {
  await new Promise<void>((resolve) => {
    nextTick(() => {
      uni.pageScrollTo({
        selector: `#${anchorId}`,
        offsetTop: -(navigationHeightPx.value + SCROLL_GAP_PX),
        duration: SCROLL_MS,
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

const peerConfirmedCandidates = computed<SeatSwapCandidate[]>(() =>
  (currentView.value?.candidates ?? []).filter((candidate) => candidate.status === 'peer_confirmed_me'),
)

const viewingMatchedCandidate = computed<SeatSwapCandidate | null>(() =>
  currentView.value?.candidates.find((candidate) => candidate.request_id === viewingMatchedId.value) ?? null,
)

// 列表里的「查看」：已匹配的看对方联系方式，对方邀请的进申请人列表确认换座。
function openCandidateView(requestId: string): void {
  const candidate = currentView.value?.candidates.find((item) => item.request_id === requestId)
  if (candidate?.status === 'matched') {
    viewingMatchedId.value = requestId
    matchedSheetVisible.value = true
    return
  }
  applicantSheetVisible.value = true
}

function cancelMatchFromSheet(requestId: string): void {
  matchedSheetVisible.value = false
  openMatchedCancelForCandidate(requestId)
}

async function confirmApplicant(requestId: string): Promise<void> {
  await confirmCandidate(requestId)
  // 邀请人都处理完（或已匹配）后收起弹层；确认失败时仍保留列表。
  if (!peerConfirmedCandidates.value.length) applicantSheetVisible.value = false
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
/* 地图不再吸顶；划出视口后由浮窗唤起地图弹层。地图面板与其他卡片同宽，不再横向出血。 */
.seat-map-section { display: flex; flex-direction: column; align-items: stretch; }
.page-root {
  position: relative;
  min-height: 100vh;
  background: var(--fi-color-page-soft);
}

/* 背景与首页同一手法：顶部 600rpx 球场图随页面滚动，渐变从透明过渡到页面底色，不再有硬边。 */
.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-600);
  pointer-events: none;
  z-index: 0;
}

.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-600);
  background: var(--fi-color-page-backdrop-fade);
  pointer-events: none;
  z-index: 0;
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
