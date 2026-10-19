<template>
  <view :key="insight.team_id" class="summary-panel">
    <view class="summary-panel__header">
      <text class="summary-panel__name">{{ insight.team_name }}</text>
      <text class="summary-panel__rank">第 {{ insight.rank_no }} 名</text>

    </view>

    <view class="summary-panel__grid">
      <view class="briefing-card">
        <text class="briefing-card__label">总进球</text>
        <text class="summary-value">{{ insight.goals_for_total ?? 0 }}</text>
      </view>
      <view class="briefing-card">
        <text class="briefing-card__label">总失球</text>
        <text class="summary-value">{{ insight.goals_against_total ?? 0 }}</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TeamInsight } from '../../../types/insight'

defineProps<{ insight: TeamInsight }>()

</script>

<style scoped lang="css">
.summary-panel { min-width: 0; animation: insights-summary-fade 260ms ease both; }
.summary-panel__header {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  min-width: 0;
}
.summary-panel__name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-extrabold);
}
.summary-panel__rank {
  flex-shrink: 0;
  padding: var(--fi-space-8) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
.summary-panel__grid {
  margin-top: var(--fi-space-18);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-12);
}
.briefing-card {
  animation: none;
  display: grid;
  gap: var(--fi-space-20);
  padding: var(--fi-space-18) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}
.briefing-card__label { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.summary-value {
  display: inline-block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-60);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-insights-value);
}
@keyframes insights-summary-fade {
  from { opacity: 0; }
  to { opacity: 1; }
}
@media (prefers-reduced-motion: reduce) {
  .summary-panel { animation: none; }
}
</style>
