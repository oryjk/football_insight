<template>
  <view class="seat-swap-candidate">
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
          <text class="seat-swap-candidate__seat-label">可换出</text>
          <text class="seat-swap-candidate__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
        </view>
      </view>
      <text v-if="statusText" class="seat-swap-candidate__status" :class="statusClass">
        {{ statusText }}
      </text>
    </view>
    <view class="seat-swap-candidate__wants">
      <text class="seat-swap-candidate__wants-label">想换到</text>
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
    <button
      v-if="action === 'confirm'"
      class="seat-swap-candidate__action"
      @tap.stop="emit('confirm', candidate.request_id)"
    >
      确认换座
    </button>
    <button
      v-else-if="action === 'cancel_confirmation'"
      class="seat-swap-candidate__action seat-swap-candidate__action--secondary"
      @tap.stop="emit('cancel-confirmation', candidate.request_id)"
    >
      取消匹配
    </button>
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
  action: 'confirm' | 'cancel_confirmation' | 'none'
}>()

const emit = defineEmits<{
  (e: 'confirm', requestId: string): void
  (e: 'cancel-confirmation', requestId: string): void
}>()

const fallbackInitial = computed(() => (props.candidate.display_name || '球').slice(0, 1))
const statusText = computed(() => seatSwapStatusLabel(props.candidate.status))
const statusClass = computed(() => ({
  'seat-swap-candidate__status--hot': props.candidate.status !== 'matched',
}))
const desiredSeatText = computed(() => formatSeatSwapDesiredSeats(props.candidate.desired_seats))
</script>

<style scoped>
.seat-swap-candidate {
  margin: 10rpx 0;
  padding: 22rpx;
  border-radius: 22rpx;
  background: rgba(255, 255, 255, 0.96);
  border: 1rpx solid rgba(238, 233, 224, 0.95);
  box-shadow: 0 8rpx 18rpx rgba(46, 38, 27, 0.04);
}

.seat-swap-candidate__user {
  display: flex;
  align-items: center;
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
  background: linear-gradient(180deg, rgba(255, 247, 232, 0.95), rgba(251, 240, 215, 0.88));
  color: #6e4f16;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.52);
}

.seat-swap-candidate__seat-label {
  flex-shrink: 0;
  padding: 7rpx 10rpx 7rpx 12rpx;
  background: rgba(244, 194, 58, 0.18);
  color: rgba(129, 95, 31, 0.88);
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
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  border: 1rpx solid rgba(230, 220, 198, 0.92);
  color: #9c855c;
  font-size: 20rpx;
  font-weight: 400;
  line-height: 1;
}

.seat-swap-candidate__status--hot {
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  border-color: rgba(29, 138, 85, 0.35);
  color: #167348;
}

.seat-swap-candidate__wants {
  display: inline-flex;
  align-items: center;
  max-width: calc(100% - 80rpx);
  margin-top: 12rpx;
  margin-left: 80rpx;
  overflow: hidden;
  border-radius: 22rpx;
  background: linear-gradient(180deg, rgba(237, 249, 242, 0.96), rgba(227, 244, 233, 0.9));
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
  background: rgba(255, 251, 242, 0.6);
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
  margin-top: 18rpx;
  width: 100%;
  padding: 14rpx 22rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 24rpx;
  font-weight: 400;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.08), 0 8rpx 18rpx rgba(21, 22, 27, 0.14);
}

.seat-swap-candidate__action::after {
  border: 0;
}

.seat-swap-candidate__action--secondary {
  background: #ffffff;
  color: #20242c;
  border: 2rpx solid rgba(32, 36, 44, 0.16);
  box-shadow: none;
}
</style>
