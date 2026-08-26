<template>
  <view class="seat-map-panel">
    <view class="seat-map-panel__head">
      <text class="seat-map-panel__title">座位区域</text>
      <text class="seat-map-panel__hint">点分区可切换</text>
    </view>
    <view class="stadium-wrap">
      <StadiumMap
        :mode="mode"
        :regions="regions"
        :badges="badges"
        :filter-key="filterKey"
        :current-key="currentKey"
        :desired-keys="desiredKeys"
        @region-tap="(key: string) => emit('region-tap', key)"
      />
    </view>
    <view class="legend">
      <view class="legend__items">
        <template v-if="hasMyRequest">
          <view class="legend__item">
            <view class="legend__dot legend__dot--current"></view>
            <text>当前座位</text>
          </view>
          <view class="legend__item">
            <view class="legend__dot legend__dot--desired"></view>
            <text>目标座位</text>
          </view>
        </template>
        <view v-else class="legend__item">
          <view class="legend__dot legend__dot--hot"></view>
          <text>有发布</text>
        </view>
      </view>
      <text class="legend__hint">点分区可筛选</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import StadiumMap from '../../../components/StadiumMap.vue'
import type { TicketWatchRegion } from '../../../types/ticketWatch'

defineProps<{
  mode: 'browse' | 'filter' | 'published'
  regions: TicketWatchRegion[]
  badges: Record<string, number>
  filterKey: string
  currentKey: string
  desiredKeys: string[]
  hasMyRequest: boolean
}>()

const emit = defineEmits<{
  (e: 'region-tap', key: string): void
}>()
</script>

<style scoped>
/* 与页面主体同宽：面板 sticky 在品牌导航下方，负 margin 抵消页面左右 padding。 */
.seat-map-panel {
  position: sticky;
  top: var(--fi-brand-nav-height);
  z-index: 5;
  margin: 4rpx -24rpx 0;
  padding: var(--fi-space-10) var(--fi-space-24) var(--fi-space-6);
  background: rgba(255, 255, 255, 0.97);
  box-shadow: var(--fi-shadow-soft);
}

.seat-map-panel__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: 0 4rpx 2rpx;
}

.seat-map-panel__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: 500;
}

.seat-map-panel__hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
}

.stadium-wrap {
  margin: 0 -24rpx;
}

.legend {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2rpx 4rpx var(--fi-space-6);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.legend__items {
  display: flex;
  align-items: center;
  gap: var(--fi-space-22);
}

.legend__item {
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
}

.legend__dot {
  display: inline-block;
  width: 16rpx;
  height: 16rpx;
  border-radius: 50%;
}

.legend__dot--current {
  background: #e23b2e;
}

.legend__dot--desired {
  background: #1d8a55;
}

.legend__dot--hot {
  background: var(--fi-primitive-ink);
}

.legend__hint {
  color: var(--fi-color-text-muted);
}
</style>
