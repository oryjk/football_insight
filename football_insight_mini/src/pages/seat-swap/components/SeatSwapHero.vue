<template>
  <view class="hero-card">
    <view class="hero-card__icon-box">
      <text class="hero-card__icon-mark">↔</text>
    </view>
    <view class="hero-card__body">
      <text class="eyebrow">当前比赛 · 换座撮合</text>
      <text class="hero-card__title">{{ matchTitle }}</text>
      <text v-if="matchSummary" class="hero-card__summary">{{ matchSummary }}</text>
    </view>
    <text class="meta-pill">{{ candidatesCount }} 条意向</text>
  </view>
</template>

<script setup lang="ts">
defineProps<{
  matchTitle: string
  matchSummary: string
  candidatesCount: number
}>()
</script>

<style scoped>
.hero-card {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
  padding: var(--fi-space-24);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-lg);
  background: rgba(255, 255, 255, 0.96);
  box-shadow: var(--fi-shadow-soft);
  margin-bottom: var(--fi-space-16);
}

.hero-card__icon-box {
  flex-shrink: 0;
  width: 64rpx;
  height: 64rpx;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  display: flex;
  align-items: center;
  justify-content: center;
}

.hero-card__icon-mark {
  color: #4b515d;
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.hero-card__body {
  flex: 1;
  min-width: 0;
}

.eyebrow {
  display: block;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  font-weight: 400;
}

.hero-card__title {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: 1.18;
  font-weight: 400;
}

.hero-card__summary {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: 1.45;
}

.meta-pill {
  flex-shrink: 0;
  align-self: center;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-12) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: 400;
}

</style>
