<template>
  <view class="seat-map-panel" :class="{ 'seat-map-panel--embedded': embedded }">
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
import StadiumMap from './StadiumMap.vue'
import type { TicketWatchRegion } from '../../../types/ticketWatch'

defineProps<{
  mode: 'browse' | 'filter' | 'published'
  regions: TicketWatchRegion[]
  badges: Record<string, number>
  filterKey: string
  currentKey: string
  desiredKeys: string[]
  hasMyRequest: boolean
  /** 用于地图弹层内部：去掉自身背景、阴影与横向出血。 */
  embedded?: boolean
}>()

const emit = defineEmits<{
  (e: 'region-tap', key: string): void
}>()
</script>

<style scoped>
/* 外部布局由页面的 seat-map-section 包裹节点负责。 */
.seat-map-panel {
  padding: var(--fi-space-10) var(--fi-space-24) var(--fi-space-6);
  background: var(--fi-color-surface-frosted);
  box-shadow: var(--fi-shadow-soft);
}

.seat-map-panel--embedded {
  padding: 0;
  background: transparent;
  box-shadow: none;
}

.seat-map-panel__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: 0 var(--fi-space-4) var(--fi-space-2);
}

.seat-map-panel__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.seat-map-panel__hint {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.stadium-wrap {
  margin: 0 calc(-1 * var(--fi-space-24));
}

.seat-map-panel--embedded .stadium-wrap {
  margin: 0;
}

.legend {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-2) var(--fi-space-4) var(--fi-space-6);
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
  width: var(--fi-space-16);
  height: var(--fi-space-16);
  border-radius: var(--fi-radius-round);
}

.legend__dot--current {
  background: var(--fi-color-seat-current);
}

.legend__dot--desired {
  background: var(--fi-color-seat-desired);
}

.legend__dot--hot {
  background: var(--fi-primitive-ink);
}

.legend__hint {
  color: var(--fi-color-text-muted);
}
</style>
