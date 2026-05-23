<template>
  <view class="page-root">
    <view class="page">
      <view v-if="loading" class="state-card">
        <text>正在加载换座池...</text>
      </view>

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
        <button class="ghost-action" @tap="loadPage">重试</button>
      </view>

      <template v-else>
        <view class="hero-card">
          <view class="hero-card__icon-box">
            <text class="hero-card__icon-mark">↔</text>
          </view>
          <view class="hero-card__body">
            <text class="eyebrow">当前比赛 · 换座撮合</text>
            <text class="hero-card__title">{{ matchTitle }}</text>
            <text v-if="matchSummary" class="hero-card__summary">{{ matchSummary }}</text>
          </view>
          <text class="meta-pill">{{ candidatesCount }} 条意向</text>
        </view>

        <view v-if="!currentView?.available" class="info-row">
          <text class="info-row__text">换座撮合只在成都蓉城当前比赛开放。</text>
        </view>
        <view v-else-if="!isLoggedIn" class="info-row info-row--login">
          <text class="info-row__text">未登录可浏览脱敏意向;发布、确认和查看联系方式需要登录。</text>
          <button class="info-row__action" @tap="goToUserPage">去登录</button>
        </view>

        <view class="seat-map-panel">
          <view class="seat-map-panel__head">
            <text class="seat-map-panel__title">座位区域</text>
            <text class="seat-map-panel__hint">点分区可切换</text>
          </view>
          <view class="stadium-wrap">
            <StadiumMap
              :mode="mainMapMode"
              :regions="regions"
              :badges="regionBadgeCounts"
              :filter-key="browsingFilterKey"
              :current-key="currentView?.my_request?.current_region_key || ''"
              :desired-keys="myDesiredKeys"
              @region-tap="handleMainMapTap"
            />
          </view>
          <view class="legend">
            <view class="legend__items">
              <template v-if="currentView?.my_request">
                <view class="legend__item">
                  <view class="legend__dot legend__dot--current"></view>
                  <text>我的当前</text>
                </view>
                <view class="legend__item">
                  <view class="legend__dot legend__dot--desired"></view>
                  <text>我的目标</text>
                </view>
              </template>
              <view v-else class="legend__item">
                <view class="legend__dot legend__dot--hot"></view>
                <text>有发布</text>
              </view>
            </view>
            <text class="legend__hint">点分区可筛选</text>
          </view>
        </view>

        <view v-if="browsingFilterKey" class="filter-row">
          <view class="filter-row__main">
            <text class="filter-row__label">想换到 {{ browsingFilterName || browsingFilterKey }}</text>
            <text class="filter-row__count">{{ filteredCandidates.length }} 条</text>
          </view>
          <button class="filter-row__clear" @tap="clearFilter">✕ 清除</button>
        </view>

        <template v-if="!browsingFilterKey">
          <view class="section-row">
            <text class="section-row__title">换座池</text>
            <text class="section-row__sub">共 {{ candidatesCount }} 条</text>
          </view>

          <view v-if="!seatSwapRegionGroups.length" class="empty-row">
            <text>当前还没有换座意向。</text>
          </view>

          <template v-for="group in seatSwapRegionGroups" :key="group.region_key">
            <view
              :id="buildSeatSwapRegionAnchorId(group.region_key)"
              class="group-row"
              :class="{ 'group-row--open': isRegionGroupExpanded(group.region_key) }"
              @tap="toggleRegionGroup(group.region_key)"
            >
              <view class="group-row__main">
                <text class="group-row__name">{{ group.region_name }}</text>
                <text class="group-row__count">{{ group.requests.length }} 条</text>
                <text v-if="groupHasMyDesired(group)" class="group-row__hit">命中我的目标</text>
              </view>
              <text class="group-row__caret">{{ isRegionGroupExpanded(group.region_key) ? '收起' : '展开' }}</text>
            </view>
            <template v-if="isRegionGroupExpanded(group.region_key)">
              <SeatSwapCandidateCard
                v-for="candidate in group.requests"
                :key="candidate.request_id"
                :candidate="candidate"
                :action="candidateAction(candidate)"
                @confirm="confirmCandidate"
                @cancel-confirmation="cancelCandidateConfirmation"
              />
            </template>
          </template>
        </template>

        <template v-else>
          <view v-if="!filteredCandidates.length" class="empty-row">
            <text>该分区暂无换座意向。</text>
          </view>
          <SeatSwapCandidateCard
            v-for="candidate in filteredCandidates"
            :key="candidate.request_id"
            :candidate="candidate"
            :action="candidateAction(candidate)"
            @confirm="confirmCandidate"
            @cancel-confirmation="cancelCandidateConfirmation"
          />
        </template>
      </template>
    </view>

    <view v-if="!loading && !errorMessage && currentView?.available" class="dock">
      <button
        v-if="!currentView?.my_request"
        class="dock-cta"
        @tap="handleDockCtaTap"
      >
        <text class="dock-cta__icon">+</text>
        <text class="dock-cta__label">{{ isLoggedIn ? '发布我的换座' : '登录后发布' }}</text>
      </button>
      <view v-else class="dock-status" @tap="openManageSheet">
        <view class="dock-status__head">
          <view class="dock-status__label">
            <view class="dock-status__dot"></view>
            <text>{{ myRequestStatusLabel }}</text>
          </view>
          <text class="dock-status__manage">管理 ›</text>
        </view>
        <view class="dock-status__body">
          <text class="dock-status__seat dock-status__seat--current">{{ formatSeatLabel(currentView.my_request) }}</text>
          <text class="dock-status__arrow">→</text>
          <text class="dock-status__seat dock-status__seat--desired">{{ myDesiredSummary }}</text>
        </view>
      </view>
    </view>

    <FiBottomSheet
      v-model:visible="publishSheetVisible"
      :eyebrow="publishSheetEyebrow"
      :title="publishSheetTitle"
    >
      <view class="steps">
        <view
          v-for="item in selectionSteps"
          :key="item.step"
          class="steps__item"
          :class="{
            'steps__item--active': selectionStep === item.step,
            'steps__item--done': item.index < selectionStepIndex,
          }"
          @tap="jumpToStep(item.step)"
        >
          <text>{{ item.index }} · {{ item.label }}</text>
        </view>
      </view>

      <StadiumMap
        :mode="sheetMapMode"
        :regions="regions"
        :staged-current-key="stagedCurrentRegionKey"
        :staged-desired-keys="stagedDesiredKeys"
        :current-key="form.current_region_key"
        :desired-keys="confirmedDesiredKeys"
        @region-tap="handleSheetMapTap"
      />

      <template v-if="selectionStep === 'select_current'">
        <view class="selected-tags">
          <text v-if="stagedCurrentRegionName" class="tag tag--current">已选 · {{ stagedCurrentRegionName }}</text>
          <text v-else class="tag tag--empty">点选你现在的分区</text>
        </view>
        <view class="row-input">
          <view class="row-input__field">
            <text class="row-input__label">当前排</text>
            <input v-model="form.current_row" class="input-box" placeholder="如 8" />
            <text v-if="formErrors.current_row" class="field-error">{{ formErrors.current_row }}</text>
          </view>
          <view class="row-input__field">
            <text class="row-input__label">当前号</text>
            <input v-model="form.current_seat_no" class="input-box" placeholder="如 15" />
            <text v-if="formErrors.current_seat_no" class="field-error">{{ formErrors.current_seat_no }}</text>
          </view>
        </view>
        <text v-if="formErrors.current_region_key" class="field-error">{{ formErrors.current_region_key }}</text>
      </template>

      <template v-else-if="selectionStep === 'select_desired'">
        <view class="selected-tags">
          <text class="tag tag--current">当前 · {{ form.current_region_name }}</text>
          <text
            v-for="seat in stagedDesiredSeats"
            :key="`tag-${seat.region_key}`"
            class="tag tag--desired"
          >
            {{ seat.region_name }}
          </text>
          <text v-if="!stagedDesiredSeats.length" class="tag tag--empty">点选你想换到的分区(可多选)</text>
        </view>
        <view v-if="stagedDesiredSeats.length" class="desired-rows">
          <view
            v-for="seat in stagedDesiredSeats"
            :key="`desired-${seat.region_key}`"
            class="desired-rows__item"
          >
            <text class="desired-rows__name">{{ seat.region_name }}</text>
            <input v-model="seat.desired_row" class="input-box input-box--short" placeholder="排" />
            <input v-model="seat.desired_seat_no" class="input-box input-box--short" placeholder="号" />
          </view>
        </view>
        <text v-if="formErrors.desired_seats" class="field-error">{{ formErrors.desired_seats }}</text>
      </template>

      <template v-else>
        <view class="summary-card">
          <view class="summary-card__row">
            <text class="summary-card__label">当前</text>
            <text class="summary-card__value">{{ formatSeatLabel(form) }}</text>
          </view>
          <view class="summary-card__row">
            <text class="summary-card__label">目标</text>
            <text class="summary-card__value">{{ confirmedDesiredSummary }}</text>
          </view>
        </view>
        <view class="row-input">
          <view class="row-input__field">
            <text class="row-input__label">微信号</text>
            <input v-model="form.wechat_id" class="input-box" placeholder="至少填一项" />
          </view>
          <view class="row-input__field">
            <text class="row-input__label">手机号</text>
            <input v-model="form.phone_number" class="input-box" type="number" placeholder="11 位手机号" />
          </view>
        </view>
        <text v-if="formErrors.contact" class="field-error">{{ formErrors.contact }}</text>
        <text v-if="formErrors.phone_number" class="field-error">{{ formErrors.phone_number }}</text>
      </template>

      <template #footer>
        <view class="sheet-actions">
          <button v-if="selectionStep !== 'select_current'" class="btn-ghost" @tap="goPreviousSelectionStep">上一步</button>
          <button
            v-if="selectionStep === 'select_current'"
            class="btn-primary"
            :disabled="!canConfirmCurrentSelection"
            @tap="confirmCurrentSelection"
          >下一步 →</button>
          <button
            v-else-if="selectionStep === 'select_desired'"
            class="btn-primary"
            :disabled="!canConfirmDesiredSelection"
            @tap="confirmDesiredSelection"
          >下一步 →</button>
          <button
            v-else
            class="btn-primary"
            :disabled="submitting"
            @tap="submitForm"
          >{{ submitting ? '提交中...' : (currentView?.my_request ? '更新发布' : '发布换座') }}</button>
        </view>
      </template>
    </FiBottomSheet>

    <SeatSwapManageSheet
      v-model:visible="manageSheetVisible"
      v-model:cancel-reason="cancelReason"
      :request="currentView?.my_request"
      :status-label="myRequestStatusLabel"
      :desired-summary="myDesiredSummary"
      :evidence-file-name="evidenceFileName"
      @close="closeManageSheet"
      @edit="openEditFromManage"
      @delete="deleteRequest"
      @choose-evidence="chooseEvidence"
      @submit-matched-cancel="submitMatchedCancel"
    />
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import FiBottomSheet from '../../components/FiBottomSheet.vue'
import SeatSwapCandidateCard from '../../components/SeatSwapCandidateCard.vue'
import SeatSwapManageSheet from '../../components/SeatSwapManageSheet.vue'
import StadiumMap from '../../components/StadiumMap.vue'
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
import type { SeatSwapCurrentResponse, SeatSwapCandidate } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { getAccessToken } from '../../utils/authStorage'
import { extractApiErrorMessage } from '../../utils/apiError'
import { formatDatetime } from '../../utils/format'
import {
  buildSeatSwapRegionAnchorId,
  buildSeatSwapMockCurrentResponse,
  buildSeatSwapMockRegions,
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  countSeatSwapDesiredRegions,
  filterSeatSwapRequestsByDesiredRegion,
  filterOutMySeatSwapRequest,
  formatSeatLabel,
  groupSeatSwapRequestsByRegion,
  hasSeatSwapFormErrors,
  previousSeatSwapStep,
  resolveSeatSwapBrowseFilterKey,
  resolveSeatSwapCandidateAction,
  statusLabel,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  type SeatSwapFormErrors,
  type SeatSwapSelectionStep,
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
const formErrors = ref<SeatSwapFormErrors>({})

const publishSheetVisible = ref(false)
const manageSheetVisible = ref(false)
const browsingFilterKey = ref('')

const selectionStep = ref<SeatSwapSelectionStep>('select_current')
const stagedCurrentRegionKey = ref('')
const stagedDesiredSeats = ref<SeatSwapFormState['desired_seats']>([])
const expandedRegionKeys = ref<string[]>([])
const pendingConfirmTarget = ref<SeatSwapCandidate | null>(null)
const miniProgramNoticeEnabled = ref(false)

const cancelReason = ref('')
const evidenceFileName = ref('')
const evidenceBase64 = ref('')
const evidenceContentType = ref('image/jpeg')

const form = reactive<SeatSwapFormState>({
  current_region_key: '',
  current_region_name: '',
  current_row: '',
  current_seat_no: '',
  wechat_id: '',
  phone_number: '',
  desired_seats: [],
})

const selectionSteps: Array<{ step: SeatSwapSelectionStep; index: number; label: string }> = [
  { step: 'select_current', index: 1, label: '当前' },
  { step: 'select_desired', index: 2, label: '目标' },
  { step: 'ready_to_publish', index: 3, label: '联系' },
]

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

const myRequestStatusLabel = computed(() => statusLabel(currentView.value?.my_request?.status || ''))

const mainMapMode = computed<'browse' | 'filter' | 'published'>(() => {
  if (browsingFilterKey.value) return 'filter'
  if (currentView.value?.my_request) return 'published'
  return 'browse'
})

const sheetMapMode = computed<'select-current' | 'select-desired' | 'review'>(() => {
  if (selectionStep.value === 'select_current') return 'select-current'
  if (selectionStep.value === 'select_desired') return 'select-desired'
  return 'review'
})

const stagedCurrentRegionName = computed(() => findRegion(stagedCurrentRegionKey.value)?.block_name || '')
const stagedDesiredKeys = computed<string[]>(() => stagedDesiredSeats.value.map((s) => s.region_key).filter(Boolean))
const confirmedDesiredKeys = computed<string[]>(() => form.desired_seats.map((s) => s.region_key).filter(Boolean))
const confirmedDesiredSummary = computed(() => {
  const names = form.desired_seats.map((s) => s.region_name)
  return names.length ? names.join(' / ') : '未选择'
})

const selectionStepIndex = computed(() => {
  if (selectionStep.value === 'select_current') return 1
  if (selectionStep.value === 'select_desired') return 2
  return 3
})

const publishSheetEyebrow = computed(() => `第 ${selectionStepIndex.value} 步 / 共 3 步`)

const publishSheetTitle = computed(() => {
  if (selectionStep.value === 'select_current') return '点选你的当前分区'
  if (selectionStep.value === 'select_desired') {
    return pendingConfirmTarget.value ? '已为你预填目标分区' : '点选你想换到的分区'
  }
  return '补充联系方式并发布'
})

const canConfirmCurrentSelection = computed(() =>
  canConfirmCurrentSeatRegion(
    stagedCurrentRegionKey.value,
    form.current_row,
    form.current_seat_no,
  ),
)
const canConfirmDesiredSelection = computed(() => canConfirmDesiredSeatRegions(stagedDesiredSeats.value))

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function findRegion(key: string): TicketWatchRegion | undefined {
  if (!key) return undefined
  return regions.value.find((r) => regionKey(r) === key)
}

function groupHasMyDesired(group: { region_key: string }): boolean {
  return myDesiredKeys.value.includes(group.region_key)
}

function candidateAction(c: SeatSwapCandidate): ReturnType<typeof resolveSeatSwapCandidateAction> {
  const mineId = currentView.value?.my_request?.request_id
  return resolveSeatSwapCandidateAction({
    candidateStatus: c.status,
    candidateRequestId: c.request_id,
    myRequestId: mineId,
    isLoggedIn: isLoggedIn.value,
  })
}

function syncExpandedRegionKeys(): void {
  const available = seatSwapRegionGroups.value.map((g) => g.region_key)
  expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => available.includes(k))
  if (!expandedRegionKeys.value.length && available.length) {
    const hit = available.find((k) => myDesiredKeys.value.includes(k))
    expandedRegionKeys.value = [hit || available[0]]
  }
}

function isRegionGroupExpanded(key: string): boolean {
  return expandedRegionKeys.value.includes(key)
}

function toggleRegionGroup(key: string): void {
  if (isRegionGroupExpanded(key)) {
    expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => k !== key)
  } else {
    expandedRegionKeys.value = [...expandedRegionKeys.value, key]
  }
}

function resetFormForNewRequest(): void {
  form.current_region_key = ''
  form.current_region_name = ''
  form.current_row = ''
  form.current_seat_no = ''
  form.wechat_id = ''
  form.phone_number = ''
  form.desired_seats = []
  formErrors.value = {}
  stagedCurrentRegionKey.value = ''
  stagedDesiredSeats.value = []
  selectionStep.value = 'select_current'
  pendingConfirmTarget.value = null
  miniProgramNoticeEnabled.value = false
}

function fillFormFromMine(): void {
  const mine = currentView.value?.my_request
  if (!mine) {
    resetFormForNewRequest()
    return
  }
  form.current_region_key = mine.current_region_key
  form.current_region_name = mine.current_region_name
  form.current_row = mine.current_row
  form.current_seat_no = mine.current_seat_no
  form.wechat_id = mine.contact?.wechat_id || ''
  form.phone_number = mine.contact?.phone_number || ''
  form.desired_seats = mine.desired_seats.map((s) => ({
    region_key: s.region_key,
    region_name: s.region_name,
    desired_row: s.desired_row || '',
    desired_seat_no: s.desired_seat_no || '',
  }))
  stagedCurrentRegionKey.value = mine.current_region_key
  stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
  formErrors.value = {}
  selectionStep.value = 'ready_to_publish'
  miniProgramNoticeEnabled.value = Boolean(mine.seat_swap_notice_enabled)
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

function handleSheetMapTap(key: string): void {
  const region = findRegion(key)
  if (!region) return
  formErrors.value = {}
  if (selectionStep.value === 'select_current') {
    stagedCurrentRegionKey.value = key
  } else if (selectionStep.value === 'select_desired') {
    if (key === form.current_region_key) return
    stagedDesiredSeats.value = toggleDesiredSeatRegion(stagedDesiredSeats.value, {
      region_key: key,
      region_name: region.block_name,
    })
  }
}

function confirmCurrentSelection(): void {
  const errors: SeatSwapFormErrors = {}
  if (!stagedCurrentRegionKey.value.trim()) {
    errors.current_region_key = '请选择当前分区'
  }
  if (!form.current_row.trim()) {
    errors.current_row = '请输入当前排号'
  }
  if (!form.current_seat_no.trim()) {
    errors.current_seat_no = '请输入当前座号'
  }
  if (hasSeatSwapFormErrors(errors)) {
    formErrors.value = errors
    return
  }

  const region = findRegion(stagedCurrentRegionKey.value)
  if (!region) {
    formErrors.value = { current_region_key: '请选择当前分区' }
    return
  }
  form.current_region_key = regionKey(region)
  form.current_region_name = region.block_name
  if (pendingConfirmTarget.value && stagedDesiredSeats.value.length) {
    form.desired_seats = stagedDesiredSeats.value.map((s) => ({ ...s }))
    selectionStep.value = 'ready_to_publish'
    return
  }
  selectionStep.value = 'select_desired'
}

function confirmDesiredSelection(): void {
  if (!canConfirmDesiredSeatRegions(stagedDesiredSeats.value)) {
    formErrors.value = { desired_seats: '请选择想换到的分区' }
    return
  }
  form.desired_seats = stagedDesiredSeats.value.map((s) => ({ ...s }))
  selectionStep.value = 'ready_to_publish'
}

function goPreviousSelectionStep(): void {
  if (selectionStep.value === 'ready_to_publish') {
    stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
  }
  selectionStep.value = previousSeatSwapStep(selectionStep.value)
}

function jumpToStep(target: SeatSwapSelectionStep): void {
  const order: SeatSwapSelectionStep[] = ['select_current', 'select_desired', 'ready_to_publish']
  const currentIdx = order.indexOf(selectionStep.value)
  const targetIdx = order.indexOf(target)
  if (targetIdx >= currentIdx) return
  if (target === 'select_current') {
    stagedCurrentRegionKey.value = form.current_region_key || stagedCurrentRegionKey.value
  } else if (target === 'select_desired') {
    stagedDesiredSeats.value = form.desired_seats.map((s) => ({ ...s }))
  }
  selectionStep.value = target
}

function openPublishSheet(): void {
  if (currentView.value?.my_request) {
    fillFormFromMine()
  } else {
    resetFormForNewRequest()
  }
  publishSheetVisible.value = true
}

function openPublishSheetForCandidate(candidate: SeatSwapCandidate): void {
  resetFormForNewRequest()
  pendingConfirmTarget.value = candidate
  stagedDesiredSeats.value = [
    {
      region_key: candidate.current_region_key,
      region_name: candidate.current_region_name,
      desired_row: candidate.current_row || '',
      desired_seat_no: candidate.current_seat_no || '',
    },
  ]
  form.desired_seats = stagedDesiredSeats.value.map((seat) => ({ ...seat }))
  selectionStep.value = 'select_current'
  publishSheetVisible.value = true
}

function openManageSheet(): void {
  cancelReason.value = ''
  evidenceFileName.value = ''
  evidenceBase64.value = ''
  manageSheetVisible.value = true
}

function closeManageSheet(): void {
  manageSheetVisible.value = false
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

async function submitForm(): Promise<void> {
  const errors = validateSeatSwapForm(form)
  formErrors.value = errors
  if (hasSeatSwapFormErrors(errors)) return

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
    if (pendingConfirmTarget.value) {
      await confirmSeatSwapCandidate(pendingConfirmTarget.value.request_id)
      uni.showToast({ title: '已确认换座', icon: 'success' })
    } else {
      uni.showToast({ title: '发布成功', icon: 'success' })
    }
    publishSheetVisible.value = false
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
    resetFormForNewRequest()
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

async function cancelCandidateConfirmation(requestId: string): Promise<void> {
  try {
    await cancelSeatSwapCandidateConfirmation(requestId)
    uni.showToast({ title: '已取消匹配', icon: 'success' })
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '取消失败'), icon: 'none' })
  }
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

function chooseEvidence(): void {
  uni.chooseImage({
    count: 1,
    success: (result) => {
      const path = result.tempFilePaths[0]
      evidenceFileName.value = path.split('/').pop() || 'seat-swap-cancel.jpg'
      const fs = uni.getFileSystemManager()
      fs.readFile({
        filePath: path,
        encoding: 'base64',
        success: (readResult) => {
          evidenceBase64.value = String(readResult.data)
        },
        fail: () => {
          evidenceFileName.value = ''
          evidenceBase64.value = ''
          uni.showToast({ title: '截图读取失败', icon: 'none' })
        },
      })
    },
  })
}

async function submitMatchedCancel(): Promise<void> {
  const matchedId = currentView.value?.my_request?.request_id
  const target = currentView.value?.candidates.find((c) => c.status === 'matched')
  if (!matchedId || !target) {
    uni.showToast({ title: '暂无可撤销的匹配', icon: 'none' })
    return
  }
  if (!cancelReason.value.trim() || !evidenceBase64.value) {
    uni.showToast({ title: '请填写说明并上传截图', icon: 'none' })
    return
  }
  try {
    await cancelMatchedSeatSwap(target.request_id, {
      reason: cancelReason.value,
      evidence_file_name: evidenceFileName.value || 'seat-swap-cancel.jpg',
      evidence_content_type: evidenceContentType.value,
      evidence_base64: evidenceBase64.value,
    })
    uni.showToast({ title: '已提交撤销', icon: 'success' })
    manageSheetVisible.value = false
    await loadPage()
  } catch (err) {
    uni.showToast({ title: extractApiErrorMessage(err, '提交撤销失败'), icon: 'none' })
  }
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

onShow(() => {
  void loadPage()
})
</script>

<style scoped>
.page-root {
  position: relative;
  min-height: 100vh;
}

.page {
  position: relative;
  z-index: 1;
  min-height: 100vh;
  padding: 28rpx 24rpx 240rpx;
  box-sizing: border-box;
}

.state-card {
  margin-top: 22rpx;
  padding: 28rpx;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  border-radius: 28rpx;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 12rpx 26rpx rgba(26, 28, 36, 0.06);
  text-align: center;
}

.state-card--error {
  color: #b42318;
}

.hero-card {
  display: flex;
  align-items: center;
  gap: 18rpx;
  padding: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  border-radius: 28rpx;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 12rpx 26rpx rgba(26, 28, 36, 0.06);
  margin-bottom: 16rpx;
}

.hero-card__icon-box {
  flex-shrink: 0;
  width: 64rpx;
  height: 64rpx;
  border-radius: 18rpx;
  background: #f6f7fb;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
}

.hero-card__icon-mark {
  color: #4b515d;
  font-size: 36rpx;
  font-weight: 800;
  line-height: 1;
}

.hero-card__body {
  flex: 1;
  min-width: 0;
}

.eyebrow {
  display: block;
  color: #8f9198;
  font-size: 20rpx;
  font-weight: 400;
}

.hero-card__title {
  display: block;
  margin-top: 6rpx;
  color: #121212;
  font-size: 32rpx;
  line-height: 1.18;
  font-weight: 400;
}

.hero-card__summary {
  display: block;
  margin-top: 6rpx;
  color: #8f9198;
  font-size: 22rpx;
  line-height: 1.45;
}

.meta-pill {
  flex-shrink: 0;
  align-self: center;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 20rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #f6f7fb;
  color: #6d7280;
  font-size: 22rpx;
  font-weight: 400;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
  padding: 18rpx 22rpx;
  margin-bottom: 14rpx;
  border-radius: 24rpx;
  background: #f6f7fb;
  border: 1rpx dashed rgba(207, 211, 220, 0.95);
}

.info-row__text {
  flex: 1;
  color: #6d7280;
  font-size: 24rpx;
  line-height: 1.5;
}

.info-row__action {
  flex-shrink: 0;
  padding: 10rpx 20rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #fff;
  font-size: 22rpx;
  font-weight: 400;
}

.info-row__action::after {
  border: 0;
}

.stadium-wrap {
  margin: 0 -24rpx;
}

.seat-map-panel {
  position: sticky;
  top: 0;
  z-index: 5;
  margin: 4rpx -24rpx 0;
  padding: 10rpx 24rpx 6rpx;
  background: rgba(255, 255, 255, 0.97);
  box-shadow: 0 8rpx 20rpx rgba(26, 28, 36, 0.06);
}

.seat-map-panel__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 16rpx;
  padding: 0 4rpx 2rpx;
}

.seat-map-panel__title {
  color: #121212;
  font-size: 24rpx;
  font-weight: 500;
}

.seat-map-panel__hint {
  color: #8f9198;
  font-size: 20rpx;
}

.legend {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2rpx 4rpx 6rpx;
  color: #8f9198;
  font-size: 22rpx;
}

.legend__items {
  display: flex;
  align-items: center;
  gap: 22rpx;
}

.legend__item {
  display: flex;
  align-items: center;
  gap: 6rpx;
}

.legend__dot {
  display: inline-block;
  width: 16rpx;
  height: 16rpx;
  border-radius: 50%;
}

.legend__dot--current {
  background: #9aa0aa;
}

.legend__dot--desired {
  background: #1d8a55;
}

.legend__dot--hot {
  background: #15161b;
}

.legend__hint {
  color: #8f9198;
}

.section-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  padding: 20rpx 8rpx 12rpx;
}

.section-row__title {
  color: #121212;
  font-size: 30rpx;
  font-weight: 400;
}

.section-row__sub {
  color: #8f9198;
  font-size: 22rpx;
}

.group-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
  padding: 22rpx 12rpx;
  border-top: 1rpx solid rgba(232, 233, 238, 0.95);
}

.group-row:first-of-type {
  border-top: 0;
}

.group-row__main {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12rpx;
  flex-wrap: wrap;
}

.group-row__name {
  color: #121212;
  font-size: 28rpx;
  font-weight: 400;
}

.group-row__count {
  color: #8f9198;
  font-size: 22rpx;
}

.group-row__hit {
  font-size: 20rpx;
  padding: 4rpx 14rpx;
  border-radius: 999rpx;
  background: #eef8f2;
  color: #167348;
  border: 1rpx solid rgba(29, 138, 85, 0.3);
}

.group-row__caret {
  flex-shrink: 0;
  font-size: 22rpx;
  color: #9aa0aa;
  padding: 6rpx 4rpx;
  line-height: 1.2;
}

.filter-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18rpx 8rpx;
  margin-bottom: 8rpx;
  border-bottom: 1rpx solid rgba(232, 233, 238, 0.95);
}

.filter-row__main {
  display: flex;
  align-items: center;
  gap: 14rpx;
}

.filter-row__label {
  color: #121212;
  font-size: 28rpx;
  font-weight: 400;
}

.filter-row__count {
  color: #8f9198;
  font-size: 22rpx;
}

.filter-row__clear {
  flex-shrink: 0;
  background: #f6f7fb;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  padding: 8rpx 18rpx;
  border-radius: 999rpx;
  font-size: 22rpx;
  font-weight: 400;
}

.filter-row__clear::after {
  border: 0;
}

.empty-row {
  padding: 32rpx 12rpx;
  text-align: center;
  color: #8f9198;
  font-size: 24rpx;
}

.dock {
  position: fixed;
  left: 24rpx;
  right: 24rpx;
  bottom: calc(24rpx + var(--window-bottom));
  z-index: 8;
}

.dock-cta {
  width: 100%;
  padding: 26rpx 30rpx;
  border-radius: 28rpx;
  background: #15161b;
  color: #fff;
  font-size: 28rpx;
  font-weight: 400;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14rpx;
  box-shadow: 0 16rpx 36rpx rgba(21, 22, 27, 0.2);
  letter-spacing: 1rpx;
}

.dock-cta::after {
  border: 0;
}

.dock-cta__icon {
  flex-shrink: 0;
  display: inline-flex;
  width: 36rpx;
  height: 36rpx;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.18);
  align-items: center;
  justify-content: center;
  font-size: 26rpx;
  line-height: 1;
}

.dock-cta__label {
  font-size: 28rpx;
}

.dock-status {
  padding: 22rpx 26rpx;
  border-radius: 28rpx;
  background: #15161b;
  color: #fff;
  box-shadow: 0 16rpx 36rpx rgba(21, 22, 27, 0.32);
}

.dock-status__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12rpx;
}

.dock-status__label {
  display: flex;
  align-items: center;
  gap: 12rpx;
  font-size: 24rpx;
  font-weight: 400;
}

.dock-status__dot {
  width: 14rpx;
  height: 14rpx;
  border-radius: 50%;
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.18);
}

.dock-status__manage {
  font-size: 22rpx;
  font-weight: 400;
  color: rgba(255, 255, 255, 0.78);
  padding: 6rpx 16rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.12);
}

.dock-status__body {
  display: flex;
  align-items: center;
  gap: 12rpx;
  flex-wrap: wrap;
  font-size: 22rpx;
}

.dock-status__seat {
  padding: 6rpx 14rpx;
  border-radius: 12rpx;
  font-weight: 400;
}

.dock-status__seat--current {
  background: #f6f7fb;
  color: #4b515d;
}

.dock-status__seat--desired {
  background: #eef8f2;
  color: #167348;
}

.dock-status__arrow {
  color: rgba(255, 255, 255, 0.4);
}

.steps {
  display: flex;
  gap: 10rpx;
  margin: 0 0 18rpx;
}

.steps__item {
  flex: 1;
  min-width: 0;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 64rpx;
  padding: 12rpx 10rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 21rpx;
  font-weight: 400;
  line-height: 1.2;
  box-sizing: border-box;
}

.steps__item--done {
  background: #eef8f2;
  border-color: rgba(29, 138, 85, 0.3);
  color: #167348;
}

.steps__item--active {
  background: #15161b;
  color: #fff;
  border-color: #15161b;
}

.selected-tags {
  display: flex;
  gap: 10rpx;
  flex-wrap: wrap;
  margin: 18rpx 0 0;
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 8rpx 16rpx;
  border-radius: 999rpx;
  font-size: 22rpx;
  font-weight: 400;
  border: 1rpx solid transparent;
  line-height: 1;
}

.tag--current {
  background: #f6f7fb;
  color: #4b515d;
  border-color: rgba(232, 233, 238, 0.95);
}

.tag--desired {
  background: #eef8f2;
  color: #167348;
  border-color: rgba(29, 138, 85, 0.3);
}

.tag--empty {
  background: #ffffff;
  color: #8f9198;
  border-color: rgba(207, 211, 220, 0.95);
  border-style: dashed;
}

.row-input {
  display: flex;
  gap: 16rpx;
  margin-top: 12rpx;
}

.row-input--full {
  flex-direction: column;
}

.row-input__field {
  flex: 1;
}

.row-input__label {
  display: block;
  color: #6d7280;
  font-size: 22rpx;
  font-weight: 400;
  margin-bottom: 6rpx;
}

.input-box {
  width: 100%;
  height: 72rpx;
  padding: 0 18rpx;
  background: #ffffff;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  border-radius: 16rpx;
  font-size: 26rpx;
  font-weight: 400;
  color: #121212;
  box-sizing: border-box;
}

.input-box--short {
  width: 140rpx;
  flex-shrink: 0;
}

.textarea {
  width: 100%;
  min-height: 140rpx;
  padding: 14rpx 18rpx;
  background: #ffffff;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  border-radius: 16rpx;
  font-size: 26rpx;
  color: #121212;
  box-sizing: border-box;
}

.desired-rows {
  margin-top: 16rpx;
  display: flex;
  flex-direction: column;
  gap: 12rpx;
}

.desired-rows__item {
  display: flex;
  align-items: stretch;
  gap: 12rpx;
}

.desired-rows__name {
  flex: 1;
  height: 72rpx;
  padding: 0 18rpx;
  border-radius: 16rpx;
  background: #eef8f2;
  color: #167348;
  font-size: 24rpx;
  font-weight: 400;
  display: flex;
  align-items: center;
  box-sizing: border-box;
}

.field-error {
  display: block;
  margin-top: 12rpx;
  color: #b42318;
  font-size: 22rpx;
}

.summary-card {
  padding: 18rpx 22rpx;
  margin: 18rpx 0 0;
  border-radius: 16rpx;
  background: #ffffff;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
}

.summary-card__row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 16rpx;
  padding: 6rpx 0;
}

.summary-card__label {
  color: #8f9198;
  font-size: 22rpx;
}

.summary-card__value {
  color: #121212;
  font-size: 24rpx;
  font-weight: 400;
  text-align: right;
  flex: 1;
}

.btn-primary {
  flex: 1;
  padding: 20rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #fff;
  font-size: 28rpx;
  font-weight: 400;
  box-shadow: 0 8rpx 18rpx rgba(21, 22, 27, 0.12);
}

.btn-primary[disabled] {
  background: #c5c9d2;
  box-shadow: none;
}

.btn-primary--danger {
  background: #b42318;
}

.btn-primary::after {
  border: 0;
}

.btn-ghost {
  flex-shrink: 0;
  padding: 20rpx 28rpx;
  border-radius: 999rpx;
  background: #ffffff;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 26rpx;
  font-weight: 400;
}

.btn-ghost--danger {
  border-color: rgba(180, 35, 24, 0.3);
  color: #b42318;
  background: #fff7f6;
}

.btn-ghost::after {
  border: 0;
}

.ghost-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 16rpx;
  padding: 12rpx 22rpx;
  border-radius: 999rpx;
  background: #ffffff;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 24rpx;
}

.ghost-action::after {
  border: 0;
}

.sheet-actions {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 16rpx;
}

.sheet-actions .btn-primary:only-child {
  width: 100%;
  min-width: 0;
  flex: 1 1 auto;
}
</style>
