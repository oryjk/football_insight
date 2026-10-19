<template>
  <view class="seat-map-panel" :class="{ 'seat-map-panel--embedded': embedded }">
    <!-- 首行即本场比赛：对阵 · 开球时间 + 意向数；不再单独放比赛卡。 -->
    <view v-if="headline" class="seat-map-panel__head">
      <view class="seat-map-panel__match">
        <text class="seat-map-panel__title">{{ headline }}</text>
        <text v-if="headlineMeta" class="seat-map-panel__meta">{{ headlineMeta }}</text>
      </view>
      <text v-if="badge" class="seat-map-panel__badge">{{ badge }}</text>
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
  /** 首行标题（对阵），为空时不显示首行。 */
  headline?: string
  /** 首行副信息（开球时间）。 */
  headlineMeta?: string
  /** 首行右侧徽标（意向数）。 */
  badge?: string
}>()

const emit = defineEmits<{
  (e: 'region-tap', key: string): void
}>()
</script>

<style scoped>
/* 与页面其他卡片同一套 FiPanel 外观（圆角、描边、底色、阴影），不再左右出血。 */
.seat-map-panel {
  padding: var(--fi-space-10) var(--fi-space-24) var(--fi-space-6);
  border-radius: var(--fi-component-panel-radius);
  border: var(--fi-component-panel-border);
  background: var(--fi-component-panel-bg);
  box-shadow: var(--fi-component-panel-shadow);
  overflow: hidden;
}

.seat-map-panel--embedded {
  padding: 0;
  background: transparent;
  box-shadow: none;
}

.seat-map-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: var(--fi-space-10) var(--fi-space-4) var(--fi-space-4);
}

.seat-map-panel__match {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-12);
}

.seat-map-panel__title {
  flex-shrink: 1;
  min-width: 0;
  overflow: hidden;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.seat-map-panel__meta {
  flex-shrink: 0;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  white-space: nowrap;
}

.seat-map-panel__badge {
  flex-shrink: 0;
  padding: var(--fi-space-8) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
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
