<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="座位位置"
    :title="candidate ? `${candidate.display_name} 的座位` : ''"
    height="80vh"
    fit-content
    @update:visible="emit('update:visible', $event)"
  >
    <view v-if="candidate" class="seat-sheet">
      <StadiumMap
        mode="review"
        :regions="regions"
        :current-key="candidate.current_region_key"
        :desired-keys="desiredKeys"
      />
      <view class="seat-sheet__rows">
        <view class="seat-sheet__row">
          <view class="seat-sheet__dot seat-sheet__dot--current" />
          <text class="seat-sheet__label">当前</text>
          <text class="seat-sheet__value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
        </view>
        <view class="seat-sheet__row">
          <view class="seat-sheet__dot seat-sheet__dot--desired" />
          <text class="seat-sheet__label">目标</text>
          <text class="seat-sheet__value">{{ formatSeatSwapDesiredSeats(candidate.desired_seats) }}</text>
        </view>
      </view>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import StadiumMap from './StadiumMap.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import type { TicketWatchRegion } from '../../../types/ticketWatch'
import { formatSeatSwapDesiredSeats, formatSeatSwapSeatLabel } from '../../../utils/stadiumRegions'

const props = defineProps<{
  visible: boolean
  regions: TicketWatchRegion[]
  candidate: SeatSwapCandidate | null
}>()

const emit = defineEmits<{ (e: 'update:visible', value: boolean): void }>()

const desiredKeys = computed(() => props.candidate?.desired_seats.map((seat) => seat.region_key) || [])
</script>

<style scoped>
.seat-sheet {
  padding-bottom: var(--fi-space-24);
}

.seat-sheet__rows {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.seat-sheet__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}

.seat-sheet__dot {
  flex-shrink: 0;
  width: var(--fi-space-16);
  height: var(--fi-space-16);
  border-radius: var(--fi-radius-round);
}

.seat-sheet__dot--current {
  background: var(--fi-color-seat-current);
}

.seat-sheet__dot--desired {
  background: var(--fi-color-seat-desired);
}

.seat-sheet__label {
  flex-shrink: 0;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.seat-sheet__value {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}
</style>
