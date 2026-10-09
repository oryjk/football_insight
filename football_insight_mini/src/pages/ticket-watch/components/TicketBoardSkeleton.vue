<template>
  <view v-if="mode === 'current'">
    <view class="skeleton-stack">
      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--title" />
          </view>
          <view class="skeleton-pill" />
        </view>

        <view
          class="skeleton-line skeleton-line--meta skeleton-line--meta-wide"
        />

        <view class="watch-metrics skeleton-metrics">
          <view class="watch-metric skeleton-metric">
            <view class="skeleton-line skeleton-line--label" />
            <view class="skeleton-line skeleton-line--metric" />
          </view>
          <view class="watch-metric skeleton-metric">
            <view class="skeleton-line skeleton-line--label" />
            <view class="skeleton-line skeleton-line--metric" />
          </view>
        </view>

        <view class="watch-monitor-actions">
          <view class="skeleton-button" />
        </view>

        <view class="skeleton-line skeleton-line--body" />
      </view>

      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="focus-grid focus-grid--triple">
          <view
            v-for="index in 3"
            :key="`current-skeleton-focus-${index}`"
            class="skeleton-focus-chip"
          >
            <view class="skeleton-line skeleton-line--chip-name" />
            <view class="skeleton-line skeleton-line--chip-meta" />
            <view class="skeleton-line skeleton-line--chip-count" />
          </view>
        </view>
      </view>

      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="inventory-grid">
          <view
            v-for="index in 8"
            :key="`current-skeleton-cell-${index}`"
            class="skeleton-inventory-cell"
          >
            <view class="skeleton-line skeleton-line--cell-name" />
            <view class="skeleton-line skeleton-line--cell-count" />
            <view class="skeleton-line skeleton-line--cell-time" />
          </view>
        </view>
      </view>
    </view>
  </view>
  <view v-else-if="mode === 'history'">
    <view class="skeleton-stack">
      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="history-chip-list">
          <view
            v-for="index in 4"
            :key="`history-skeleton-chip-${index}`"
            class="skeleton-history-chip"
          >
            <view class="skeleton-line skeleton-line--history-date" />
            <view class="skeleton-line skeleton-line--history-match" />
          </view>
        </view>
      </view>

      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="focus-grid focus-grid--triple">
          <view
            v-for="index in 3"
            :key="`history-skeleton-focus-${index}`"
            class="skeleton-focus-chip"
          >
            <view class="skeleton-line skeleton-line--chip-name" />
            <view class="skeleton-line skeleton-line--chip-meta" />
            <view class="skeleton-line skeleton-line--chip-count" />
          </view>
        </view>
      </view>

      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="inventory-grid">
          <view
            v-for="index in 8"
            :key="`history-skeleton-cell-${index}`"
            class="skeleton-inventory-cell"
          >
            <view class="skeleton-line skeleton-line--cell-name" />
            <view class="skeleton-line skeleton-line--cell-count" />
            <view class="skeleton-line skeleton-line--cell-time" />
          </view>
        </view>
      </view>
    </view>
  </view>
  <view v-else-if="mode === 'history-stats'">
    <view class="skeleton-stack">
      <view class="panel skeleton-panel">
        <view class="section-heading section-heading--compact">
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--kicker" />
            <view class="skeleton-line skeleton-line--section" />
          </view>
          <view class="skeleton-pill skeleton-pill--short" />
        </view>

        <view class="history-trend-summary">
          <view
            v-for="index in 3"
            :key="`history-stats-skeleton-${index}`"
            class="history-trend-summary__item"
          >
            <view class="skeleton-line skeleton-line--label" />
            <view class="skeleton-line skeleton-line--metric" />
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{ mode: 'current' | 'history' | 'history-stats' }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.skeleton-stack {
  display: grid;
  gap: var(--fi-space-18);
}

.skeleton-panel {
  position: relative;
  overflow: hidden;
}

.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    var(--fi-color-ticket-skeleton-panel-background) 0%,
    var(--fi-color-ticket-skeleton-panel-background-2) 48%,
    var(--fi-color-ticket-skeleton-panel-background) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
}

.skeleton-copy-group {
  display: grid;
  gap: var(--fi-space-12);
  width: 100%;
}

.skeleton-line,
.skeleton-pill,
.skeleton-button,
.skeleton-focus-chip,
.skeleton-history-chip,
.skeleton-inventory-cell {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-skeleton-line-background),
    var(--fi-color-ticket-skeleton-line-background-2)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-skeleton-line-border);
}

.skeleton-line {
  border-radius: var(--fi-radius-round);
}

.skeleton-line--kicker {
  width: 168rpx;
  height: 24rpx;
}

.skeleton-line--title {
  width: 430rpx;
  max-width: 100%;
  height: 56rpx;
  border-radius: var(--fi-radius-md);
}

.skeleton-line--section {
  width: 320rpx;
  max-width: 100%;
  height: 40rpx;
  border-radius: var(--fi-radius-md);
}

.skeleton-line--meta {
  margin-top: var(--fi-space-18);
  height: 24rpx;
}

.skeleton-line--meta-wide {
  width: 72%;
}

.skeleton-line--label {
  width: 110rpx;
  height: 22rpx;
}

.skeleton-line--metric {
  width: 120rpx;
  height: 42rpx;
  margin-top: var(--fi-space-12);
  border-radius: var(--fi-radius-ticket-20);
}

.skeleton-line--body {
  width: 88%;
  height: 26rpx;
  margin-top: var(--fi-space-18);
}

.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-pill--short {
  width: 120rpx;
}

.skeleton-button {
  width: 100%;
  height: 72rpx;
  margin-top: var(--fi-space-18);
  border-radius: var(--fi-radius-round);
}

.skeleton-metric {
  min-height: 134rpx;
}

.skeleton-focus-chip {
  min-height: 132rpx;
  border-radius: var(--fi-radius-ticket-22);
  padding: var(--fi-space-18) var(--fi-space-14);
  display: grid;
  align-content: start;
  gap: var(--fi-space-10);
}

.skeleton-line--chip-name {
  width: 64rpx;
  height: 28rpx;
}

.skeleton-line--chip-meta {
  width: 82rpx;
  height: 18rpx;
}

.skeleton-line--chip-count {
  width: 96rpx;
  height: 22rpx;
}

.skeleton-history-chip {
  min-width: 250rpx;
  border-radius: var(--fi-radius-ticket-26);
  padding: var(--fi-space-20) var(--fi-space-22);
  display: grid;
  gap: var(--fi-space-12);
}

.skeleton-line--history-date {
  width: 120rpx;
  height: 22rpx;
}

.skeleton-line--history-match {
  width: 190rpx;
  height: 24rpx;
}

.skeleton-inventory-cell {
  min-height: 170rpx;
  border-radius: var(--fi-radius-md);
  padding: var(--fi-space-22) var(--fi-space-14) var(--fi-space-16);
  display: grid;
  justify-items: center;
  align-content: center;
  gap: var(--fi-space-10);
}

.skeleton-line--cell-name {
  width: 74rpx;
  height: 28rpx;
}

.skeleton-line--cell-count {
  width: 82rpx;
  height: 22rpx;
}

.skeleton-line--cell-time {
  width: 96rpx;
  height: 18rpx;
}

.watch-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-16);
}

.watch-monitor-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-12);
}

.watch-metric {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  min-height: 106rpx;
  border-radius: var(--fi-radius-ticket-22);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-watch-metric-background),
    var(--fi-color-ticket-watch-metric-background-2)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-metric-border);
  padding: var(--fi-space-16) var(--fi-space-78) var(--fi-space-18)
    var(--fi-space-18);
  box-shadow: var(--fi-shadow-ticket-watch-metric);
}

.focus-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-14);
}

.focus-grid--triple {
  grid-template-columns: repeat(3, minmax(0, 1fr));
  margin-top: 0;
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-14);
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

.history-chip-list {
  display: inline-flex;
  gap: var(--fi-space-10);
  padding-bottom: var(--fi-space-6);
}

@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}
</style>
