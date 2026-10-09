<template>
  <view class="panel history-trend-panel">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">Replay Volume</text>
        <text class="section-title">历史总回流走势</text>
      </view>
      <text class="meta-pill">
        {{ loading ? '统计中' : `${trend.loadedCount}/${trend.totalCount} 场` }}
      </text>
    </view>

    <view class="history-trend-summary">
      <view class="history-trend-summary__item">
        <text class="history-trend-summary__label">最近场</text>
        <text class="history-trend-summary__value">{{
          trend.latestTotal
        }}</text>
      </view>
      <view class="history-trend-summary__item">
        <text class="history-trend-summary__label">最高场</text>
        <text class="history-trend-summary__value">{{ trend.maxTotal }}</text>
      </view>
      <view class="history-trend-summary__item">
        <text class="history-trend-summary__label">场均</text>
        <text class="history-trend-summary__value">{{
          trend.averageTotal
        }}</text>
      </view>
    </view>

    <scroll-view scroll-y class="history-trend-scroll" show-scrollbar="false">
      <view class="history-trend-list">
        <view
          v-for="point in trend.points"
          :key="`history-trend-${point.match_id}`"
          class="history-trend-row"
          :class="[
            `history-trend-row--${point.tone}`,
            { 'history-trend-row--selected': point.is_selected },
          ]"
          @tap="emit('select', point.match_id)"
        >
          <view class="history-trend-row__meta">
            <text class="history-trend-row__date">{{ point.label }}</text>
            <text class="history-trend-row__opponent">{{ point.teams }}</text>
          </view>
          <view class="history-trend-row__track">
            <view
              class="history-trend-row__bar"
              :style="{ width: formatHistoryTrendBarWidth(point) }"
            />
          </view>
          <text class="history-trend-row__value">{{
            point.is_loaded ? point.total_occurrences : '...'
          }}</text>
        </view>
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchHistoryRefluxTrend } from '../helpers'
import { formatHistoryTrendBarWidth } from '../presentation'
defineProps<{ loading: boolean; trend: TicketWatchHistoryRefluxTrend }>()
const emit = defineEmits<{ (e: 'select', matchId: number): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.history-trend-panel {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-history-trend-summary-item-background-2),
    var(--fi-color-ticket-history-trend-panel-background)
  );
  border-color: var(--fi-color-ticket-history-trend-panel-border-color);
}

.history-trend-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-18);
}

.history-trend-summary__item {
  min-width: 0;
  border-radius: var(--fi-radius-sm);
  padding: var(--fi-space-16) var(--fi-space-14);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-history-trend-summary-item-background),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-history-trend-summary-item-border);
}

.history-trend-summary__label {
  display: block;
  color: var(--fi-color-ticket-history-trend-summary-label-color);
  font-size: var(--fi-font-18);
  line-height: var(--fi-leading-none);
}

.history-trend-summary__value {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.history-trend-scroll {
  width: 100%;
  margin-top: var(--fi-space-20);
  max-height: 620rpx;
}

.history-trend-list {
  display: grid;
  gap: var(--fi-space-12);
  padding: var(--fi-space-4) 0 var(--fi-space-2);
}

.history-trend-row {
  display: grid;
  grid-template-columns: minmax(0, 178rpx) minmax(90rpx, 1fr) 92rpx;
  align-items: center;
  gap: var(--fi-space-14);
  min-height: 84rpx;
  padding: var(--fi-space-14) var(--fi-space-16);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-history-trend-row-background);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-history-trend-row-border);
}

.history-trend-row--selected {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-history-trend-row-selected-background),
    var(--fi-color-ticket-history-trend-row-selected-background-2)
  );
  border-color: var(--fi-color-ticket-history-trend-summary-value-color);
  box-shadow: var(--fi-shadow-ticket-history-trend-row-selected);
}

.history-trend-row__meta {
  min-width: 0;
}

.history-trend-row__date,
.history-trend-row__opponent {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-trend-row__date {
  color: var(--fi-color-ticket-history-trend-summary-label-color);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.history-trend-row__opponent {
  margin-top: var(--fi-space-8);
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-ticket-1-08);
}

.history-trend-row__track {
  position: relative;
  overflow: hidden;
  height: 22rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-history-trend-row-track-background);
}

.history-trend-row__bar {
  height: 100%;
  border-radius: var(--fi-radius-round);
  background: linear-gradient(
    90deg,
    var(--fi-color-ticket-history-trend-row-bar-background),
    var(--fi-color-ticket-history-trend-row-bar-background-2)
  );
  transition: width 0.24s ease;
}

.history-trend-row--low .history-trend-row__bar {
  background: linear-gradient(
    90deg,
    var(
      --fi-color-ticket-history-trend-row-low-history-trend-row-bar-background
    ),
    var(
      --fi-color-ticket-history-trend-row-low-history-trend-row-bar-background-2
    )
  );
}

.history-trend-row--mid .history-trend-row__bar {
  background: linear-gradient(
    90deg,
    var(
      --fi-color-ticket-history-trend-row-mid-history-trend-row-bar-background
    ),
    var(
      --fi-color-ticket-history-trend-row-mid-history-trend-row-bar-background-2
    )
  );
}

.history-trend-row--high .history-trend-row__bar {
  background: linear-gradient(
    90deg,
    var(
      --fi-color-ticket-history-trend-row-high-history-trend-row-bar-background
    ),
    var(
      --fi-color-ticket-history-trend-row-high-history-trend-row-bar-background-2
    )
  );
}

.history-trend-row--empty .history-trend-row__bar {
  background: transparent;
}

.history-trend-row--selected .history-trend-row__track {
  background: var(
    --fi-color-ticket-history-trend-row-selected-history-trend-row-track-background
  );
}

.history-trend-row--selected .history-trend-row__bar {
  background: linear-gradient(
    90deg,
    var(
      --fi-color-ticket-history-trend-row-selected-history-trend-row-bar-background
    ),
    var(--fi-color-bg)
  );
}

.history-trend-row__value {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  text-align: right;
}

.history-trend-row--selected .history-trend-row__date,
.history-trend-row--selected .history-trend-row__opponent,
.history-trend-row--selected .history-trend-row__value {
  color: var(--fi-color-bg);
}
</style>
