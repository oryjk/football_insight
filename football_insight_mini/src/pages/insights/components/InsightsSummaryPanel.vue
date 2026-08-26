<template>
  <view class="panel summary-panel">
    <view class="section-heading">
      <view>
        <text class="section-kicker">当前球队</text>
        <text class="section-title">{{ insight.team_name }}</text>
      </view>
      <text class="meta-note">第 {{ insight.rank_no }} 名</text>
    </view>

    <view class="summary-panel__grid">
      <view class="briefing-card">
        <text class="briefing-card__label">总进球</text>
        <view class="briefing-card__body">
          <text
            :key="`goals-for-${insight.team_id}-${animatedGoalsFor}`"
            class="summary-value summary-value--animated"
          >
            {{ animatedGoalsFor }}
          </text>
          <text class="briefing-card__subvalue">赛季累计打进</text>
        </view>
      </view>

      <view class="briefing-card">
        <text class="briefing-card__label">总失球</text>
        <view class="briefing-card__body">
          <text
            :key="`goals-against-${insight.team_id}-${animatedGoalsAgainst}`"
            class="summary-value summary-value--animated"
          >
            {{ animatedGoalsAgainst }}
          </text>
          <text class="briefing-card__subvalue">赛季累计丢失</text>
        </view>
      </view>
    </view>
  </view>

</template>

<script setup lang="ts">
import type { TeamInsight } from '../../../types/insight'
import { useAnimatedInteger } from '../../../composables/useAnimatedInteger'

const props = defineProps<{
  insight: TeamInsight
}>()

const animatedGoalsFor = useAnimatedInteger(() => props.insight.goals_for_total ?? 0)
const animatedGoalsAgainst = useAnimatedInteger(() => props.insight.goals_against_total ?? 0)
</script>
<style scoped lang="css">
.panel {
  position: relative;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-component-title-size);
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 1rpx;
  padding: var(--fi-space-8) 0 0;
}

.summary-panel__grid {
  margin-top: var(--fi-space-18);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-12);
}

.briefing-card {
  border-radius: var(--fi-radius-lg);
  border: var(--fi-primitive-border-width) solid var(--fi-color-divider);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 253, 0.98));
  padding: var(--fi-space-18) var(--fi-space-20);
}

.briefing-card__label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.briefing-card__body {
  display: grid;
  min-height: 130rpx;
  align-items: end;
  gap: var(--fi-space-6);
}

.summary-value {
  display: inline-block;
  color: var(--fi-color-text-strong);
  font-size: 60rpx;
  font-weight: var(--fi-weight-extrabold);
  line-height: 0.92;
}

.summary-value--animated {
  animation: fi-value-pop 420ms cubic-bezier(0.22, 1, 0.36, 1);
  transform-origin: center bottom;
}

.briefing-card__subvalue {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.board-entry__copy {
  position: relative;
  z-index: 1;
  display: block;
  margin-top: var(--fi-space-18);
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
}
</style>
