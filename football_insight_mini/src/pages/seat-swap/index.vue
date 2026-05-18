<template>
  <view class="page">
    <view class="hero-card">
      <view>
        <text class="eyebrow">Seat Swap</text>
        <text class="hero-title">当前比赛换座</text>
        <text class="hero-copy">只撮合成都蓉城当前比赛，双向匹配后再交换联系方式。</text>
      </view>
      <text class="hero-badge">{{ currentView?.available ? '开放中' : '待比赛' }}</text>
    </view>

    <view v-if="loading" class="state-card">
      <text>正在加载换座池...</text>
    </view>

    <view v-else-if="errorMessage" class="state-card state-card--error">
      <text>{{ errorMessage }}</text>
      <button class="ghost-action" @tap="loadPage">重试</button>
    </view>

    <template v-else>
      <view class="panel match-panel">
        <view class="section-heading">
          <view>
            <text class="section-kicker">当前比赛</text>
            <text class="section-title">{{ matchTitle }}</text>
          </view>
          <text class="meta-pill">{{ currentView?.candidates.length || 0 }} 条意向</text>
        </view>
        <text class="panel-copy">{{ matchCopy }}</text>
      </view>

      <view v-if="!isLoggedIn" class="panel login-panel">
        <text class="section-title">登录后发布我的座位</text>
        <text class="panel-copy">未登录可以浏览脱敏意向；发布、确认和查看联系方式需要登录。</text>
        <button class="primary-action" @tap="goToUserPage">去登录</button>
      </view>

      <view v-if="currentView?.available && isLoggedIn" class="panel form-panel">
        <view class="section-heading">
          <view>
            <text class="section-kicker">我的请求</text>
            <text class="section-title">{{ currentView.my_request ? '更新换座意向' : '发布换座意向' }}</text>
          </view>
          <text v-if="currentView.my_request" class="meta-pill">已发布</text>
        </view>

        <view class="field">
          <text class="field-label">当前分区</text>
          <picker :range="regionNames" :value="currentRegionIndex" @change="selectCurrentRegion">
            <view class="picker-box">{{ form.current_region_name || '请选择当前分区' }}</view>
          </picker>
          <text v-if="formErrors.current_region_key" class="field-error">{{ formErrors.current_region_key }}</text>
        </view>

        <view class="field-row">
          <view class="field field--half">
            <text class="field-label">当前排</text>
            <input v-model="form.current_row" class="input" placeholder="如 8" />
            <text v-if="formErrors.current_row" class="field-error">{{ formErrors.current_row }}</text>
          </view>
          <view class="field field--half">
            <text class="field-label">当前号</text>
            <input v-model="form.current_seat_no" class="input" placeholder="如 15" />
            <text v-if="formErrors.current_seat_no" class="field-error">{{ formErrors.current_seat_no }}</text>
          </view>
        </view>

        <view class="field">
          <view class="field-head">
            <text class="field-label">想换到</text>
            <button class="mini-action" @tap="addDesiredSeat">添加</button>
          </view>
          <view
            v-for="(seat, index) in form.desired_seats"
            :key="`desired-${index}`"
            class="desired-row"
          >
            <picker :range="regionNames" :value="desiredRegionIndex(seat.region_key)" :data-index="index" @change="selectDesiredRegion">
              <view class="picker-box picker-box--compact">{{ seat.region_name || '目标分区' }}</view>
            </picker>
            <input v-model="seat.desired_row" class="input input--short" placeholder="排(选填)" />
            <input v-model="seat.desired_seat_no" class="input input--short" placeholder="号(选填)" />
            <button class="icon-action" @tap="removeDesiredSeat(index)">×</button>
          </view>
          <text v-if="formErrors.desired_seats" class="field-error">{{ formErrors.desired_seats }}</text>
        </view>

        <view class="field-row">
          <view class="field field--half">
            <text class="field-label">微信号</text>
            <input v-model="form.wechat_id" class="input" placeholder="至少填一项" />
          </view>
          <view class="field field--half">
            <text class="field-label">手机号</text>
            <input v-model="form.phone_number" class="input" type="number" placeholder="至少填一项" />
          </view>
        </view>
        <text v-if="formErrors.contact" class="field-error">{{ formErrors.contact }}</text>
        <text v-if="formErrors.phone_number" class="field-error">{{ formErrors.phone_number }}</text>

        <view class="form-actions">
          <button class="primary-action" :disabled="submitting" @tap="submitForm">
            {{ submitting ? '提交中...' : '保存换座请求' }}
          </button>
          <button v-if="currentView.my_request?.status === 'active'" class="ghost-action" @tap="deleteRequest">撤销</button>
        </view>
      </view>

      <view v-if="currentView?.my_request?.status === 'matched'" class="panel cancel-panel">
        <text class="section-title">已匹配成功</text>
        <text class="panel-copy">如线下协商取消，需要填写说明并上传双方达成一致的截图。</text>
        <textarea v-model="cancelReason" class="textarea" placeholder="撤销说明" />
        <button class="ghost-action" @tap="chooseEvidence">选择截图</button>
        <text v-if="evidenceFileName" class="evidence-name">{{ evidenceFileName }}</text>
        <button class="danger-action" @tap="submitMatchedCancel">提交撤销申请</button>
      </view>

      <view class="panel">
        <view class="section-heading">
          <view>
            <text class="section-kicker">换座池</text>
            <text class="section-title">当前候选</text>
          </view>
          <text class="meta-pill">{{ currentView?.candidates.length || 0 }}</text>
        </view>

        <view v-if="!currentView?.candidates.length" class="empty-copy">
          <text>当前还没有其他换座意向。</text>
        </view>

        <view
          v-for="candidate in currentView?.candidates"
          :key="candidate.request_id"
          class="candidate-card"
          :class="{ 'candidate-card--hot': candidate.status !== 'display_only' }"
        >
          <view class="candidate-card__head">
            <view>
              <text class="candidate-name">{{ candidate.display_name }}</text>
              <text class="candidate-seat">{{ formatSeatLabel(candidate) }}</text>
            </view>
            <text class="status-pill">{{ statusLabel(candidate.status) }}</text>
          </view>
          <text class="candidate-wants">想换：{{ desiredSeatText(candidate.desired_seats) }}</text>

          <view v-if="candidate.contact" class="contact-box">
            <text v-if="candidate.contact.wechat_id">微信：{{ candidate.contact.wechat_id }}</text>
            <text v-if="candidate.contact.phone_number">手机：{{ candidate.contact.phone_number }}</text>
          </view>

          <button
            v-if="candidate.status !== 'display_only' && candidate.status !== 'matched' && isLoggedIn"
            class="primary-action primary-action--small"
            @tap="confirmCandidate(candidate.request_id)"
          >
            确认换座
          </button>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { onShow } from '@dcloudio/uni-app'
import {
  confirmSeatSwapCandidate,
  deleteMySeatSwapRequest,
  getCurrentSeatSwap,
  upsertMySeatSwapRequest,
  cancelMatchedSeatSwap,
} from '../../api/seatSwap'
import { getTicketWatchRegions } from '../../api/ticketWatch'
import type { SeatSwapCurrentResponse, SeatSwapDesiredSeat } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { getAccessToken } from '../../utils/authStorage'
import { extractApiErrorMessage } from '../../utils/apiError'
import {
  formatSeatLabel,
  hasSeatSwapFormErrors,
  statusLabel,
  validateSeatSwapForm,
  type SeatSwapFormErrors,
  type SeatSwapFormState,
} from './helpers'

const loading = ref(true)
const submitting = ref(false)
const errorMessage = ref('')
const currentView = ref<SeatSwapCurrentResponse | null>(null)
const regions = ref<TicketWatchRegion[]>([])
const isLoggedIn = ref(false)
const formErrors = ref<SeatSwapFormErrors>({})
const cancelReason = ref('')
const evidenceFileName = ref('')
const evidenceBase64 = ref('')
const evidenceContentType = ref('image/jpeg')

interface PickerChangeEvent {
  detail: {
    value: number | string
  }
  currentTarget?: {
    dataset?: Record<string, unknown>
  }
}

const form = reactive<SeatSwapFormState>({
  current_region_key: '',
  current_region_name: '',
  current_row: '',
  current_seat_no: '',
  wechat_id: '',
  phone_number: '',
  desired_seats: [
    {
      region_key: '',
      region_name: '',
      desired_row: '',
      desired_seat_no: '',
    },
  ],
})

const regionNames = computed(() => regions.value.map((region) => region.block_name))
const currentRegionIndex = computed(() =>
  Math.max(0, regions.value.findIndex((region) => (region.block_key || region.block_name) === form.current_region_key)),
)
const matchTitle = computed(() => {
  const match = currentView.value?.current_match
  return match ? `${match.home_team_name} VS ${match.away_team_name}` : '暂无当前比赛'
})
const matchCopy = computed(() => {
  if (!currentView.value?.available) {
    return '换座撮合只在成都蓉城当前比赛开放。'
  }
  return currentView.value.current_match?.kickoff_at || '当前比赛换座池已开放'
})

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function fillFormFromMine(): void {
  const mine = currentView.value?.my_request
  if (!mine) {
    return
  }
  form.current_region_key = mine.current_region_key
  form.current_region_name = mine.current_region_name
  form.current_row = mine.current_row
  form.current_seat_no = mine.current_seat_no
  form.wechat_id = mine.contact?.wechat_id || ''
  form.phone_number = mine.contact?.phone_number || ''
  form.desired_seats = mine.desired_seats.map((seat) => ({
    region_key: seat.region_key,
    region_name: seat.region_name,
    desired_row: seat.desired_row || '',
    desired_seat_no: seat.desired_seat_no || '',
  }))
}

async function loadPage(): Promise<void> {
  loading.value = true
  errorMessage.value = ''
  isLoggedIn.value = !!getAccessToken()
  try {
    const [view, regionList] = await Promise.all([
      getCurrentSeatSwap(),
      getTicketWatchRegions(),
    ])
    currentView.value = view
    regions.value = regionList
    fillFormFromMine()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '换座池加载失败')
  } finally {
    loading.value = false
  }
}

function selectCurrentRegion(event: PickerChangeEvent): void {
  const region = regions.value[Number(event.detail.value)]
  if (!region) return
  form.current_region_key = regionKey(region)
  form.current_region_name = region.block_name
}

function desiredRegionIndex(key: string): number {
  return Math.max(0, regions.value.findIndex((region) => regionKey(region) === key))
}

function selectDesiredRegion(event: PickerChangeEvent): void {
  const index = Number(event.currentTarget?.dataset?.index)
  const region = regions.value[Number(event.detail.value)]
  if (!region || !form.desired_seats[index]) return
  form.desired_seats[index].region_key = regionKey(region)
  form.desired_seats[index].region_name = region.block_name
}

function addDesiredSeat(): void {
  form.desired_seats.push({
    region_key: '',
    region_name: '',
    desired_row: '',
    desired_seat_no: '',
  })
}

function removeDesiredSeat(index: number): void {
  if (form.desired_seats.length <= 1) return
  form.desired_seats.splice(index, 1)
}

function desiredSeatText(seats: SeatSwapDesiredSeat[]): string {
  return seats
    .map((seat) => {
      const extra = [seat.desired_row ? `${seat.desired_row}排` : '', seat.desired_seat_no ? `${seat.desired_seat_no}号` : '']
        .filter(Boolean)
        .join(' ')
      return extra ? `${seat.region_name} ${extra}` : seat.region_name
    })
    .join('、')
}

async function submitForm(): Promise<void> {
  const errors = validateSeatSwapForm(form)
  formErrors.value = errors
  if (hasSeatSwapFormErrors(errors)) {
    return
  }

  submitting.value = true
  try {
    await upsertMySeatSwapRequest({
      current_region_key: form.current_region_key,
      current_region_name: form.current_region_name,
      current_row: form.current_row,
      current_seat_no: form.current_seat_no,
      wechat_id: form.wechat_id || null,
      phone_number: form.phone_number || null,
      desired_seats: form.desired_seats.map((seat) => ({
        region_key: seat.region_key,
        region_name: seat.region_name,
        desired_row: seat.desired_row || null,
        desired_seat_no: seat.desired_seat_no || null,
      })),
    })
    uni.showToast({ title: '已保存', icon: 'success' })
    await loadPage()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '保存失败'), icon: 'none' })
  } finally {
    submitting.value = false
  }
}

async function deleteRequest(): Promise<void> {
  try {
    await deleteMySeatSwapRequest()
    uni.showToast({ title: '已撤销', icon: 'success' })
    await loadPage()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '撤销失败'), icon: 'none' })
  }
}

async function confirmCandidate(requestId: string): Promise<void> {
  try {
    await confirmSeatSwapCandidate(requestId)
    uni.showToast({ title: '已确认', icon: 'success' })
    await loadPage()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '确认失败'), icon: 'none' })
  }
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
  const target = currentView.value?.candidates.find((candidate) => candidate.status === 'matched')
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
    await loadPage()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '提交撤销失败'), icon: 'none' })
  }
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

onShow(() => {
  loadPage()
})
</script>

<style scoped>
.page {
  min-height: 100vh;
  padding: 28rpx 24rpx 140rpx;
  background: #f6f5f2;
}

.hero-card,
.panel,
.state-card {
  border: 1rpx solid rgba(20, 22, 28, 0.08);
  border-radius: 32rpx;
  background: rgba(255, 255, 255, 0.94);
  box-shadow: 0 18rpx 42rpx rgba(30, 30, 36, 0.08);
}

.hero-card {
  display: flex;
  justify-content: space-between;
  gap: 24rpx;
  padding: 34rpx;
}

.eyebrow,
.section-kicker {
  display: block;
  color: #9297a3;
  font-size: 24rpx;
  font-weight: 800;
  letter-spacing: 0;
}

.hero-title {
  display: block;
  margin-top: 10rpx;
  color: #17191f;
  font-size: 48rpx;
  font-weight: 900;
}

.hero-copy,
.panel-copy,
.empty-copy {
  display: block;
  margin-top: 14rpx;
  color: #6f7480;
  font-size: 28rpx;
  line-height: 1.65;
}

.hero-badge,
.meta-pill,
.status-pill {
  align-self: flex-start;
  border-radius: 999rpx;
  padding: 10rpx 18rpx;
  background: #f3ead9;
  color: #9b7a42;
  font-size: 24rpx;
  font-weight: 800;
}

.panel,
.state-card {
  margin-top: 22rpx;
  padding: 28rpx;
}

.state-card--error {
  color: #b42318;
}

.section-heading,
.field-head,
.candidate-card__head,
.form-actions,
.field-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 18rpx;
}

.section-title {
  display: block;
  color: #17191f;
  font-size: 34rpx;
  font-weight: 900;
}

.field {
  margin-top: 22rpx;
}

.field--half {
  flex: 1;
}

.field-label {
  display: block;
  margin-bottom: 10rpx;
  color: #505663;
  font-size: 24rpx;
  font-weight: 800;
}

.picker-box,
.input,
.textarea {
  min-height: 78rpx;
  border: 1rpx solid #e4e6ec;
  border-radius: 18rpx;
  padding: 18rpx 20rpx;
  background: #fbfbfd;
  color: #17191f;
  font-size: 28rpx;
}

.picker-box--compact {
  min-width: 190rpx;
}

.input--short {
  width: 150rpx;
}

.textarea {
  width: 100%;
  height: 150rpx;
  box-sizing: border-box;
}

.field-error {
  display: block;
  margin-top: 8rpx;
  color: #b42318;
  font-size: 24rpx;
}

.desired-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
  margin-top: 12rpx;
}

.primary-action,
.ghost-action,
.danger-action,
.mini-action,
.icon-action {
  border-radius: 999rpx;
  padding: 18rpx 26rpx;
  font-size: 26rpx;
  font-weight: 900;
}

.primary-action {
  background: #15161b;
  color: #fff;
}

.primary-action--small {
  margin-top: 18rpx;
  padding: 14rpx 22rpx;
  font-size: 24rpx;
}

.ghost-action,
.mini-action,
.icon-action {
  background: #eef0f4;
  color: #2f333b;
}

.danger-action {
  margin-top: 16rpx;
  background: #b42318;
  color: #fff;
}

.candidate-card {
  margin-top: 18rpx;
  border-radius: 24rpx;
  padding: 22rpx;
  background: #f7f8fa;
}

.candidate-card--hot {
  background: #fff8ea;
}

.candidate-name {
  display: block;
  color: #17191f;
  font-size: 30rpx;
  font-weight: 900;
}

.candidate-seat,
.candidate-wants,
.contact-box,
.evidence-name {
  display: block;
  margin-top: 8rpx;
  color: #687080;
  font-size: 25rpx;
  line-height: 1.5;
}

.contact-box {
  border-radius: 18rpx;
  padding: 16rpx;
  background: rgba(255, 255, 255, 0.78);
}
</style>
