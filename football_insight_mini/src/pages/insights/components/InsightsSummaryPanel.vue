<template>
  <view class="summary-panel">
    <view class="summary-panel__header">
      <text class="summary-panel__name">{{ insight.team_name }}</text>
      <text class="summary-panel__rank">第 {{ insight.rank_no }} 名</text>
      <view
        class="summary-panel__board"
        role="button"
        aria-label="打开当前球队战术板"
        hover-class="summary-panel__board--pressed"
        :hover-stay-time="80"
        @tap="emit('open-board')"
      >
        <text>战术板</text>
        <image :src="chevronIcon" class="summary-panel__chevron" mode="aspectFit" />
      </view>
    </view>

    <view class="summary-panel__grid">
      <view class="briefing-card">
        <text class="briefing-card__label">总进球</text>
        <text
          :key="`goals-for-${insight.team_id}-${animatedGoalsFor}`"
          class="summary-value summary-value--animated"
        >{{ animatedGoalsFor }}</text>
      </view>
      <view class="briefing-card">
        <text class="briefing-card__label">总失球</text>
        <text
          :key="`goals-against-${insight.team_id}-${animatedGoalsAgainst}`"
          class="summary-value summary-value--animated"
        >{{ animatedGoalsAgainst }}</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TeamInsight } from '../../../types/insight'
import { useAnimatedInteger } from '../../../composables/useAnimatedInteger'
import chevronIcon from '../../../static/rankings/chevron-right.svg'

const props = defineProps<{ insight: TeamInsight }>()
const emit = defineEmits<{ (e: 'open-board'): void }>()

const animatedGoalsFor = useAnimatedInteger(() => props.insight.goals_for_total ?? 0)
const animatedGoalsAgainst = useAnimatedInteger(() => props.insight.goals_against_total ?? 0)
</script>

<style scoped lang="css">
.summary-panel { min-width: 0; }
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
.summary-panel__board {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  margin-left: auto;
  min-height: var(--fi-space-64);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
}
.summary-panel__board--pressed { opacity: var(--fi-opacity-insights-pressed); }
.summary-panel__chevron { width: var(--fi-space-32); height: var(--fi-space-32); }
.summary-panel__grid {
  margin-top: var(--fi-space-18);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-12);
}
.briefing-card {
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
.summary-value--animated {
  animation: fi-value-pop 420ms cubic-bezier(0.22, 1, 0.36, 1);
  transform-origin: center bottom;
}
@media (prefers-reduced-motion: reduce) {
  .summary-value--animated { animation: none; }
}
</style>
