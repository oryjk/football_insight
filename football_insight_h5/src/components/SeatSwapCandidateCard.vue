<template>
  <div class="seat-swap-candidate" :class="cardClass">
    <div class="seat-swap-candidate__main">
      <div class="seat-swap-candidate__content">
        <div class="seat-swap-candidate__user">
          <img
            v-if="candidate.avatar_url"
            class="seat-swap-candidate__avatar"
            :src="candidate.avatar_url"
            :alt="candidate.display_name"
          />
          <div v-else class="seat-swap-candidate__avatar seat-swap-candidate__avatar--fallback">
            <span>{{ fallbackInitial }}</span>
          </div>
          <div class="seat-swap-candidate__id">
            <span class="seat-swap-candidate__name">{{ candidate.display_name }}</span>
            <div class="seat-swap-candidate__seat">
              <span class="seat-swap-candidate__seat-label">当前座位</span>
              <span class="seat-swap-candidate__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</span>
            </div>
          </div>
          <span v-if="statusText" class="seat-swap-candidate__status" :class="statusClass">
            {{ statusText }}
          </span>
        </div>
        <div class="seat-swap-candidate__wants">
          <span class="seat-swap-candidate__wants-label">目标座位</span>
          <span class="seat-swap-candidate__wants-value">{{ desiredSeatText }}</span>
        </div>
        <div v-if="candidate.contact" class="seat-swap-candidate__contact">
          <span v-if="candidate.contact.wechat_id" class="seat-swap-candidate__contact-line">
            微信：{{ candidate.contact.wechat_id }}
          </span>
          <span v-if="candidate.contact.phone_number" class="seat-swap-candidate__contact-line">
            手机：{{ candidate.contact.phone_number }}
          </span>
        </div>
      </div>
      <button
        v-if="action === 'confirm'"
        type="button"
        class="seat-swap-candidate__action seat-swap-candidate__action--primary"
        @click.stop="emit('confirm', candidate.request_id)"
      >
        {{ candidate.status === 'display_only' ? '我要换到这' : '确认换座' }}
      </button>
      <button
        v-else-if="action === 'cancel_confirmation'"
        type="button"
        class="seat-swap-candidate__action seat-swap-candidate__action--secondary"
        @click.stop="emit('cancel-confirmation', candidate.request_id)"
      >
        取消匹配
      </button>
      <button
        v-else-if="action === 'matched_cancel'"
        type="button"
        class="seat-swap-candidate__action seat-swap-candidate__action--matched-cancel"
        @click.stop="emit('matched-cancel', candidate.request_id)"
      >
        取消匹配
      </button>
    </div>
  </div>
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
  margin: 5px 0;
  padding: 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.96);
  border: 1px solid rgba(232, 233, 238, 0.95);
  box-shadow: 0 4px 9px rgba(26, 28, 36, 0.04);
}

.seat-swap-candidate__main {
  display: flex;
  align-items: stretch;
  gap: 9px;
}

.seat-swap-candidate__content {
  min-width: 0;
  flex: 1;
}

.seat-swap-candidate__user {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.seat-swap-candidate__avatar {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #dde5ee;
  object-fit: cover;
}

.seat-swap-candidate__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #4b5563;
  font-size: 14px;
}

.seat-swap-candidate__id {
  flex: 1;
  min-width: 0;
}

.seat-swap-candidate__name {
  display: block;
  color: #121212;
  font-size: 14px;
}

.seat-swap-candidate__seat {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  margin-top: 4px;
  overflow: hidden;
  border-radius: 9px;
  background: #fff1f0;
  color: #b42318;
}

.seat-swap-candidate__seat-label {
  flex-shrink: 0;
  padding: 4px 5px 4px 6px;
  background: rgba(226, 59, 46, 0.12);
  color: #b42318;
  font-size: 10px;
  font-weight: 600;
  line-height: 1;
}

.seat-swap-candidate__seat-value {
  min-width: 0;
  padding: 4px 7px 4px 5px;
  overflow: hidden;
  font-size: 13px;
  font-weight: 800;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.seat-swap-candidate__status {
  flex-shrink: 0;
  padding: 3px 7px;
  border-radius: 999px;
  background: #f6f7fb;
  border: 1px solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 10px;
  line-height: 1.4;
  white-space: nowrap;
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
  max-width: calc(100% - 40px);
  margin-top: 6px;
  margin-left: 40px;
  overflow: hidden;
  border-radius: 9px;
  background: #eef8f2;
  color: #175c31;
}

.seat-swap-candidate__wants-label {
  flex-shrink: 0;
  padding: 4px 5px 4px 6px;
  background: rgba(70, 171, 89, 0.12);
  color: rgba(24, 103, 67, 0.88);
  font-size: 10px;
  font-weight: 600;
  line-height: 1;
}

.seat-swap-candidate__wants-value {
  min-width: 0;
  padding: 4px 7px 4px 5px;
  overflow: hidden;
  font-size: 12px;
  font-weight: 700;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.seat-swap-candidate__contact {
  margin-top: 6px;
  padding: 8px 9px;
  border-radius: 8px;
  background: #f6f7fb;
  font-size: 11px;
  color: #6f6a5f;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.seat-swap-candidate__action {
  flex-shrink: 0;
  align-self: center;
  padding: 9px 12px;
  border: 0;
  border-radius: 11px;
  color: #fff;
  font-size: 14px;
  font-weight: 700;
  line-height: 1.25;
  cursor: pointer;
  text-align: center;
}

.seat-swap-candidate__action--primary {
  background: #15161b;
  box-shadow: 0 5px 11px rgba(21, 22, 27, 0.12);
}

.seat-swap-candidate__action--secondary {
  background: #ffffff;
  color: #20242c;
  border: 1px solid rgba(32, 36, 44, 0.16);
  box-shadow: none;
}

.seat-swap-candidate__action--matched-cancel {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
  border: 1px solid rgba(255, 255, 255, 0.28);
  box-shadow: none;
}

.seat-swap-candidate--matched {
  background: linear-gradient(180deg, #c61f26 0%, #b3131f 100%);
  border-color: rgba(152, 12, 24, 0.92);
  box-shadow: 0 6px 14px rgba(138, 12, 22, 0.22);
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

.seat-swap-candidate--matched .seat-swap-candidate__seat,
.seat-swap-candidate--matched .seat-swap-candidate__seat-label,
.seat-swap-candidate--matched .seat-swap-candidate__wants {
  background: rgba(255, 255, 255, 0.14);
}

.seat-swap-candidate--matched .seat-swap-candidate__status {
  background: rgba(255, 255, 255, 0.14);
  border-color: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}

.seat-swap-candidate--matched .seat-swap-candidate__contact {
  background: rgba(255, 255, 255, 0.16);
  border: 1px solid rgba(255, 255, 255, 0.16);
}

@media (max-width: 768px) {
  .seat-swap-candidate__wants {
    margin-left: 0;
    max-width: 100%;
  }
}
</style>
