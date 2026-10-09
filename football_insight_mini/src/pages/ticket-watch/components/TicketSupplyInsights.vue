<template>
  <view class="insight-stack">
    <view class="panel">
      <view class="section-heading section-heading--compact">
        <view>
          <text class="section-kicker">{{
            history ? 'Replay Signal' : 'Supply Signal'
          }}</text>
          <text class="section-title">{{
            history ? '历史供给洞察' : '比赛供给洞察'
          }}</text>
        </view>
        <view class="section-heading__actions">
          <text class="meta-pill">{{ stats.priceBandCount }} 个价位</text>
          <button class="collapse-pill" @tap="emit('toggle', 'insight')">
            {{ insightCollapsed ? '展开' : '收起' }}
          </button>
        </view>
      </view>

      <template v-if="!insightCollapsed">
        <view class="insight-grid">
          <view class="insight-metric">
            <text class="insight-metric__label">覆盖率</text>
            <text class="insight-metric__value">{{
              formatCoverage(stats)
            }}</text>
            <text class="insight-metric__note">{{
              formatPercent(stats.activeRegionRatio)
            }}</text>
          </view>
          <view class="insight-metric">
            <text class="insight-metric__label">热点价位</text>
            <text class="insight-metric__value">{{
              formatHotPrice(stats)
            }}</text>
            <text class="insight-metric__note">{{
              formatHotPriceMeta(stats)
            }}</text>
          </view>
          <view class="insight-metric">
            <text class="insight-metric__label">累计回流</text>
            <text class="insight-metric__value">{{
              stats.totalOccurrences
            }}</text>
            <text class="insight-metric__note">张</text>
          </view>
        </view>

        <text class="insight-copy">{{
          buildBoardInsightSummary(stats, history ? 'history' : 'current')
        }}</text>
      </template>
    </view>
    <view class="panel">
      <view class="section-heading section-heading--compact">
        <view>
          <text class="section-kicker">{{
            history ? 'Replay Decision' : 'Decision Guide'
          }}</text>
          <text class="section-title">{{
            history ? '历史抢票决策' : '帮用户抢票决策'
          }}</text>
        </view>
        <view class="section-heading__actions">
          <text class="meta-pill">{{ history ? '复盘' : '实时' }}</text>
          <button class="collapse-pill" @tap="emit('toggle', 'decision')">
            {{ decisionCollapsed ? '展开' : '收起' }}
          </button>
        </view>
      </view>

      <view v-if="!decisionCollapsed" class="decision-list">
        <view v-for="line in decisionLines" :key="line" class="decision-item">
          <text class="decision-item__dot" />
          <text class="decision-item__text">{{ line }}</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchBoardStats } from '../helpers'
import {
  formatCoverage,
  formatPercent,
  formatHotPrice,
  formatHotPriceMeta,
  buildBoardInsightSummary,
} from '../presentation'
defineProps<{
  stats: TicketWatchBoardStats
  decisionLines: string[]
  insightCollapsed: boolean
  decisionCollapsed: boolean
  history?: boolean
}>()
const emit = defineEmits<{ (e: 'toggle', section: string): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.insight-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-16);
}

.insight-metric {
  border-radius: var(--fi-radius-sm);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-insight-metric-background),
    var(--fi-color-ticket-watch-match-card-reference-background)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-insight-metric-border);
  padding: var(--fi-space-16);
}

.insight-metric__label {
  display: block;
  color: var(--fi-color-ticket-insight-metric-label-color);
  font-size: var(--fi-font-18);
}

.insight-metric__value {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-ticket-1-05);
}

.insight-metric__note {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-ticket-insight-metric-label-color);
  font-size: var(--fi-font-18);
}

.insight-copy {
  display: block;
  margin-top: var(--fi-space-14);
  color: var(--fi-color-ticket-insight-copy-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-ticket-1-6);
}

.decision-list {
  display: grid;
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-16);
}

.decision-item {
  display: flex;
  align-items: flex-start;
  gap: var(--fi-space-14);
  padding: var(--fi-space-16) var(--fi-space-18);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-bg);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
}

.decision-item__dot {
  width: 14rpx;
  height: 14rpx;
  margin-top: var(--fi-space-10);
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-history-chip-teams-color);
}

.decision-item__text {
  color: var(--fi-color-ticket-price-tone-neutral-tone-text);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-ticket-1-55);
}
.insight-stack {
  display: grid;
  gap: var(--fi-space-18);
}
</style>
