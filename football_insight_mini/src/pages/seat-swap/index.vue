<template>
  <page-meta :page-style="pageLockStyle" />
  <view class="page-root">
    <FiBrandNav open-on-current-page @open-ai="openAiFromBrandNav" />
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page">
      <view v-if="loading" class="state-card">
        <text>正在加载换座池...</text>
      </view>

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
        <button class="ghost-action" @tap="loadPage">重试</button>
      </view>

      <template v-else>
        <SeatSwapHero
          :match-title="matchTitle"
          :match-summary="matchSummary"
          :candidates-count="candidatesCount"
        />

        <view v-if="!currentView?.available" class="info-row">
          <text class="info-row__text">换座撮合只在成都蓉城当前比赛开放。</text>
        </view>
        <view v-else-if="!isLoggedIn" class="info-row info-row--login">
          <text class="info-row__text">未登录可浏览脱敏意向;发布、确认和查看联系方式需要登录。</text>
          <button class="info-row__action" @tap="goToUserPage">去登录</button>
        </view>

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
      </template>
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

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="aiPublicConfig?.ai_chat_mode"
      @close="closeAiChat"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import SeatSwapManageSheet from '../../components/SeatSwapManageSheet.vue'
import SeatSwapHero from './components/SeatSwapHero.vue'
import SeatSwapRegionPanel from './components/SeatSwapRegionPanel.vue'
import SeatSwapPoolList from './components/SeatSwapPoolList.vue'
import SeatSwapDock from './components/SeatSwapDock.vue'
import SeatSwapPublishSheet from './components/SeatSwapPublishSheet.vue'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import {
  MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID,
  cancelSeatSwapCandidateConfirmation,
  confirmSeatSwapCandidate,
  deleteMySeatSwapRequest,
  getCurrentSeatSwap,
  upsertMySeatSwapRequest,
  cancelMatchedSeatSwap,
} from '../../api/seatSwap'
import { getTicketWatchRegions } from '../../api/ticketWatch'
import type { SeatSwapCandidate, SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { getAccessToken } from '../../utils/authStorage'
import { extractApiErrorMessage } from '../../utils/apiError'
import { formatDatetime } from '../../utils/format'
import { useAiChatSheet } from '../../composables/useAiChatSheet'
import {
  buildSeatSwapMockCurrentResponse,
  buildSeatSwapMockRegions,
  buildSeatSwapRegionAnchorId,
  countSeatSwapDesiredRegions,
  filterSeatSwapRequestsByDesiredRegion,
  filterOutMySeatSwapRequest,
  formatSeatLabel,
  groupSeatSwapRequestsByRegion,
  resolveSeatSwapBrowseFilterKey,
  resolveDefaultExpandedSeatSwapRegionKeys,
  statusLabel,
  type SeatSwapFormState,
} from './helpers'

const USE_SEAT_SWAP_MOCK_LARGE_DATA =
  import.meta.env.DEV && String(import.meta.env.VITE_SEAT_SWAP_MOCK_LARGE_DATA || '').trim() === '1'

const loading = ref(true)
const submitting = ref(false)
const errorMessage = ref('')
const currentView = ref<SeatSwapCurrentResponse | null>(null)
const regions = ref<TicketWatchRegion[]>([])
const isLoggedIn = ref(false)

const publishSheetVisible = ref(false)
const manageSheetVisible = ref(false)
const browsingFilterKey = ref('')

const pageScrollLocked = computed(() => publishSheetVisible.value || manageSheetVisible.value)
const pageLockStyle = computed(() => pageScrollLocked.value ? 'overflow: hidden;' : '')

const expandedRegionKeys = ref<string[]>([])
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const pendingMatchedCancelTargetId = ref('')
const miniProgramNoticeEnabled = ref(false)
const {
  aiChatVisible,
  currentAiUser,
  aiPublicConfig,
  openAiChat,
  closeAiChat,
} = useAiChatSheet()

const cancelReason = ref('')

const matchTitle = computed(() => {
  const m = currentView.value?.current_match
  return m ? `${m.home_team_name} VS ${m.away_team_name}` : '暂无当前比赛'
})

const matchSummary = computed(() => {
  const raw = currentView.value?.current_match?.kickoff_at
  if (!raw) return ''
  return `${formatDatetime(raw)} · 凤凰山专业足球场`
})

const candidatesCount = computed(() => currentView.value?.candidates.length || 0)

const displayCandidates = computed<SeatSwapCandidate[]>(() => {
  return filterOutMySeatSwapRequest(
    currentView.value?.candidates || [],
    currentView.value?.my_request?.request_id,
  )
})

const seatSwapRegionGroups = computed(() => groupSeatSwapRequestsByRegion(displayCandidates.value))

const filteredCandidates = computed<SeatSwapCandidate[]>(() => {
  if (!browsingFilterKey.value) return []
  return filterSeatSwapRequestsByDesiredRegion(displayCandidates.value, browsingFilterKey.value)
})

const browsingFilterName = computed(() => findRegion(browsingFilterKey.value)?.block_name || '')

const regionBadgeCounts = computed<Record<string, number>>(() => {
  return countSeatSwapDesiredRegions(displayCandidates.value)
})

const myDesiredKeys = computed<string[]>(() =>
  currentView.value?.my_request?.desired_seats.map((s) => s.region_key) || [],
)

const myDesiredSummary = computed(() => {
  const names = currentView.value?.my_request?.desired_seats.map((s) => s.region_name) || []
  return names.length ? names.join(' / ') : '未选择'
})

const myRequestSeatLabel = computed(() =>
  currentView.value?.my_request ? formatSeatLabel(currentView.value.my_request) : '',
)

const myRequestStatusLabel = computed(() => statusLabel(currentView.value?.my_request?.status || ''))

const mainMapMode = computed<'browse' | 'filter' | 'published'>(() => {
  if (browsingFilterKey.value) return 'filter'
  if (currentView.value?.my_request) return 'published'
  return 'browse'
})

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function findRegion(key: string): TicketWatchRegion | undefined {
  if (!key) return undefined
  return regions.value.find((r) => regionKey(r) === key)
}

function syncExpandedRegionKeys(): void {
  const available = seatSwapRegionGroups.value.map((g) => g.region_key)
  expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => available.includes(k))
  if (!expandedRegionKeys.value.length && available.length) {
    expandedRegionKeys.value = resolveDefaultExpandedSeatSwapRegionKeys(available)
  }
}

function toggleRegionGroup(key: string): void {
  if (expandedRegionKeys.value.includes(key)) {
    expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => k !== key)
  } else {
    expandedRegionKeys.value = [...expandedRegionKeys.value, key]
  }
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''
  isLoggedIn.value = !!getAccessToken()
  try {
    if (USE_SEAT_SWAP_MOCK_LARGE_DATA) {
      currentView.value = buildSeatSwapMockCurrentResponse({
        candidateCount: 96,
        includeMyRequest: true,
      })
      regions.value = buildSeatSwapMockRegions()
      syncExpandedRegionKeys()
      return
    }

    const [view, regionList] = await Promise.all([
      getCurrentSeatSwap(),
      getTicketWatchRegions(),
    ])
    currentView.value = view
    regions.value = regionList
    syncExpandedRegionKeys()
  } catch (err) {
    errorMessage.value = extractApiErrorMessage(err, '换座池加载失败')
  } finally {
    loading.value = false
  }
}

function handleMainMapTap(key: string): void {
  const hasBadge = Boolean(regionBadgeCounts.value[key])
  const nextFilterKey = resolveSeatSwapBrowseFilterKey(browsingFilterKey.value, key)

  if (hasBadge) {
    browsingFilterKey.value = nextFilterKey
    if (nextFilterKey) {
      void scrollToSeatSwapRegion(key)
    }
    return
  }

  browsingFilterKey.value = nextFilterKey
}

function clearFilter(): void {
  browsingFilterKey.value = ''
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

async function submitForm(payload: { form: SeatSwapFormState; presetCandidate: SeatSwapCandidate | null }): Promise<void> {
  const form = payload.form
  submitting.value = true
  try {
    const subscribeAccepted = await requestSeatSwapSubscribeMessage()
    await upsertMySeatSwapRequest({
      current_region_key: form.current_region_key,
      current_region_name: form.current_region_name,
      current_row: form.current_row,
      current_seat_no: form.current_seat_no,
      wechat_id: form.wechat_id || null,
      phone_number: form.phone_number || null,
      seat_swap_notice_enabled: subscribeAccepted,
      desired_seats: form.desired_seats.map((s) => ({
        region_key: s.region_key,
        region_name: s.region_name,
        desired_row: s.desired_row || null,
        desired_seat_no: s.desired_seat_no || null,
      })),
    })
    if (payload.presetCandidate) {
      await confirmSeatSwapCandidate(payload.presetCandidate.request_id)
      uni.showToast({ title: '已确认换座', icon: 'success' })
    } else {
      uni.showToast({ title: '发布成功', icon: 'success' })
    }
    publishSheetVisible.value = false
    pendingConfirmTarget.value = null
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '发布失败'), icon: 'none' })
  } finally {
    submitting.value = false
  }
}

async function deleteRequest(): Promise<void> {
  try {
    manageSheetVisible.value = false
    await deleteMySeatSwapRequest()
    if (currentView.value) {
      currentView.value = {
        ...currentView.value,
        my_request: null,
        candidates: filterOutMySeatSwapRequest(
          currentView.value.candidates,
          currentView.value.my_request?.request_id,
        ),
      }
    }
    miniProgramNoticeEnabled.value = false
    uni.showToast({ title: '已撤销', icon: 'success' })
    await loadPage()
  } catch (err) {
    manageSheetVisible.value = true
    uni.showToast({ title: extractApiErrorMessage(err, '撤销失败'), icon: 'none' })
  }
}

async function confirmCandidate(requestId: string): Promise<void> {
  const mine = currentView.value?.my_request
  if (!mine) {
    const candidate = currentView.value?.candidates.find((item) => item.request_id === requestId)
    if (!candidate) {
      uni.showToast({ title: '换座对象不存在', icon: 'none' })
      return
    }
    openPublishSheetForCandidate(candidate)
    return
  }

  try {
    await confirmSeatSwapCandidate(requestId)
    uni.showToast({ title: '已确认', icon: 'success' })
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '确认失败'), icon: 'none' })
  }
}

async function confirmSeatSwapCancellation(): Promise<boolean> {
  return await new Promise<boolean>((resolve) => {
    uni.showModal({
      title: '确认取消匹配',
      content: '取消后需要重新发起确认，确定继续吗？',
      confirmText: '确定取消',
      cancelText: '先保留',
      success: (result) => resolve(Boolean(result.confirm)),
      fail: () => resolve(false),
    })
  })
}

async function cancelCandidateConfirmation(requestId: string): Promise<void> {
  const confirmed = await confirmSeatSwapCancellation()
  if (!confirmed) {
    return
  }

  try {
    await cancelSeatSwapCandidateConfirmation(requestId)
    uni.showToast({ title: '已取消匹配', icon: 'success' })
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '取消失败'), icon: 'none' })
  }
}

function openMatchedCancelForCandidate(requestId: string): void {
  pendingMatchedCancelTargetId.value = requestId
  cancelReason.value = ''
  manageSheetVisible.value = true
}

async function requestSeatSwapSubscribeMessage(): Promise<boolean> {
  if (!MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID) {
    return miniProgramNoticeEnabled.value
  }

  // #ifdef MP-WEIXIN
  return await new Promise<boolean>((resolve) => {
    uni.requestSubscribeMessage({
      tmplIds: [MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID],
      success: (result) => {
        const subscribeResult = result as unknown as Record<string, string | undefined>
        const accepted = subscribeResult[MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID] === 'accept'
        miniProgramNoticeEnabled.value = accepted
        resolve(accepted)
      },
      fail: () => {
        resolve(miniProgramNoticeEnabled.value)
      },
    })
  })
  // #endif

  return miniProgramNoticeEnabled.value
}

async function submitMatchedCancel(): Promise<void> {
  const matchedId = currentView.value?.my_request?.request_id
  const target = currentView.value?.candidates.find((candidate) => {
    if (pendingMatchedCancelTargetId.value) {
      return candidate.request_id === pendingMatchedCancelTargetId.value
    }
    return candidate.status === 'matched'
  })
  if (!matchedId || !target) {
    uni.showToast({ title: '暂无可撤销的匹配', icon: 'none' })
    return
  }
  if (!cancelReason.value.trim()) {
    uni.showToast({ title: '请填写撤销说明', icon: 'none' })
    return
  }
  try {
    await cancelMatchedSeatSwap(target.request_id, {
      reason: cancelReason.value,
      evidence_file_name: '',
      evidence_content_type: '',
      evidence_base64: '',
    })
    uni.showToast({ title: '已提交撤销', icon: 'success' })
    manageSheetVisible.value = false
    pendingMatchedCancelTargetId.value = ''
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '提交撤销失败'), icon: 'none' })
  }
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

function openAiFromBrandNav(): void {
  void openAiChat()
}

onShow(() => {
  void loadPage()
})
</script>

<style scoped>
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
  background:
    linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, var(--fi-color-page-soft) 58%, var(--fi-color-page-soft) 100%);
  pointer-events: none;
  z-index: 0;
}

.page {
  position: relative;
  z-index: 1;
  min-height: calc(100vh - var(--fi-brand-nav-height));
  padding-top: calc(var(--fi-brand-nav-height) + var(--fi-space-28));
  padding-right: var(--fi-space-24);
  padding-bottom: 240rpx;
  padding-left: var(--fi-space-24);
  box-sizing: border-box;
}

.state-card {
  margin-top: var(--fi-space-22);
  padding: var(--fi-space-28);
  border: 1rpx solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-lg);
  background: rgba(255, 255, 255, 0.96);
  box-shadow: var(--fi-shadow-soft);
  text-align: center;
}

.state-card--error {
  color: #b42318;
}

.info-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-22);
  margin-bottom: var(--fi-space-14);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
  border: 1rpx dashed rgba(207, 211, 220, 0.95);
}

.info-row__text {
  flex: 1;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.info-row__action {
  flex-shrink: 0;
  padding: var(--fi-space-10) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: 400;
}

.info-row__action::after {
  border: 0;
}

.ghost-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-12) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-white);
  border: 1rpx solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.ghost-action::after {
  border: 0;
}
</style>
