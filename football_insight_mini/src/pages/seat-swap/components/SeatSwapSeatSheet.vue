<template>
  <FiBottomSheet
    :visible="visible"
    :title="candidate ? `${candidate.display_name} 想换座` : ''"
    height="84vh"
    fit-content
    respect-window-bottom
    @update:visible="emit('update:visible', $event)"
  >
    <view v-if="candidate && summary" class="seat-sheet">
      <!-- 图只回答"在哪"：红=现在、绿=想换到 -->
      <StadiumMap
        mode="review"
        :regions="regions"
        :current-key="candidate.current_region_key"
        :desired-keys="desiredKeys"
      />

      <!-- 文字回答"值不值得换"：现在 → 想换到，附票价与差价 -->
      <view class="swap">
        <view class="swap__from">
          <text class="swap__caption">现在</text>
          <text class="swap__region">{{ regionLabel(summary.current.region) }}</text>
          <text v-if="summary.current.seat" class="swap__seat">{{ summary.current.seat }}</text>
          <text v-if="summary.current.price" class="swap__price">{{ summary.current.price }}</text>
        </view>

        <image class="swap__arrow" src="/static/rankings/chevron-right.svg" mode="aspectFit" />

        <view class="swap__to">
          <text class="swap__caption swap__caption--to">想换到</text>
          <view v-for="item in summary.desired" :key="item.key" class="swap__row">
            <view class="swap__row-main">
              <text class="swap__row-region">{{ regionLabel(item.region) }}</text>
              <text v-if="item.seat" class="swap__row-seat">{{ item.seat }}</text>
            </view>
            <text v-if="item.price" class="swap__row-price">{{ item.price }}</text>
            <text v-if="item.delta" class="swap__delta" :class="`swap__delta--${item.deltaKind}`">{{ item.delta }}</text>
          </view>
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
import { buildSeatSwapSwapSummary } from '../helpers'

const props = defineProps<{
  visible: boolean
  regions: TicketWatchRegion[]
  candidate: SeatSwapCandidate | null
}>()

const emit = defineEmits<{ (e: 'update:visible', value: boolean): void }>()

const desiredKeys = computed(() => props.candidate?.desired_seats.map((seat) => seat.region_key) || [])
const summary = computed(() => (props.candidate ? buildSeatSwapSwapSummary(props.candidate, props.regions) : null))

/** 纯数字分区补"区"字（104 → 104区），VIP 等保持原样。 */
function regionLabel(region: string): string {
  return /^\d+$/.test(region) ? `${region}区` : region
}
</script>

<style scoped>
.seat-sheet {
  padding-bottom: var(--fi-space-24);
}

.swap {
  display: flex;
  align-items: stretch;
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-20);
}

.swap__from,
.swap__to {
  display: flex;
  flex-direction: column;
  border-radius: var(--fi-radius-md);
  padding: var(--fi-space-18) var(--fi-space-20);
}

.swap__from {
  flex: 0 0 34%;
  gap: var(--fi-space-4);
  background: var(--fi-color-seat-current-bg);
}

.swap__to {
  flex: 1;
  min-width: 0;
  gap: var(--fi-space-12);
  background: var(--fi-color-seat-desired-bg);
}

.swap__caption {
  color: var(--fi-color-seat-current);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.swap__caption--to {
  color: var(--fi-color-seat-desired);
}

.swap__region {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.swap__seat {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}

.swap__price {
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.swap__arrow {
  flex-shrink: 0;
  align-self: center;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.swap__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}

.swap__row-main {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-8);
}

.swap__row-region {
  flex-shrink: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.swap__row-seat {
  min-width: 0;
  overflow: hidden;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.swap__row-price {
  flex-shrink: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.swap__delta {
  flex-shrink: 0;
  padding: var(--fi-space-4) var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.swap__delta--cheaper {
  background: var(--fi-color-surface);
  color: var(--fi-color-seat-desired);
}

.swap__delta--pricier {
  background: var(--fi-color-surface);
  color: var(--fi-color-seat-current);
}

.swap__delta--same {
  background: var(--fi-color-surface);
  color: var(--fi-color-text-muted);
}
</style>
