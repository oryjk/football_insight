<template>
  <FiBottomSheet
    :visible="visible"
    :eyebrow="sheetEyebrow"
    :title="sheetTitle"
    height="76vh"
    compact-footer
    avoid-keyboard
    @update:visible="handleVisibleChange"
  >
    <view v-if="presetCandidate" class="candidate-target">
      <text class="candidate-target__label">我要换到 · {{ presetCandidate.display_name }}的座位</text>
      <text class="candidate-target__seat">{{ formatSeatSwapSeatLabel(presetCandidate) }}</text>
      <text class="candidate-target__hint">对方想换到 {{ formatSeatSwapDesiredSeats(presetCandidate.desired_seats) }}</text>
      <text class="candidate-target__hint">目标已固定，填写你的座位和联系方式后确认换座。</text>
    </view>
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
          <input v-model="form.current_row" :adjust-position="false" type="number" class="input-box" placeholder="如 8" />
          <text v-if="formErrors.current_row" class="field-error">{{ formErrors.current_row }}</text>
        </view>
        <view class="row-input__field">
          <text class="row-input__label">当前号</text>
          <input v-model="form.current_seat_no" :adjust-position="false" type="number" class="input-box" placeholder="如 15" />
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
          <input v-model="form.wechat_id" :adjust-position="false" class="input-box" placeholder="至少填一项" />
        </view>
        <view class="row-input__field">
          <text class="row-input__label">手机号</text>
          <input v-model="form.phone_number" :adjust-position="false" class="input-box" type="number" placeholder="11 位手机号" />
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
        >{{ submitting ? '提交中...' : (presetCandidate ? '确认换座' : (myRequest ? '更新发布' : '发布换座')) }}</button>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import StadiumMap from './StadiumMap.vue'
import type { SeatSwapCandidate, SeatSwapRequest } from '../../../types/seatSwap'
import type { TicketWatchRegion } from '../../../types/ticketWatch'
import { formatSeatSwapDesiredSeats, formatSeatSwapSeatLabel } from '../../../utils/stadiumRegions'
import type { SeatSwapFormState } from '../helpers'
import { useSeatSwapPublishForm } from './useSeatSwapPublishForm'

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

const {
  selectionStep, stagedCurrentRegionKey, stagedDesiredSeats, formErrors, form,
  selectionSteps, selectionStepIndex, sheetEyebrow, sheetTitle, sheetMapMode,
  stagedCurrentRegionName, stagedDesiredKeys, confirmedDesiredKeys,
  canConfirmCurrentSelection, canConfirmDesiredSelection, regionTagClass,
  handleSheetMapTap, confirmCurrentSelection, confirmDesiredSelection,
  goPreviousSelectionStep, jumpToStep, submit,
} = useSeatSwapPublishForm(props, (payload) => emit('submit', payload))

function handleVisibleChange(value: boolean): void {
  emit('update:visible', value)
}

</script>

<style scoped>
.candidate-target {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
  margin-bottom: var(--fi-space-18);
  padding: var(--fi-space-18);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
}

.candidate-target__label,
.candidate-target__hint {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
}

.candidate-target__seat {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
}

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
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-snug);
  box-sizing: border-box;
}

.steps__item--done {
  background: var(--fi-color-seat-desired-bg);
  border-color: transparent;
  color: var(--fi-color-seat-desired);
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
  border: var(--fi-border-width) solid transparent;
  line-height: var(--fi-leading-none);
}

/* 区域色是 StadiumMap 业务色系，不属于视觉 token，保留字面量（见下方 decorative 块）。 */
.tag--region {
  border-radius: var(--fi-radius-xs);
  border-color: transparent;
  color: var(--fi-primitive-white);
  font-weight: var(--fi-weight-black);
  box-shadow: 0 8rpx 18rpx rgba(18, 25, 20, 0.18);
}

/* decorative:start 票价分区配色，与 StadiumMap 一致 */
.tag--region-blue { background: #336fbd; }
.tag--region-green { background: #46ab59; }
.tag--region-purple { background: #6c369b; }
.tag--region-yellow { background: #f4c23a; color: #17191f; }
.tag--region-navy { background: #0f215e; }
.tag--region-red { background: #ec3b20; }
.tag--region-vip { background: #b90000; }
.tag--region-muted { background: #d9dee7; color: #17191f; }
/* decorative:end */

.tag--empty {
  background: var(--fi-color-seat-current-bg);
  color: var(--fi-color-seat-current);
  border-color: rgba(var(--fi-primitive-red-rgb), 0.42);
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
  border-radius: var(--fi-radius-xs);
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
  margin-bottom: var(--fi-space-6);
}

.input-box {
  width: 100%;
  height: 72rpx;
  padding: 0 var(--fi-space-18);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-xs);
  font-size: var(--fi-font-26);
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
  border-radius: var(--fi-radius-xs);
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
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-22);
}

.btn-primary {
  flex: 1;
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  box-shadow: var(--fi-shadow-soft);
}

.btn-primary[disabled] {
  background: var(--fi-color-text-caption);
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
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
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
