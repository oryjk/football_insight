<template>
  <FiPanel>
    <view class="match-card">
      <view class="match-card__icon-box">
        <image class="match-card__icon" src="/static/seat-swap/arrow-left-right.svg" mode="aspectFit" />
      </view>
      <view class="match-card__body">
        <text class="match-card__eyebrow">当前比赛 · 换座撮合</text>
        <text class="match-card__title">{{ matchTitle }}</text>
        <text v-if="matchSummary" class="match-card__summary">{{ matchSummary }}</text>
      </view>
      <text class="match-card__pill">{{ candidatesCount }} 条意向</text>
    </view>
    <!-- 二期"本场动态"预留；一期不传内容。 -->
    <slot />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'

defineProps<{
  matchTitle: string
  matchSummary: string
  candidatesCount: number
}>()
</script>

<style scoped>
.match-card {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
}

.match-card__icon-box {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-md);
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
}

.match-card__icon {
  width: var(--fi-space-36);
  height: var(--fi-space-36);
}

.match-card__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.match-card__eyebrow {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.match-card__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.match-card__summary {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.match-card__pill {
  flex-shrink: 0;
  padding: var(--fi-space-12) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
</style>
