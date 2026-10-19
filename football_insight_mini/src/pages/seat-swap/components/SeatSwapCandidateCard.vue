<template>
  <view class="candidate" :class="cardClass">
    <view class="candidate__main">
      <!-- 卡片内的球场缩略图：红=当前座位、绿=目标座位，点开看大图。 -->
      <SeatSwapSeatThumb
        :current-key="candidate.current_region_key"
        :desired-keys="desiredKeys"
        @open="emit('preview-seat', candidate.request_id)"
      />
      <view class="candidate__content">
        <view class="candidate__user">
          <text class="candidate__name">{{ candidate.display_name }}</text>
          <view v-if="statusText" class="candidate__status" :class="statusClass">
            <view class="candidate__status-dot" />
            <text>{{ statusText }}</text>
          </view>
        </view>
        <view class="candidate__seat">
          <text class="candidate__seat-label">当前</text>
          <text class="candidate__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
        </view>
        <view class="candidate__wants">
          <text class="candidate__wants-label">目标</text>
          <text class="candidate__wants-value">{{ desiredSeatText }}</text>
        </view>
        <view v-if="candidate.contact" class="candidate__contact">
          <text v-if="candidate.contact.wechat_id" class="candidate__contact-line">
            微信:{{ candidate.contact.wechat_id }}
          </text>
          <text v-if="candidate.contact.phone_number" class="candidate__contact-line">
            手机:{{ candidate.contact.phone_number }}
          </text>
        </view>
      </view>
      <button
        v-if="action === 'confirm'"
        class="candidate__action candidate__action--primary"
        :class="{ 'candidate__action--urgent': candidate.status !== 'display_only' }"
        hover-class="candidate__action--pressed"
        @tap.stop="emit('confirm', candidate.request_id)"
      >
        {{ candidate.status === 'display_only' ? '我要换到这' : '确认换座' }}
      </button>
      <button
        v-else-if="action === 'cancel_confirmation'"
        class="candidate__action candidate__action--secondary"
        hover-class="candidate__action--pressed"
        @tap.stop="emit('cancel-confirmation', candidate.request_id)"
      >
        取消匹配
      </button>
      <button
        v-else-if="action === 'matched_cancel'"
        class="candidate__action candidate__action--matched-cancel"
        hover-class="candidate__action--pressed"
        @tap.stop="emit('matched-cancel', candidate.request_id)"
      >
        取消匹配
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SeatSwapSeatThumb from './SeatSwapSeatThumb.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import {
  formatSeatSwapDesiredSeats,
  formatSeatSwapSeatLabel,
  seatSwapStatusLabel,
} from '../../../utils/stadiumRegions'

const props = defineProps<{
  candidate: SeatSwapCandidate
  action: 'confirm' | 'cancel_confirmation' | 'matched_cancel' | 'none'
}>()

const emit = defineEmits<{
  (e: 'confirm', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
  (e: 'matched-cancel', requestId: string): void
  (e: 'preview-seat', requestId: string): void
}>()

const desiredKeys = computed(() => props.candidate.desired_seats.map((seat) => seat.region_key))
const statusText = computed(() => seatSwapStatusLabel(props.candidate.status))
const isMatched = computed(() => props.candidate.status === 'matched')
const cardClass = computed(() => ({
  'candidate--matched': isMatched.value,
}))
// 状态色只保留中性、可沟通（绿点）与"对方已确认你"（主色强调，需要你去操作）。
const statusClass = computed(() => ({
  'candidate__status--ready': props.candidate.status === 'communicable',
  'candidate__status--attention': props.candidate.status === 'peer_confirmed_me',
}))
const desiredSeatText = computed(() => formatSeatSwapDesiredSeats(props.candidate.desired_seats))
</script>

<style scoped>
.candidate {
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-card);
}

.candidate__main {
  display: flex;
  align-items: flex-start;
  gap: var(--fi-space-16);
}

.candidate__content {
  flex: 1;
  min-width: 0;
}

.candidate__user {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}




.candidate__name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.candidate__seat,
.candidate__wants {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  overflow: hidden;
  border-radius: var(--fi-radius-sm);
}

.candidate__seat {
  margin-top: var(--fi-space-10);
  background: var(--fi-color-seat-current-bg);
  color: var(--fi-color-seat-current);
}

.candidate__wants {
  margin-top: var(--fi-space-8);
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.candidate__seat-label,
.candidate__wants-label {
  flex-shrink: 0;
  padding: var(--fi-space-6) var(--fi-space-10) var(--fi-space-6) var(--fi-space-12);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.candidate__seat-label {
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
}

.candidate__wants-label {
  background: rgba(var(--fi-primitive-ink-rgb), 0.04);
}

.candidate__seat-value,
.candidate__wants-value {
  min-width: 0;
  padding: var(--fi-space-6) var(--fi-space-14) var(--fi-space-6) var(--fi-space-10);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
}

.candidate__seat-value {
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.candidate__wants-value {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.candidate__status {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  padding: var(--fi-space-6) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.candidate__status-dot {
  width: var(--fi-space-12);
  height: var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-muted);
}

.candidate__status--ready .candidate__status-dot {
  background: var(--fi-color-seat-desired);
}

.candidate__status--attention {
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
  color: var(--fi-color-primary);
}

.candidate__status--attention .candidate__status-dot {
  background: var(--fi-color-primary);
}

.candidate__contact {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
  margin-top: var(--fi-space-12);
  padding: var(--fi-space-16) var(--fi-space-18);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
}

.candidate__contact-line {
  display: block;
}

/* 卡片内操作是次级按钮：单行胶囊、竖直居中，不再是撑满整卡高度的黑块。 */
.candidate__action {
  flex-shrink: 0;
  align-self: center;
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: var(--fi-component-seat-swap-action-width);
  min-height: var(--fi-component-seat-swap-action-min-height);
  margin: 0;
  padding: 0 var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  text-align: center;
  white-space: nowrap;
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.candidate__action::after {
  border: 0;
}

.candidate__action--pressed {
  transform: scale(0.96);
}

.candidate__action--primary {
  background: var(--fi-color-surface);
  color: var(--fi-color-text-strong);
  border: var(--fi-border-width) solid var(--fi-primitive-ink);
}

/* 对方已确认、等你确认换座：唯一需要立刻处理的卡片操作，用品牌红实心。 */
.candidate__action--primary.candidate__action--urgent {
  background: var(--fi-color-primary);
  border-color: var(--fi-color-primary);
  color: var(--fi-primitive-white);
}

.candidate__action--secondary {
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-ink-rgb), 0.16);
}

.candidate__action--matched-cancel {
  background: rgba(var(--fi-primitive-white-rgb), 0.18);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-white-rgb), 0.28);
}

/* 已匹配：纯色主色卡（原红色渐变） */
.candidate--matched {
  background: var(--fi-color-primary);
  border-color: var(--fi-color-primary-deep);
  box-shadow: var(--fi-shadow-card-strong);
}

.candidate--matched .candidate__name,
.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__contact,
.candidate--matched .candidate__status {
  color: var(--fi-primitive-white);
}

.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__status,
.candidate--matched .candidate__contact {
  background: rgba(var(--fi-primitive-white-rgb), 0.14);
}

.candidate--matched .candidate__seat-label,
.candidate--matched .candidate__wants-label {
  background: rgba(var(--fi-primitive-white-rgb), 0.12);
}

.candidate--matched .candidate__status-dot {
  background: var(--fi-primitive-white);
}

.candidate--matched .candidate__contact {
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-20) var(--fi-space-22);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-white-rgb), 0.16);
  font-size: var(--fi-font-26);
  gap: var(--fi-space-8);
}
</style>
