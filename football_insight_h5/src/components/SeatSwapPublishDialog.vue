<template>
  <div v-if="visible" class="dialog">
    <div class="dialog__scrim" @click="handleClose"></div>
    <div class="dialog__panel">
      <div class="dialog__head">
        <div class="dialog__head-main">
          <span class="dialog__kicker">{{ presetCandidate ? '确认换座' : (isEdit ? '编辑发布' : '发布换座') }}</span>
          <span class="dialog__title">{{ stepTitle }}</span>
        </div>
        <button type="button" class="dialog__close" @click="handleClose">✕</button>
      </div>

      <div v-if="presetCandidate" class="dialog__preset">
        发布后与 <span class="dialog__preset-name">{{ presetCandidate.display_name }}</span>
        （{{ presetCandidate.current_region_name }} {{ presetCandidate.current_row }}排 {{ presetCandidate.current_seat_no }}号）确认换座
      </div>

      <div class="dialog__steps">
        <span
          v-for="(item, index) in stepLabels"
          :key="item"
          class="dialog__step"
          :class="{ 'dialog__step--active': stepIndex === index, 'dialog__step--done': stepIndex > index }"
        >
          {{ index + 1 }}. {{ item }}
        </span>
      </div>

      <div class="dialog__body">
        <template v-if="step === 'select_current'">
          <StadiumMap
            mode="select-current"
            :regions="regions"
            :staged-current-key="form.current_region_key"
            hint="分区示意"
          />
          <RegionChipGrid
            mode="select-current"
            :regions="regions"
            :staged-current-key="form.current_region_key"
            hint="点选你球票所在的当前分区"
            @region-tap="handleCurrentRegionTap"
          />
          <div class="dialog__fields">
            <div class="dialog__field">
              <span class="dialog__label">当前分区</span>
              <div class="dialog__region-value">
                <span :class="form.current_region_key ? 'dialog__region-text' : 'dialog__region-text dialog__region-text--empty'">
                  {{ form.current_region_name || '在上方分区列表中点选' }}
                </span>
              </div>
            </div>
            <div class="dialog__field">
              <span class="dialog__label">排号</span>
              <input v-model="form.current_row" class="dialog__input" type="number" placeholder="如 8" />
            </div>
            <div class="dialog__field">
              <span class="dialog__label">座号</span>
              <input v-model="form.current_seat_no" class="dialog__input" type="number" placeholder="如 15" />
            </div>
          </div>
        </template>

        <template v-else-if="step === 'select_desired'">
          <StadiumMap
            mode="select-desired"
            :regions="regions"
            :staged-current-key="form.current_region_key"
            :staged-desired-keys="desiredKeys"
            hint="分区示意"
          />
          <RegionChipGrid
            mode="select-desired"
            :regions="regions"
            :staged-current-key="form.current_region_key"
            :staged-desired-keys="desiredKeys"
            hint="点选想换到的分区，可多选"
            @region-tap="handleDesiredRegionTap"
          />
          <div v-if="form.desired_seats.length" class="dialog__desired-list">
            <div v-for="seat in form.desired_seats" :key="seat.region_key" class="dialog__desired-item">
              <span class="dialog__desired-name">{{ seat.region_name }}</span>
              <input v-model="seat.desired_row" class="dialog__input dialog__input--small" type="number" placeholder="排（可选）" />
              <input v-model="seat.desired_seat_no" class="dialog__input dialog__input--small" type="number" placeholder="号（可选）" />
              <button type="button" class="dialog__desired-remove" @click="removeDesired(seat.region_key)">✕</button>
            </div>
          </div>
          <span v-else class="dialog__desired-empty">还没选目标分区，在上方分区列表中点选（可多选）</span>
        </template>

        <template v-else>
          <div class="dialog__review">
            <div class="dialog__review-row">
              <span class="dialog__review-label">当前座位</span>
              <span class="dialog__review-seat dialog__review-seat--current">
                {{ form.current_region_name }} {{ form.current_row }}排 {{ form.current_seat_no }}号
              </span>
            </div>
            <div class="dialog__review-row">
              <span class="dialog__review-label">目标座位</span>
              <div class="dialog__review-desired">
                <span v-for="seat in form.desired_seats" :key="seat.region_key" class="dialog__review-seat dialog__review-seat--desired">
                  {{ formatDesiredSeat(seat) }}
                </span>
              </div>
            </div>
          </div>
          <div class="dialog__fields dialog__fields--contact">
            <div class="dialog__field">
              <span class="dialog__label">微信号</span>
              <input v-model="form.wechat_id" class="dialog__input" type="text" placeholder="选填，至少填一种联系方式" />
            </div>
            <div class="dialog__field">
              <span class="dialog__label">手机号</span>
              <input v-model="form.phone_number" class="dialog__input" type="number" placeholder="选填，11 位手机号" />
            </div>
          </div>
          <span class="dialog__contact-hint">联系方式仅对确认换座的对方可见</span>
        </template>

        <span v-if="errorText" class="dialog__error">{{ errorText }}</span>
      </div>

      <div class="dialog__footer">
        <button v-if="step !== 'select_current'" type="button" class="dialog__btn dialog__btn--ghost" @click="goPrev">
          上一步
        </button>
        <button
          v-if="step !== 'ready_to_publish'"
          type="button"
          class="dialog__btn dialog__btn--primary"
          :class="{ 'dialog__btn--disabled': !canGoNext }"
          @click="goNext"
        >
          下一步
        </button>
        <button
          v-else
          type="button"
          class="dialog__btn dialog__btn--primary"
          :class="{ 'dialog__btn--disabled': submitting }"
          @click="handleSubmit"
        >
          {{ submitting ? '提交中…' : submitText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import StadiumMap from './StadiumMap.vue'
import RegionChipGrid from './RegionChipGrid.vue'
import type { TicketWatchRegion } from '../types/ticketWatch'
import type { SeatSwapCandidate, SeatSwapRequest } from '../types/seatSwap'
import {
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  hasSeatSwapFormErrors,
  previousSeatSwapStep,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  type SeatSwapDesiredSeatFormState,
  type SeatSwapFormState,
  type SeatSwapSelectionStep,
} from '../utils/seatSwap'

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

const stepLabels = ['当前座位', '目标座位', '联系方式']
const step = ref<SeatSwapSelectionStep>('select_current')
const errorText = ref('')

const emptyForm = (): SeatSwapFormState => ({
  current_region_key: '',
  current_region_name: '',
  current_row: '',
  current_seat_no: '',
  wechat_id: '',
  phone_number: '',
  desired_seats: [],
})

const form = ref<SeatSwapFormState>(emptyForm())

const isEdit = computed(() => Boolean(props.myRequest))
const stepIndex = computed(() => (step.value === 'select_current' ? 0 : step.value === 'select_desired' ? 1 : 2))
const stepTitle = computed(() => {
  if (step.value === 'select_current') return '点选你的当前座位'
  if (step.value === 'select_desired') return '点选想换到的分区'
  return '留下联系方式并确认'
})
const submitText = computed(() => {
  if (props.presetCandidate) return '发布并确认换座'
  return isEdit.value ? '保存修改' : '确认发布'
})

const desiredKeys = computed(() => form.value.desired_seats.map((seat) => seat.region_key))

const canGoNext = computed(() => {
  if (step.value === 'select_current') {
    return canConfirmCurrentSeatRegion(form.value.current_region_key, form.value.current_row, form.value.current_seat_no)
  }
  if (step.value === 'select_desired') {
    return canConfirmDesiredSeatRegions(form.value.desired_seats)
  }
  return true
})

watch(
  () => props.visible,
  (visible) => {
    if (!visible) return
    errorText.value = ''
    step.value = 'select_current'
    if (props.myRequest) {
      form.value = {
        current_region_key: props.myRequest.current_region_key,
        current_region_name: props.myRequest.current_region_name,
        current_row: props.myRequest.current_row,
        current_seat_no: props.myRequest.current_seat_no,
        wechat_id: props.myRequest.contact?.wechat_id || '',
        phone_number: props.myRequest.contact?.phone_number || '',
        desired_seats: props.myRequest.desired_seats.map((seat) => ({
          region_key: seat.region_key,
          region_name: seat.region_name,
          desired_row: seat.desired_row || '',
          desired_seat_no: seat.desired_seat_no || '',
        })),
      }
    } else {
      form.value = emptyForm()
    }
  },
)

function regionKeyOf(region: TicketWatchRegion): string {
  return region.block_key || region.block_name
}

function handleCurrentRegionTap(key: string): void {
  const region = props.regions.find((item) => regionKeyOf(item) === key)
  if (!region) return
  form.value.current_region_key = key
  form.value.current_region_name = region.block_name
  errorText.value = ''
}

function handleDesiredRegionTap(key: string): void {
  const region = props.regions.find((item) => regionKeyOf(item) === key)
  if (!region) return
  form.value.desired_seats = toggleDesiredSeatRegion(form.value.desired_seats, {
    region_key: key,
    region_name: region.block_name,
  })
  errorText.value = ''
}

function removeDesired(key: string): void {
  form.value.desired_seats = form.value.desired_seats.filter((seat) => seat.region_key !== key)
}

function formatDesiredSeat(seat: SeatSwapDesiredSeatFormState): string {
  const row = seat.desired_row.trim()
  const seatNo = seat.desired_seat_no.trim()
  if (!row && !seatNo) return seat.region_name
  return `${seat.region_name} ${row ? `${row}排` : ''}${seatNo ? `${seatNo}号` : ''}`
}

function goPrev(): void {
  errorText.value = ''
  step.value = previousSeatSwapStep(step.value)
}

function goNext(): void {
  if (!canGoNext.value) {
    errorText.value = step.value === 'select_current' ? '请先点选当前分区并填写排号、座号' : '请至少选择一个目标分区'
    return
  }
  errorText.value = ''
  if (step.value === 'select_current') {
    step.value = 'select_desired'
  } else if (step.value === 'select_desired') {
    step.value = 'ready_to_publish'
  }
}

function handleClose(): void {
  if (props.submitting) return
  emit('update:visible', false)
}

function handleSubmit(): void {
  const errors = validateSeatSwapForm(form.value)
  if (hasSeatSwapFormErrors(errors)) {
    errorText.value = errors.contact || errors.phone_number || errors.desired_seats
      || errors.current_region_key || errors.current_row || errors.current_seat_no || '请检查填写内容'
    return
  }
  errorText.value = ''
  emit('submit', { form: form.value, presetCandidate: props.presetCandidate })
}
</script>

<style scoped>
.dialog {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.dialog__scrim {
  position: absolute;
  inset: 0;
  background: rgba(15, 17, 22, 0.46);
  backdrop-filter: blur(2px);
}

.dialog__panel {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 660px;
  max-height: 86vh;
  border-radius: 14px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: linear-gradient(180deg, rgba(255, 251, 242, 1), rgba(255, 255, 255, 0.99));
  box-shadow: 0 16px 36px rgba(12, 14, 20, 0.24);
  overflow: hidden;
}

.dialog__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 22px 8px;
}

.dialog__head-main {
  display: grid;
  gap: 3px;
}

.dialog__kicker {
  color: #8f7c5f;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1.5px;
}

.dialog__title {
  color: #17181c;
  font-size: 18px;
  font-weight: 800;
}

.dialog__close {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #9c855c;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.dialog__preset {
  margin: 0 22px 4px;
  padding: 7px 10px;
  border-radius: 8px;
  background: #eef8f2;
  border: 1px solid rgba(29, 138, 85, 0.3);
  color: #175c31;
  font-size: 12px;
  line-height: 1.5;
}

.dialog__preset-name {
  font-weight: 700;
}

.dialog__steps {
  display: flex;
  gap: 8px;
  padding: 8px 22px 12px;
}

.dialog__step {
  flex: 1;
  text-align: center;
  padding: 5px 4px;
  border-radius: 999px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: rgba(255, 251, 242, 0.7);
  color: #9c855c;
  font-size: 11px;
  white-space: nowrap;
}

.dialog__step--active {
  background: linear-gradient(180deg, #20242c, #191d26);
  border-color: #20242c;
  color: #fff;
  font-weight: 700;
}

.dialog__step--done {
  color: #167348;
  border-color: rgba(29, 138, 85, 0.3);
  background: #eef8f2;
}

.dialog__body {
  flex: 1;
  min-height: 0;
  padding: 0 22px 8px;
  box-sizing: border-box;
  overflow-y: auto;
  display: grid;
  gap: 12px;
  align-content: start;
}

.dialog__fields {
  display: flex;
  gap: 12px;
  margin-top: 14px;
}

.dialog__fields--contact {
  margin-top: 0;
}

.dialog__field {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 4px;
}

.dialog__label {
  color: #8f7c5f;
  font-size: 11px;
}

.dialog__input {
  width: 100%;
  height: 34px;
  padding: 0 11px;
  border-radius: 8px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: rgba(255, 251, 242, 0.5);
  color: #17181c;
  font-size: 13px;
  box-sizing: border-box;
  outline: none;
}

.dialog__input::placeholder {
  color: #988f84;
  font-size: 12px;
}

.dialog__input--small {
  width: 80px;
  flex-shrink: 0;
}

.dialog__region-value {
  height: 34px;
  padding: 0 11px;
  border-radius: 8px;
  border: 1px dashed rgba(230, 220, 198, 0.92);
  background: rgba(255, 251, 242, 0.4);
  display: flex;
  align-items: center;
  box-sizing: border-box;
}

.dialog__region-text {
  color: #17181c;
  font-size: 13px;
  font-weight: 700;
}

.dialog__region-text--empty {
  color: #988f84;
  font-weight: 400;
  font-size: 12px;
}

.dialog__desired-list {
  display: grid;
  gap: 8px;
  margin-top: 14px;
}

.dialog__desired-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 9px;
  border-radius: 8px;
  border: 1px solid rgba(29, 138, 85, 0.3);
  background: #eef8f2;
}

.dialog__desired-name {
  flex: 1;
  min-width: 0;
  color: #175c31;
  font-size: 13px;
  font-weight: 700;
}

.dialog__desired-remove {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(29, 138, 85, 0.24);
  color: #167348;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.dialog__desired-empty {
  display: block;
  margin-top: 14px;
  padding: 12px;
  border-radius: 8px;
  border: 1px dashed rgba(230, 220, 198, 0.92);
  color: #988f84;
  font-size: 12px;
  text-align: center;
}

.dialog__review {
  display: grid;
  gap: 10px;
  margin-bottom: 14px;
  padding: 9px 10px;
  border-radius: 8px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: rgba(255, 255, 255, 0.8);
}

.dialog__review-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dialog__review-label {
  flex-shrink: 0;
  width: 64px;
  color: #988f84;
  font-size: 12px;
}

.dialog__review-desired {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.dialog__review-seat {
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 700;
}

.dialog__review-seat--current {
  background: linear-gradient(180deg, #f7efe1, #f1e3ca);
  border: 1px solid rgba(220, 201, 165, 0.6);
  color: #927445;
}

.dialog__review-seat--desired {
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  border: 1px solid rgba(29, 138, 85, 0.3);
  color: #167348;
}

.dialog__contact-hint {
  display: block;
  margin-top: 8px;
  color: #988f84;
  font-size: 11px;
}

.dialog__error {
  display: block;
  margin-top: 12px;
  color: #dd524d;
  font-size: 12px;
}

.dialog__footer {
  display: flex;
  gap: 12px;
  padding: 14px 22px 18px;
  border-top: 1px solid rgba(238, 233, 224, 0.95);
}

.dialog__btn {
  flex: 1;
  padding: 9px 0;
  border-radius: 999px;
  border: 0;
  font-size: 14px;
  font-weight: 700;
  text-align: center;
  cursor: pointer;
}

.dialog__btn--primary {
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  box-shadow: 0 4px 10px rgba(21, 22, 27, 0.16);
}

.dialog__btn--ghost {
  flex: 0 0 90px;
  background: #fff;
  border: 1px solid rgba(32, 36, 44, 0.16);
  color: #5b5f6a;
}

.dialog__btn--disabled {
  opacity: 0.45;
}

.dialog__btn:active {
  transform: scale(0.98);
}

@media (max-width: 768px) {
  .dialog {
    padding: 0;
    align-items: flex-end;
  }

  .dialog__panel {
    max-width: none;
    max-height: 92vh;
    border-radius: 14px 14px 0 0;
  }
}
</style>
