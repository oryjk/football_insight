<template>
  <view class="seat-swap-candidate" :class="cardClass">
    <view class="seat-swap-candidate__main">
      <view class="seat-swap-candidate__content">
        <view class="seat-swap-candidate__user">
          <image
            v-if="candidate.avatar_url"
            class="seat-swap-candidate__avatar"
            :src="candidate.avatar_url"
            mode="aspectFill"
          />
          <view v-else class="seat-swap-candidate__avatar seat-swap-candidate__avatar--fallback">
            <text>{{ fallbackInitial }}</text>
          </view>
          <view class="seat-swap-candidate__id">
            <text class="seat-swap-candidate__name">{{ candidate.display_name }}</text>
            <view class="seat-swap-candidate__seat">
              <text class="seat-swap-candidate__seat-label">当前座位</text>
              <text class="seat-swap-candidate__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
            </view>
          </view>
          <text v-if="statusText" class="seat-swap-candidate__status" :class="statusClass">
            {{ statusText }}
          </text>
        </view>
        <view class="seat-swap-candidate__wants">
          <text class="seat-swap-candidate__wants-label">目标座位</text>
          <text class="seat-swap-candidate__wants-value">{{ desiredSeatText }}</text>
        </view>
        <view v-if="candidate.contact" class="seat-swap-candidate__contact">
          <text v-if="candidate.contact.wechat_id" class="seat-swap-candidate__contact-line">
            微信:{{ candidate.contact.wechat_id }}
          </text>
          <text v-if="candidate.contact.phone_number" class="seat-swap-candidate__contact-line">
            手机:{{ candidate.contact.phone_number }}
          </text>
        </view>
      </view>
      <button
        v-if="action === 'confirm'"
        class="seat-swap-candidate__action seat-swap-candidate__action--primary"
        @tap.stop="emit('confirm', candidate.request_id)"
      >
        {{ candidate.status === 'display_only' ? '我要换到这' : '确认换座' }}
      </button>
      <button
        v-else-if="action === 'cancel_confirmation'"
        class="seat-swap-candidate__action seat-swap-candidate__action--secondary"
        @tap.stop="emit('cancel-confirmation', candidate.request_id)"
      >
        取消匹配
      </button>
      <button
        v-else-if="action === 'matched_cancel'"
        class="seat-swap-candidate__action seat-swap-candidate__action--matched-cancel"
        @tap.stop="emit('matched-cancel', candidate.request_id)"
      >
        取消匹配
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SeatSwapCandidate } from '../types/seatSwap'
import {
  formatSeatSwapDesiredSeats,
  formatSeatSwapSeatLabel,
  seatSwapStatusLabel,
} from '../utils/stadiumRegions'

const props = defineProps<{
  candidate: SeatSwapCandidate
  action: 'confirm' | 'cancel_confirmation' | 'matched_cancel' | 'none'
}>()

const emit = defineEmits<{
  (e: 'confirm', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
  (e: 'matched-cancel', requestId: string): void
}>()

const fallbackInitial = computed(() => (props.candidate.display_name || '球').slice(0, 1))
const statusText = computed(() => seatSwapStatusLabel(props.candidate.status))
const isMatched = computed(() => props.candidate.status === 'matched')
const cardClass = computed(() => ({
  'seat-swap-candidate--matched': isMatched.value,
}))
const statusClass = computed(() => ({
  'seat-swap-candidate__status--communicable': props.candidate.status === 'communicable',
  'seat-swap-candidate__status--waiting': props.candidate.status === 'waiting_peer_confirmation',
  'seat-swap-candidate__status--peer-confirmed': props.candidate.status === 'peer_confirmed_me',
  'seat-swap-candidate__status--matched': props.candidate.status === 'matched',
}))
const desiredSeatText = computed(() => formatSeatSwapDesiredSeats(props.candidate.desired_seats))
</script>

<style scoped>
.seat-swap-candidate {
  margin: 10rpx 0;
  padding: 22rpx;
  border-radius: 22rpx;
  background: rgba(255, 255, 255, 0.96);
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  box-shadow: 0 8rpx 18rpx rgba(26, 28, 36, 0.04);
}

.seat-swap-candidate__main {
  display: flex;
  align-items: stretch;
  gap: 18rpx;
}

.seat-swap-candidate__content {
  min-width: 0;
  flex: 1;
}

.seat-swap-candidate__user {
  display: flex;
  align-items: flex-start;
  gap: 16rpx;
}

.seat-swap-candidate__avatar {
  flex-shrink: 0;
  width: 64rpx;
  height: 64rpx;
  border-radius: 50%;
  background: #dde5ee;
}

.seat-swap-candidate__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4b5563;
  font-size: 28rpx;
  font-weight: 400;
}

.seat-swap-candidate__id {
  flex: 1;
  min-width: 0;
}

.seat-swap-candidate__name {
  display: block;
  color: #121212;
  font-size: 28rpx;
  font-weight: 400;
}

.seat-swap-candidate__seat {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  margin-top: 8rpx;
  overflow: hidden;
  border-radius: 22rpx;
  background: #fff1f0;
  color: #b42318;
}

.seat-swap-candidate__seat-label {
  flex-shrink: 0;
  padding: 7rpx 10rpx 7rpx 12rpx;
  background: rgba(226, 59, 46, 0.12);
  color: #b42318;
  font-size: 20rpx;
  font-weight: 600;
  line-height: 1;
}

.seat-swap-candidate__seat-value {
  min-width: 0;
  padding: 7rpx 14rpx 7rpx 10rpx;
  overflow: hidden;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 1;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.seat-swap-candidate__status {
  flex-shrink: 0;
  padding: 6rpx 14rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 20rpx;
  font-weight: 400;
  line-height: 1;
}

.seat-swap-candidate__status--communicable {
  background: #eef8f2;
  border-color: rgba(29, 138, 85, 0.35);
  color: #167348;
}

.seat-swap-candidate__status--waiting {
  background: #fff6e8;
  border-color: rgba(217, 119, 6, 0.34);
  color: #b45309;
}

.seat-swap-candidate__status--peer-confirmed {
  background: #edf4ff;
  border-color: rgba(37, 99, 235, 0.28);
  color: #1d4ed8;
}

.seat-swap-candidate__status--matched {
  background: #f2ecff;
  border-color: rgba(109, 40, 217, 0.22);
  color: #6d28d9;
}

.seat-swap-candidate__wants {
  display: inline-flex;
  align-items: center;
  max-width: calc(100% - 80rpx);
  margin-top: 12rpx;
  margin-left: 80rpx;
  overflow: hidden;
  border-radius: 22rpx;
  background: #eef8f2;
  color: #175c31;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.5);
}

.seat-swap-candidate__wants-label {
  flex-shrink: 0;
  padding: 7rpx 10rpx 7rpx 12rpx;
  background: rgba(70, 171, 89, 0.12);
  color: rgba(24, 103, 67, 0.88);
  font-size: 20rpx;
  font-weight: 600;
  line-height: 1;
}

.seat-swap-candidate__wants-value {
  min-width: 0;
  padding: 7rpx 14rpx 7rpx 10rpx;
  overflow: hidden;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.seat-swap-candidate__contact {
  margin-top: 12rpx;
  padding: 16rpx 18rpx;
  border-radius: 16rpx;
  background: #f6f7fb;
  font-size: 22rpx;
  color: #6f6a5f;
  display: flex;
  flex-direction: column;
  gap: 4rpx;
}

.seat-swap-candidate__contact-line {
  display: block;
}

.seat-swap-candidate__action {
  flex-shrink: 0;
  align-self: stretch;
  width: 168rpx;
  min-height: 152rpx;
  padding: 18rpx 20rpx;
  border-radius: 26rpx;
  color: #fff;
  font-size: 30rpx;
  font-weight: 700;
  line-height: 1.25;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.seat-swap-candidate__action::after {
  border: 0;
}

.seat-swap-candidate__action--primary {
  background: #15161b;
  box-shadow: 0 10rpx 22rpx rgba(21, 22, 27, 0.12);
}

.seat-swap-candidate__action--secondary {
  background: #ffffff;
  color: #20242c;
  border: 2rpx solid rgba(32, 36, 44, 0.16);
  box-shadow: none;
}

.seat-swap-candidate__action--matched-cancel {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
  border: 2rpx solid rgba(255, 255, 255, 0.28);
  box-shadow: none;
}

.seat-swap-candidate--matched {
  background: linear-gradient(180deg, #c61f26 0%, #b3131f 100%);
  border-color: rgba(152, 12, 24, 0.92);
  box-shadow: 0 12rpx 28rpx rgba(138, 12, 22, 0.22);
}

.seat-swap-candidate--matched .seat-swap-candidate__avatar {
  background: rgba(255, 255, 255, 0.18);
}

.seat-swap-candidate--matched .seat-swap-candidate__avatar--fallback,
.seat-swap-candidate--matched .seat-swap-candidate__name,
.seat-swap-candidate--matched .seat-swap-candidate__seat-label,
.seat-swap-candidate--matched .seat-swap-candidate__seat-value,
.seat-swap-candidate--matched .seat-swap-candidate__wants-label,
.seat-swap-candidate--matched .seat-swap-candidate__wants-value,
.seat-swap-candidate--matched .seat-swap-candidate__contact,
.seat-swap-candidate--matched .seat-swap-candidate__contact-line {
  color: #ffffff;
}

.seat-swap-candidate--matched .seat-swap-candidate__seat {
  background: rgba(255, 255, 255, 0.14);
}

.seat-swap-candidate--matched .seat-swap-candidate__seat-label {
  background: rgba(255, 255, 255, 0.14);
}

.seat-swap-candidate--matched .seat-swap-candidate__wants {
  background: rgba(255, 255, 255, 0.14);
  box-shadow: none;
}

.seat-swap-candidate--matched .seat-swap-candidate__wants-label {
  background: rgba(255, 255, 255, 0.12);
}

.seat-swap-candidate--matched .seat-swap-candidate__status {
  background: rgba(255, 255, 255, 0.14);
  border-color: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}

.seat-swap-candidate--matched .seat-swap-candidate__status--matched {
  background: rgba(255, 255, 255, 0.18);
  border-color: rgba(255, 255, 255, 0.2);
  color: #ffffff;
}

.seat-swap-candidate--matched .seat-swap-candidate__contact {
  margin-top: 16rpx;
  padding: 20rpx 22rpx;
  background: rgba(255, 255, 255, 0.16);
  border: 2rpx solid rgba(255, 255, 255, 0.16);
  font-size: 25rpx;
  gap: 8rpx;
}
</style>
