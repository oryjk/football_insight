<template>
  <div class="page-root">
    <img class="page-bg-img" :src="phoenixStadiumBgImage" alt="" />
    <div class="page-bg-fade"></div>

    <header class="topbar">
      <span class="topbar__brand">足球洞察 · 换座位</span>
      <button v-if="isLoggedIn" type="button" class="topbar__logout" @click="handleLogout">退出登录</button>
    </header>

    <main class="seat-swap">
      <div v-if="loading" class="state-card">
        <span>正在加载换座池...</span>
      </div>

      <div v-else-if="errorMessage" class="state-card state-card--error">
        <span>{{ errorMessage }}</span>
        <button type="button" class="ghost-action" @click="loadPage">重试</button>
      </div>

      <template v-else>
        <SeatSwapHero
          :match-title="matchTitle"
          :match-summary="matchSummary"
          :candidates-count="candidatesCount"
          :is-logged-in="isLoggedIn"
          :has-my-request="!!currentView?.my_request"
          :my-seat-label="myRequestSeatLabel"
          :my-desired-summary="myDesiredSummary"
          :my-status-label="myRequestStatusLabel"
          @login="scrollToLogin"
          @publish="openPublishDialog"
          @manage="openManageDialog"
        />

        <div v-if="!currentView?.available" class="info-row">
          <span class="info-row__text">换座撮合只在成都蓉城当前比赛开放。</span>
        </div>

        <template v-if="!isLoggedIn">
          <div ref="loginAnchor" class="login-block">
            <div class="info-row">
              <span class="info-row__text">未登录可浏览脱敏意向；发布、确认和查看联系方式需要登录。</span>
            </div>
            <TestLoginPanel :users="testUsers" @select="handleTestLogin" />
          </div>
        </template>

        <div v-if="currentView?.available" class="seat-swap__grid">
          <div class="seat-swap__map-col">
            <div class="seat-swap__map-card">
              <div class="seat-swap__map-head">
                <span class="seat-swap__map-title">座位区域</span>
                <span class="seat-swap__map-hint">球场分区示意 · 颜色对应票档</span>
              </div>
              <StadiumMap
                :mode="mainMapMode"
                :regions="regions"
                :badges="regionBadgeCounts"
                :filter-key="browsingFilterKey"
                :current-key="currentView?.my_request?.current_region_key || ''"
                :desired-keys="myDesiredKeys"
              />
              <div class="seat-swap__legend">
                <template v-if="currentView?.my_request">
                  <div class="seat-swap__legend-item">
                    <span class="seat-swap__legend-dot seat-swap__legend-dot--current"></span>
                    <span>当前座位</span>
                  </div>
                  <div class="seat-swap__legend-item">
                    <span class="seat-swap__legend-dot seat-swap__legend-dot--desired"></span>
                    <span>目标座位</span>
                  </div>
                </template>
                <div v-else class="seat-swap__legend-item">
                  <span class="seat-swap__legend-dot seat-swap__legend-dot--hot"></span>
                  <span>有发布</span>
                </div>
              </div>
            </div>

            <div class="seat-swap__chip-card">
              <div class="seat-swap__map-head">
                <span class="seat-swap__map-title">按分区筛选</span>
                <span class="seat-swap__map-hint">{{ chipHint }}</span>
              </div>
              <RegionChipGrid
                :mode="mainMapMode"
                :regions="regions"
                :badges="regionBadgeCounts"
                :filter-key="browsingFilterKey"
                :current-key="currentView?.my_request?.current_region_key || ''"
                :desired-keys="myDesiredKeys"
                @region-tap="handleMainMapTap"
              />
            </div>
          </div>

          <SeatSwapPool
            class="seat-swap__pool-col"
            :groups="seatSwapRegionGroups"
            :filter-key="browsingFilterKey"
            :filter-name="browsingFilterName"
            :filtered-candidates="filteredCandidates"
            :total-count="candidatesCount"
            :my-desired-keys="myDesiredKeys"
            :is-logged-in="isLoggedIn"
            :my-request-id="currentView?.my_request?.request_id ?? null"
            @clear-filter="clearFilter"
            @confirm="confirmCandidate"
            @cancel-confirmation="cancelCandidateConfirmation"
            @matched-cancel="openMatchedCancelForCandidate"
          />
        </div>
      </template>
    </main>

    <SeatSwapPublishDialog
      v-model:visible="publishDialogVisible"
      :regions="regions"
      :my-request="currentView?.my_request ?? null"
      :preset-candidate="pendingConfirmTarget"
      :submitting="submitting"
      @submit="submitForm"
    />

    <SeatSwapManageDialog
      v-model:visible="manageDialogVisible"
      v-model:cancel-mode="manageCancelMode"
      v-model:cancel-reason="cancelReason"
      :request="currentView?.my_request"
      :status-label="myRequestStatusLabel"
      :desired-summary="myDesiredSummary"
      @edit="openEditFromManage"
      @delete="deleteRequest"
      @submit-matched-cancel="submitMatchedCancel"
    />

    <ConfirmDialog
      :visible="confirmState.visible"
      :title="confirmState.title"
      :content="confirmState.content"
      :confirm-text="confirmState.confirmText"
      @confirm="resolveConfirm(true)"
      @cancel="resolveConfirm(false)"
    />

    <div v-if="toastState.visible" class="toast">{{ toastState.message }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import SeatSwapHero from '../../components/SeatSwapHero.vue'
import SeatSwapPool from '../../components/SeatSwapPool.vue'
import SeatSwapPublishDialog from '../../components/SeatSwapPublishDialog.vue'
import SeatSwapManageDialog from '../../components/SeatSwapManageDialog.vue'
import RegionChipGrid from '../../components/RegionChipGrid.vue'
import StadiumMap from '../../components/StadiumMap.vue'
import ConfirmDialog from '../../components/ConfirmDialog.vue'
import TestLoginPanel from '../../components/TestLoginPanel.vue'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import {
  cancelMatchedSeatSwap,
  cancelSeatSwapCandidateConfirmation,
  confirmSeatSwapCandidate,
  deleteMySeatSwapRequest,
  getCurrentSeatSwap,
  upsertMySeatSwapRequest,
} from '../../api/seatSwap'
import { getTicketWatchRegions } from '../../api/ticketWatch'
import { listH5TestLoginUsers, loginAsH5TestUser } from '../../api/auth'
import type { SeatSwapCandidate, SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import type { H5TestLoginUser } from '../../types/auth'
import { clearAccessToken, getAccessToken, initTokenFromUrl } from '../../lib/auth'
import { extractApiErrorMessage } from '../../lib/request'
import { showToast, toastState } from '../../lib/toast'
import { formatDatetime } from '../../lib/format'
import {
  buildSeatSwapMockCurrentResponse,
  buildSeatSwapMockRegions,
  countSeatSwapDesiredRegions,
  filterOutMySeatSwapRequest,
  filterSeatSwapRequestsByDesiredRegion,
  formatSeatLabel,
  groupSeatSwapRequestsByRegion,
  resolveSeatSwapBrowseFilterKey,
  statusLabel,
  type SeatSwapFormState,
} from '../../utils/seatSwap'

const USE_MOCK_LARGE_DATA =
  import.meta.env.DEV && String(import.meta.env.VITE_SEAT_SWAP_MOCK_LARGE_DATA || '').trim() === '1'

const loading = ref(true)
const submitting = ref(false)
const errorMessage = ref('')
const currentView = ref<SeatSwapCurrentResponse | null>(null)
const regions = ref<TicketWatchRegion[]>([])
const isLoggedIn = ref(false)
const testUsers = ref<H5TestLoginUser[]>([])
const loginAnchor = ref<HTMLElement | null>(null)

const publishDialogVisible = ref(false)
const manageDialogVisible = ref(false)
const manageCancelMode = ref(false)
const browsingFilterKey = ref('')
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const pendingMatchedCancelTargetId = ref('')
const cancelReason = ref('')

const confirmState = reactive({
  visible: false,
  title: '',
  content: '',
  confirmText: '确定',
  resolver: null as ((value: boolean) => void) | null,
})

const anyDialogVisible = computed(
  () => publishDialogVisible.value || manageDialogVisible.value || confirmState.visible,
)
watch(anyDialogVisible, (locked) => {
  document.body.style.overflow = locked ? 'hidden' : ''
})

const matchTitle = computed(() => {
  const match = currentView.value?.current_match
  return match ? `${match.home_team_name} VS ${match.away_team_name}` : '暂无当前比赛'
})

const matchSummary = computed(() => {
  const raw = currentView.value?.current_match?.kickoff_at
  if (!raw) return ''
  return `${formatDatetime(raw)} · 凤凰山专业足球场`
})

const candidatesCount = computed(() => currentView.value?.candidates.length || 0)

const displayCandidates = computed<SeatSwapCandidate[]>(() =>
  filterOutMySeatSwapRequest(
    currentView.value?.candidates || [],
    currentView.value?.my_request?.request_id,
  ),
)

const seatSwapRegionGroups = computed(() => groupSeatSwapRequestsByRegion(displayCandidates.value))

const filteredCandidates = computed<SeatSwapCandidate[]>(() => {
  if (!browsingFilterKey.value) return []
  return filterSeatSwapRequestsByDesiredRegion(displayCandidates.value, browsingFilterKey.value)
})

const browsingFilterName = computed(() => findRegion(browsingFilterKey.value)?.block_name || '')

const regionBadgeCounts = computed<Record<string, number>>(() =>
  countSeatSwapDesiredRegions(displayCandidates.value),
)

const myDesiredKeys = computed<string[]>(() =>
  currentView.value?.my_request?.desired_seats.map((seat) => seat.region_key) || [],
)

const myDesiredSummary = computed(() => {
  const names = currentView.value?.my_request?.desired_seats.map((seat) => seat.region_name) || []
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

const chipHint = computed(() => {
  if (browsingFilterKey.value) return `已筛选：想换到 ${browsingFilterName.value} 的球迷`
  return '点色块筛选，再点一次取消'
})

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function findRegion(key: string): TicketWatchRegion | undefined {
  if (!key) return undefined
  return regions.value.find((region) => regionKey(region) === key)
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''
  isLoggedIn.value = !!getAccessToken()
  try {
    if (USE_MOCK_LARGE_DATA) {
      currentView.value = buildSeatSwapMockCurrentResponse({
        candidateCount: 96,
        includeMyRequest: true,
      })
      regions.value = buildSeatSwapMockRegions()
      return
    }

    const [view, regionList] = await Promise.all([
      getCurrentSeatSwap(),
      getTicketWatchRegions(),
    ])
    currentView.value = view
    regions.value = regionList
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '换座池加载失败')
  } finally {
    loading.value = false
  }
}

async function loadTestUsers(): Promise<void> {
  // 后端未配置 H5_TEST_LOGIN_USER_IDS 白名单时接口返回 403，此时隐藏测试登录入口。
  try {
    const result = await listH5TestLoginUsers()
    testUsers.value = result.items
  } catch {
    testUsers.value = []
  }
}

async function handleTestLogin(user: H5TestLoginUser): Promise<void> {
  try {
    await loginAsH5TestUser(user.id)
    showToast(`已切换到 ${user.display_name?.trim() || user.account_identifier}`)
    await loadPage()
  } catch (error) {
    showToast(extractApiErrorMessage(error, '测试登录失败'))
  }
}

function handleLogout(): void {
  clearAccessToken()
  isLoggedIn.value = false
  showToast('已退出登录')
  void loadPage()
}

function scrollToLogin(): void {
  loginAnchor.value?.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

function handleMainMapTap(key: string): void {
  browsingFilterKey.value = resolveSeatSwapBrowseFilterKey(browsingFilterKey.value, key)
}

function clearFilter(): void {
  browsingFilterKey.value = ''
}

function openPublishDialog(): void {
  pendingConfirmTarget.value = null
  publishDialogVisible.value = true
}

function openPublishDialogForCandidate(candidate: SeatSwapCandidate): void {
  pendingConfirmTarget.value = candidate
  publishDialogVisible.value = true
}

function openManageDialog(): void {
  cancelReason.value = ''
  manageCancelMode.value = false
  pendingMatchedCancelTargetId.value = ''
  manageDialogVisible.value = true
}

function openEditFromManage(): void {
  manageDialogVisible.value = false
  openPublishDialog()
}

async function submitForm(payload: { form: SeatSwapFormState; presetCandidate: SeatSwapCandidate | null }): Promise<void> {
  const form = payload.form
  submitting.value = true
  try {
    await upsertMySeatSwapRequest({
      current_region_key: form.current_region_key,
      current_region_name: form.current_region_name,
      current_row: form.current_row,
      current_seat_no: form.current_seat_no,
      wechat_id: form.wechat_id || null,
      phone_number: form.phone_number || null,
      // H5 没有小程序订阅消息能力，统一不开启订阅通知。
      seat_swap_notice_enabled: false,
      desired_seats: form.desired_seats.map((seat) => ({
        region_key: seat.region_key,
        region_name: seat.region_name,
        desired_row: seat.desired_row || null,
        desired_seat_no: seat.desired_seat_no || null,
      })),
    })
    if (payload.presetCandidate) {
      await confirmSeatSwapCandidate(payload.presetCandidate.request_id)
      showToast('已确认换座')
    } else {
      showToast('发布成功')
    }
    publishDialogVisible.value = false
    pendingConfirmTarget.value = null
    await loadPage()
  } catch (error) {
    showToast(extractApiErrorMessage(error, '发布失败'))
  } finally {
    submitting.value = false
  }
}

function askConfirm(title: string, content: string, confirmText: string): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    confirmState.title = title
    confirmState.content = content
    confirmState.confirmText = confirmText
    confirmState.resolver = resolve
    confirmState.visible = true
  })
}

function resolveConfirm(value: boolean): void {
  confirmState.visible = false
  confirmState.resolver?.(value)
  confirmState.resolver = null
}

async function deleteRequest(): Promise<void> {
  const confirmed = await askConfirm('确认撤销发布', '撤销后其他球迷将看不到你的换座意向，确定继续吗？', '确定撤销')
  if (!confirmed) return

  try {
    manageDialogVisible.value = false
    await deleteMySeatSwapRequest()
    showToast('已撤销')
    await loadPage()
  } catch (error) {
    manageDialogVisible.value = true
    showToast(extractApiErrorMessage(error, '撤销失败'))
  }
}

async function confirmCandidate(requestId: string): Promise<void> {
  if (!isLoggedIn.value) {
    scrollToLogin()
    showToast('请先登录后再确认换座')
    return
  }

  const mine = currentView.value?.my_request
  if (!mine) {
    const candidate = currentView.value?.candidates.find((item) => item.request_id === requestId)
    if (!candidate) {
      showToast('换座对象不存在')
      return
    }
    openPublishDialogForCandidate(candidate)
    return
  }

  try {
    await confirmSeatSwapCandidate(requestId)
    showToast('已确认')
    await loadPage()
  } catch (error) {
    showToast(extractApiErrorMessage(error, '确认失败'))
  }
}

async function cancelCandidateConfirmation(requestId: string): Promise<void> {
  const confirmed = await askConfirm('确认取消匹配', '取消后需要重新发起确认，确定继续吗？', '确定取消')
  if (!confirmed) return

  try {
    await cancelSeatSwapCandidateConfirmation(requestId)
    showToast('已取消匹配')
    await loadPage()
  } catch (error) {
    showToast(extractApiErrorMessage(error, '取消失败'))
  }
}

function openMatchedCancelForCandidate(requestId: string): void {
  pendingMatchedCancelTargetId.value = requestId
  cancelReason.value = ''
  manageCancelMode.value = true
  manageDialogVisible.value = true
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
    showToast('暂无可撤销的匹配')
    return
  }
  if (!cancelReason.value.trim()) {
    showToast('请填写撤销说明')
    return
  }
  try {
    await cancelMatchedSeatSwap(target.request_id, {
      reason: cancelReason.value,
      evidence_file_name: '',
      evidence_content_type: '',
      evidence_base64: '',
    })
    showToast('已提交撤销')
    manageDialogVisible.value = false
    manageCancelMode.value = false
    pendingMatchedCancelTargetId.value = ''
    await loadPage()
  } catch (error) {
    showToast(extractApiErrorMessage(error, '提交撤销失败'))
  }
}

onMounted(() => {
  initTokenFromUrl()
  void loadPage()
  void loadTestUsers()
})
</script>

<style scoped>
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f6f5f2;
}

.page-bg-img {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  object-fit: cover;
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
    linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, #f6f5f2 58%, #f6f5f2 100%);
  pointer-events: none;
  z-index: 0;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: rgba(246, 245, 242, 0.85);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(238, 233, 224, 0.8);
}

.topbar__brand {
  color: #17181c;
  font-size: 15px;
  font-weight: 800;
  letter-spacing: 0.5px;
}

.topbar__logout {
  padding: 5px 12px;
  border-radius: 999px;
  border: 1px solid rgba(32, 36, 44, 0.16);
  background: #fff;
  color: #5b5f6a;
  font-size: 12px;
  cursor: pointer;
}

.seat-swap {
  position: relative;
  z-index: 1;
  max-width: 1180px;
  margin: 0 auto;
  padding: 16px 12px 64px;
  box-sizing: border-box;
}

.seat-swap__grid {
  display: grid;
  grid-template-columns: minmax(0, 5fr) minmax(0, 7fr);
  gap: 20px;
  margin-top: 16px;
  align-items: start;
}

.seat-swap__map-col {
  min-width: 0;
}

.seat-swap__map-card {
  position: sticky;
  top: 60px;
  padding: 16px 16px 10px;
  border-radius: 12px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 6px 13px rgba(46, 38, 27, 0.06);
}

.seat-swap__map-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 6px;
}

.seat-swap__map-title {
  color: #17181c;
  font-size: 14px;
  font-weight: 700;
}

.seat-swap__map-hint {
  color: #988f84;
  font-size: 11px;
  text-align: right;
}

.seat-swap__legend {
  display: flex;
  align-items: center;
  gap: 12px;
  padding-top: 8px;
  color: #988f84;
  font-size: 12px;
}

.seat-swap__legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.seat-swap__legend-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.seat-swap__legend-dot--current {
  background: #e23b2e;
}

.seat-swap__legend-dot--desired {
  background: #1d8a55;
}

.seat-swap__legend-dot--hot {
  background: #15161b;
}

.seat-swap__pool-col {
  min-width: 0;
}

.seat-swap__chip-card {
  margin-top: 14px;
  padding: 14px 16px 16px;
  border-radius: 12px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 6px 13px rgba(46, 38, 27, 0.06);
}

.login-block {
  display: grid;
  gap: 10px;
  margin-top: 12px;
}

.state-card {
  margin-top: 20px;
  padding: 24px;
  border: 1px solid rgba(207, 211, 220, 0.95);
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 4px 9px rgba(26, 28, 36, 0.04);
  text-align: center;
}

.state-card--error {
  color: #b42318;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  margin-top: 12px;
  border-radius: 10px;
  background: #fff;
  border: 1px dashed rgba(207, 211, 220, 0.95);
}

.login-block .info-row {
  margin-top: 0;
}

.info-row__text {
  flex: 1;
  color: #5b5f6a;
  font-size: 13px;
  line-height: 1.5;
}

.ghost-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 12px;
  padding: 6px 14px;
  border-radius: 999px;
  background: #fff;
  border: 1px solid rgba(207, 211, 220, 0.95);
  color: #5b5f6a;
  font-size: 13px;
  cursor: pointer;
}

.toast {
  position: fixed;
  left: 50%;
  bottom: 48px;
  transform: translateX(-50%);
  z-index: 200;
  max-width: 80vw;
  padding: 10px 18px;
  border-radius: 999px;
  background: rgba(21, 22, 27, 0.92);
  color: #fff;
  font-size: 13px;
  box-shadow: 0 8px 20px rgba(12, 14, 20, 0.24);
}

@media (max-width: 1000px) {
  .seat-swap {
    padding: 12px 6px 48px;
  }

  .seat-swap__grid {
    grid-template-columns: minmax(0, 1fr);
    gap: 14px;
    margin-top: 12px;
  }

  .seat-swap__map-card {
    position: static;
    padding: 12px 10px 8px;
  }

  .seat-swap__chip-card {
    padding: 12px 10px 12px;
  }
}
</style>
