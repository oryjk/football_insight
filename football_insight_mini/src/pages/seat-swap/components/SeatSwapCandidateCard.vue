<template>
  <view class="candidate" :class="cardClass">
    <view class="candidate__main">
      <!-- 卡片内的球场缩略图：红=当前座位、绿=目标座位，点开看大图。 -->
      <SeatSwapSeatThumb
        class="candidate__thumb"
        :current-key="candidate.current_region_key"
        :desired-keys="desiredKeys"
        @open="emit('preview-seat', candidate.request_id)"
      />
      <view class="candidate__content">
        <view class="candidate__user">
          <text class="candidate__name">{{ candidate.display_name }}</text>
          <text v-if="match" class="candidate__match" :class="`candidate__match--${match.level}`">匹配度 {{ match.score }}%</text>
        </view>
        <text v-if="match" class="candidate__match-hint">{{ match.label }}：{{ match.hint }}</text>
        <view class="candidate__seat">
          <text class="candidate__seat-label">当前</text>
          <text class="candidate__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
        </view>
        <view class="candidate__wants">
          <text class="candidate__wants-label">目标</text>
          <text class="candidate__wants-value">{{ desiredSeatText }}</text>
        </view>
      </view>
      <!-- 右列：状态标签贴顶、操作按钮贴底，按钮不再垂直居中。 -->
      <view class="candidate__side">
        <view v-if="statusText" class="candidate__status" :class="statusClass">
          <view class="candidate__status-dot" />
          <text>{{ statusText }}</text>
        </view>
        <button
          v-if="action === 'confirm'"
          class="candidate__action candidate__action--primary"
          :class="{ 'candidate__action--urgent': candidate.status !== 'display_only' }"
          hover-class="candidate__action--pressed"
          @tap.stop="emit('confirm', candidate.request_id)"
        >
          <!-- lucide arrow-left-right：带互换图标，一眼看出是可点的换座操作。 -->
          <image
            class="candidate__action-icon"
            :src="candidate.status === 'display_only' ? '/static/seat-swap/arrow-left-right-ink.svg' : '/static/seat-swap/arrow-left-right-white.svg'"
            mode="aspectFit"
            aria-hidden="true"
          />
          <text>{{ candidate.status === 'display_only' ? '换到这' : '确认换座' }}</text>
        </button>
        <button
          v-else-if="action === 'view'"
          class="candidate__action candidate__action--primary candidate__action--urgent"
          hover-class="candidate__action--pressed"
          @tap.stop="emit('view', candidate.request_id)"
        >
          查看
        </button>
        <button
          v-else-if="action === 'cancel_confirmation'"
          class="candidate__action candidate__action--secondary"
          hover-class="candidate__action--pressed"
          @tap.stop="emit('cancel-confirmation', candidate.request_id)"
        >
          取消匹配
        </button>
        <!-- 已被别人匹配：原按钮位置放一个匹配成功的标记，卡片不再空着。 -->
        <view v-else-if="candidate.status === 'taken'" class="candidate__taken-mark">
          <image class="candidate__taken-icon" src="/static/seat-swap/circle-check-green.svg" mode="aspectFit" aria-hidden="true" />
          <text class="candidate__taken-text">已匹配</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import SeatSwapSeatThumb from './SeatSwapSeatThumb.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import type { SeatSwapMatch } from '../helpers'
import {
  formatSeatSwapDesiredSeats,
  formatSeatSwapSeatLabel,
  seatSwapStatusLabel,
} from '../../../utils/stadiumRegions'

const props = defineProps<{
  candidate: SeatSwapCandidate
  action: 'confirm' | 'view' | 'cancel_confirmation' | 'none'
  match?: SeatSwapMatch | null
}>()

const emit = defineEmits<{
  (e: 'confirm', requestId: string): void
  (e: 'view', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
  (e: 'preview-seat', requestId: string): void
}>()

const desiredKeys = computed(() => props.candidate.desired_seats.map((seat) => seat.region_key))
const statusText = computed(() => seatSwapStatusLabel(props.candidate.status))
const isMatched = computed(() => props.candidate.status === 'matched')
const cardClass = computed(() => ({
  'candidate--matched': isMatched.value,
  'candidate--taken': props.candidate.status === 'taken',
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
  padding: var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-surface);
}

.candidate__main {
  display: flex;
  align-items: stretch;
  gap: var(--fi-space-16);
}

.candidate__thumb {
  align-self: flex-start;
}

.candidate__content {
  flex: 1;
  min-width: 0;
}

.candidate__user {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.candidate__side {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: stretch;
  gap: var(--fi-space-12);
  width: var(--fi-component-seat-swap-action-width);
}




.candidate__match {
  flex-shrink: 0;
  padding: var(--fi-space-2) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
}

.candidate__match--perfect {
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.candidate__match--partial {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
}

.candidate__match-hint {
  display: block;
  margin-top: var(--fi-space-4);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
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
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  gap: var(--fi-space-8);
  padding: var(--fi-space-6) var(--fi-space-10);
  white-space: nowrap;
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

/* 卡片内操作是次级按钮：单行胶囊、竖直居中，不再是撑满整卡高度的黑块。 */
.candidate__action {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 100%;
  min-height: var(--fi-component-seat-swap-action-min-height);
  margin: auto 0 0;
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

.candidate__action-icon {
  flex-shrink: 0;
  width: var(--fi-space-28);
  height: var(--fi-space-28);
  margin-right: var(--fi-space-8);
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

.candidate--matched .candidate__action--primary {
  background: var(--fi-primitive-white);
  border-color: var(--fi-primitive-white);
  color: var(--fi-primitive-green-deep);
}

.candidate__taken-mark {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-6);
  margin: auto 0 0;
  min-height: var(--fi-component-seat-swap-action-min-height);
}

.candidate__taken-icon {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
}

.candidate__taken-text {
  color: var(--fi-primitive-green-deep);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

/* 已被别人匹配：灰底降权，和可操作的白色卡片区分开。 */
.candidate--taken {
  background: rgba(var(--fi-primitive-ink-rgb), 0.06);
  border-color: transparent;
}

.candidate--taken .candidate__thumb,
.candidate--taken .candidate__seat,
.candidate--taken .candidate__wants {
  opacity: 0.6;
}

.candidate--taken .candidate__name {
  color: var(--fi-color-text-secondary);
}

.candidate--taken .candidate__status {
  background: rgba(var(--fi-primitive-ink-rgb), 0.08);
}

/* 已匹配：成功绿卡片，和红色的「需要处理」邀请区分开。 */
.candidate--matched {
  background: var(--fi-primitive-green);
  border-color: var(--fi-primitive-green-deep);
  box-shadow: var(--fi-shadow-card);
}

.candidate--matched .candidate__name,
.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__status {
  color: var(--fi-primitive-white);
}

.candidate--matched .candidate__seat,
.candidate--matched .candidate__wants,
.candidate--matched .candidate__status {
  background: rgba(var(--fi-primitive-white-rgb), 0.14);
}

.candidate--matched .candidate__seat-label,
.candidate--matched .candidate__wants-label {
  background: rgba(var(--fi-primitive-white-rgb), 0.12);
}

.candidate--matched .candidate__status-dot {
  background: var(--fi-primitive-white);
}
</style>
