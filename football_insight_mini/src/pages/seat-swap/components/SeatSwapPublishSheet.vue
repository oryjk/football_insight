<template>
  <FiBottomSheet
    :visible="visible"
    :eyebrow="sheetEyebrow"
    :title="sheetTitle"
    height="76vh"
    compact-footer
    @update:visible="handleVisibleChange"
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
        <text
          v-if="stagedCurrentRegionName"
          class="tag tag--region"
          :class="regionTagClass(stagedCurrentRegionName)"
        >
          已选 · {{ stagedCurrentRegionName }}
        </text>
        <text v-else class="tag tag--empty">先点球场中的当前分区</text>
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
      <view class="seat-selection-group">
        <text class="seat-selection-group__title">当前座位</text>
        <view class="current-seat-card tag--region" :class="regionTagClass(form.current_region_name)">
          <text class="current-seat-card__region">{{ form.current_region_name }}</text>
          <text class="current-seat-card__meta">{{ form.current_row }}排</text>
          <text class="current-seat-card__meta">{{ form.current_seat_no }}号</text>
        </view>
      </view>
      <view class="selected-tags">
        <text v-if="!stagedDesiredSeats.length" class="tag tag--empty">先点球场中的目标分区(可多选)</text>
      </view>
      <view v-if="stagedDesiredSeats.length" class="desired-rows">
        <text class="seat-selection-group__title seat-selection-group__title--full">目标座位</text>
        <view
          v-for="seat in stagedDesiredSeats"
          :key="`desired-${seat.region_key}`"
          class="desired-rows__item tag--region"
          :class="regionTagClass(seat.region_name)"
        >
          <text class="desired-rows__name">{{ seat.region_name }}</text>
        </view>
      </view>
      <text v-if="formErrors.desired_seats" class="field-error">{{ formErrors.desired_seats }}</text>
    </template>

    <template v-else>
      <view class="seat-selection-group">
        <text class="seat-selection-group__title">当前座位</text>
        <view class="current-seat-card tag--region" :class="regionTagClass(form.current_region_name)">
          <text class="current-seat-card__region">{{ form.current_region_name }}</text>
          <text class="current-seat-card__meta">{{ form.current_row }}排</text>
          <text class="current-seat-card__meta">{{ form.current_seat_no }}号</text>
        </view>
      </view>
      <view class="desired-rows desired-rows--summary">
        <text class="seat-selection-group__title seat-selection-group__title--full">目标座位</text>
        <view
          v-for="seat in form.desired_seats"
          :key="`summary-desired-${seat.region_key}`"
          class="desired-rows__item tag--region"
          :class="regionTagClass(seat.region_name)"
        >
          <text class="desired-rows__name">{{ seat.region_name }}</text>
        </view>
      </view>
      <view class="row-input row-input--contact">
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
          @tap="submit"
        >{{ submitting ? '提交中...' : (myRequest ? '更新发布' : '发布换座') }}</button>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import StadiumMap from '../../../components/StadiumMap.vue'
import type { SeatSwapCandidate, SeatSwapRequest } from '../../../types/seatSwap'
import type { TicketWatchRegion } from '../../../types/ticketWatch'
import { resolveSeatSwapRegionColorGroup } from '../../../utils/stadiumRegions'
import {
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  hasSeatSwapFormErrors,
  previousSeatSwapStep,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  type SeatSwapFormErrors,
  type SeatSwapFormState,
  type SeatSwapSelectionStep,
} from '../helpers'

const props = defineProps<{
  visible: boolean
  regions: TicketWatchRegion[]
  myRequest: SeatSwapRequest | null
  presetCandidate: SeatSwapCandidate | null
  submitting: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'submit', payload: { form: SeatSwapFormState; presetCandidate: SeatSwapCandidate | null }): void
}>()

const selectionStep = ref<SeatSwapSelectionStep>('select_current')
const stagedCurrentRegionKey = ref('')
const stagedDesiredSeats = ref<SeatSwapFormState['desired_seats']>([])
const formErrors = ref<SeatSwapFormErrors>({})

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

const selectionStepIndex = computed(() => {
  if (selectionStep.value === 'select_current') return 1
  if (selectionStep.value === 'select_desired') return 2
  return 3
})

const sheetEyebrow = computed(() => `第 ${selectionStepIndex.value} 步 / 共 3 步`)

const sheetTitle = computed(() => {
  if (selectionStep.value === 'select_current') return '点选当前座位分区'
  if (selectionStep.value === 'select_desired') {
    return props.presetCandidate ? '已为你预填目标座位分区' : '点选目标座位分区'
  }
  return '补充联系方式并发布'
})

const sheetMapMode = computed<'select-current' | 'select-desired' | 'review'>(() => {
  if (selectionStep.value === 'select_current') return 'select-current'
  if (selectionStep.value === 'select_desired') return 'select-desired'
  return 'review'
})

const stagedCurrentRegionName = computed(() => findRegion(stagedCurrentRegionKey.value)?.block_name || '')
const stagedDesiredKeys = computed<string[]>(() => stagedDesiredSeats.value.map((s) => s.region_key).filter(Boolean))
const confirmedDesiredKeys = computed<string[]>(() => form.desired_seats.map((s) => s.region_key).filter(Boolean))

const canConfirmCurrentSelection = computed(() =>
  canConfirmCurrentSeatRegion(
    stagedCurrentRegionKey.value,
    form.current_row,
    form.current_seat_no,
  ),
)
const canConfirmDesiredSelection = computed(() => canConfirmDesiredSeatRegions(stagedDesiredSeats.value))

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      return
    }

    if (props.presetCandidate) {
      resetFormForNewRequest()
      stagedDesiredSeats.value = [
        {
          region_key: props.presetCandidate.current_region_key,
          region_name: props.presetCandidate.current_region_name,
          desired_row: props.presetCandidate.current_row || '',
          desired_seat_no: props.presetCandidate.current_seat_no || '',
        },
      ]
      form.desired_seats = stagedDesiredSeats.value.map((seat) => ({ ...seat }))
      selectionStep.value = 'select_current'
      return
    }

    if (props.myRequest) {
      fillFormFromMine()
    } else {
      resetFormForNewRequest()
    }
  },
)

function regionKey(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function findRegion(key: string): TicketWatchRegion | undefined {
  if (!key) return undefined
  return props.regions.find((r) => regionKey(r) === key)
}

function regionTagClass(regionName: string): string {
  return `tag--region-${resolveSeatSwapRegionColorGroup(regionName)}`
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
}

function fillFormFromMine(): void {
  const mine = props.myRequest
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
}

function handleVisibleChange(value: boolean): void {
  emit('update:visible', value)
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
    errors.current_region_key = '请选择当前座位分区'
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
    formErrors.value = { current_region_key: '请选择当前座位分区' }
    return
  }
  form.current_region_key = regionKey(region)
  form.current_region_name = region.block_name
  if (props.presetCandidate && stagedDesiredSeats.value.length) {
    form.desired_seats = stagedDesiredSeats.value.map((s) => ({ ...s }))
    selectionStep.value = 'ready_to_publish'
    return
  }
  selectionStep.value = 'select_desired'
}

function confirmDesiredSelection(): void {
  if (!canConfirmDesiredSeatRegions(stagedDesiredSeats.value)) {
    formErrors.value = { desired_seats: '请选择目标座位分区' }
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

function submit(): void {
  const errors = validateSeatSwapForm(form)
  formErrors.value = errors
  if (hasSeatSwapFormErrors(errors)) return

  emit('submit', {
    form: {
      current_region_key: form.current_region_key,
      current_region_name: form.current_region_name,
      current_row: form.current_row,
      current_seat_no: form.current_seat_no,
      wechat_id: form.wechat_id,
      phone_number: form.phone_number,
      desired_seats: form.desired_seats.map((s) => ({ ...s })),
    },
    presetCandidate: props.presetCandidate,
  })
}
</script>

<style scoped>
.steps {
  display: flex;
  gap: var(--fi-space-10);
  margin: 0 0 var(--fi-space-18);
}

.steps__item {
  flex: 1;
  min-width: 0;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 64rpx;
  padding: var(--fi-space-12) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  border: 1rpx solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: 400;
  line-height: var(--fi-leading-snug);
  box-sizing: border-box;
}

.steps__item--done {
  background: #eef8f2;
  border-color: rgba(29, 138, 85, 0.3);
  color: #167348;
}

.steps__item--active {
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  border-color: var(--fi-primitive-ink);
}

.selected-tags {
  display: flex;
  gap: var(--fi-space-10);
  flex-wrap: wrap;
  margin: var(--fi-space-18) 0 0;
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: var(--fi-space-8) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-22);
  font-weight: 400;
  border: 1rpx solid transparent;
  line-height: var(--fi-leading-none);
}

/* 区域色是 StadiumMap 业务色系，不属于视觉 token，保留字面量。 */
.tag--region {
  border-radius: 12rpx;
  border-color: transparent;
  color: var(--fi-primitive-white);
  font-weight: var(--fi-weight-black);
  box-shadow: 0 8rpx 18rpx rgba(18, 25, 20, 0.18);
}

.tag--region-blue { background: #336fbd; }
.tag--region-green { background: #46ab59; }
.tag--region-purple { background: #6c369b; }
.tag--region-yellow { background: #f4c23a; color: #17191f; }
.tag--region-navy { background: #0f215e; }
.tag--region-red { background: #ec3b20; }
.tag--region-vip { background: #b90000; }
.tag--region-muted { background: #d9dee7; color: #17191f; }

.tag--empty {
  background: #fff1f0;
  color: #c52018;
  border-color: rgba(226, 59, 46, 0.42);
  border-style: dashed;
}

.seat-selection-group {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  margin: var(--fi-space-18) 0 0;
}

.seat-selection-group__title {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: 400;
  line-height: var(--fi-leading-none);
}

.seat-selection-group__title--full {
  width: 100%;
}

.current-seat-card {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-10);
  min-height: 54rpx;
  padding: 0 var(--fi-space-16);
  border-radius: 14rpx;
  box-sizing: border-box;
}

.current-seat-card__region {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-black);
}

.current-seat-card__meta {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-black);
}

.row-input {
  display: flex;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-12);
}

.row-input--contact {
  margin-bottom: var(--fi-space-22);
}

.row-input__field {
  flex: 1;
}

.row-input__label {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: 400;
  margin-bottom: var(--fi-space-6);
}

.input-box {
  width: 100%;
  height: 72rpx;
  padding: 0 var(--fi-space-18);
  background: var(--fi-primitive-white);
  border: 1rpx solid var(--fi-color-border-chip);
  border-radius: 16rpx;
  font-size: var(--fi-font-26);
  font-weight: 400;
  color: var(--fi-color-text-strong);
  box-sizing: border-box;
}

.desired-rows {
  margin-top: var(--fi-space-12);
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-10);
}

.desired-rows--summary {
  margin-top: var(--fi-space-16);
}

.desired-rows__item {
  display: inline-flex;
  align-items: center;
  min-height: 54rpx;
  padding: 0 var(--fi-space-16);
  border-radius: 14rpx;
  box-sizing: border-box;
  box-shadow: 0 8rpx 18rpx rgba(18, 25, 20, 0.14);
}

.desired-rows__name {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-black);
}

.field-error {
  display: block;
  margin-top: var(--fi-space-12);
  color: #b42318;
  font-size: var(--fi-font-22);
}

.btn-primary {
  flex: 1;
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: 400;
  box-shadow: 0 8rpx 18rpx rgba(21, 22, 27, 0.12);
}

.btn-primary[disabled] {
  background: #c5c9d2;
  box-shadow: none;
}

.btn-primary::after {
  border: 0;
}

.btn-ghost {
  flex-shrink: 0;
  padding: var(--fi-space-20) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-white);
  border: 1rpx solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: 400;
}

.btn-ghost::after {
  border: 0;
}

.sheet-actions {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.sheet-actions .btn-primary:only-child {
  width: 100%;
  min-width: 0;
  flex: 1 1 auto;
}
</style>
