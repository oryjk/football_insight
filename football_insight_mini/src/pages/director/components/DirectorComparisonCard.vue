<template>
  <view class="compare-card">
    <view class="compare-head">
      <view class="compare-head__left">
        <text class="compare-kicker">1v1 分歧报告</text>
        <text class="compare-title">你和 @{{ targetAuthorName }} 的建队默契</text>
      </view>
      <view class="compare-score">
        <text class="compare-score__num">{{ comparison.overlap_percent }}%</text>
        <text class="compare-score__label">默契度</text>
      </view>
    </view>

    <text class="compare-summary">{{ comparison.summary_line }}</text>

    <view v-if="comparison.common_foreign.length" class="compare-group">
      <text class="compare-group__label">共同认可的外援（{{ comparison.common_foreign.length }}）</text>
      <view class="compare-tags">
        <text v-for="item in comparison.common_foreign" :key="item.key" class="compare-tag compare-tag--common">
          {{ item.player_name }}
        </text>
      </view>
    </view>

    <view v-if="comparison.leave_disagreements.length" class="compare-group">
      <text class="compare-group__label">去留最大分歧</text>
      <view class="diff-list">
        <view
          v-for="diff in comparison.leave_disagreements"
          :key="diff.player.key"
          class="diff-row"
        >
          <text class="diff-row__name">{{ diff.player.player_name }}</text>
          <text class="diff-row__detail">
            TA 想{{ diff.target_action === 'stay' ? '留' : '送走' }} · 你想{{ diff.my_action === 'stay' ? '留' : '送走' }}
          </text>
        </view>
      </view>
    </view>

    <view v-if="comparison.only_in_target.length || comparison.only_in_mine.length" class="compare-split">
      <view v-if="comparison.only_in_target.length" class="split-col">
        <text class="compare-group__label">TA 独选</text>
        <view class="compare-tags">
          <text v-for="item in comparison.only_in_target" :key="item.key" class="compare-tag">
            {{ item.player_name }}
          </text>
        </view>
      </view>
      <view v-if="comparison.only_in_mine.length" class="split-col">
        <text class="compare-group__label">你独选</text>
        <view class="compare-tags">
          <text v-for="item in comparison.only_in_mine" :key="item.key" class="compare-tag compare-tag--mine">
            {{ item.player_name }}
          </text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { DirectorComparisonDto } from '../../../types/director'

defineProps<{
  comparison: DirectorComparisonDto
  targetAuthorName: string
}>()
</script>

<style scoped lang="css">
.compare-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-primary);
  box-shadow: var(--fi-shadow-card);
}

.compare-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.compare-head__left {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.compare-kicker {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-extrabold);
}

.compare-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.compare-score {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--fi-space-4);
}

.compare-score__num {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-black);
  line-height: var(--fi-leading-none);
  font-variant-numeric: tabular-nums;
}

.compare-score__label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
}

.compare-summary {
  padding: var(--fi-space-14) var(--fi-space-16);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-normal);
}

.compare-group {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.compare-group__label {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.compare-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--fi-space-8);
}

.compare-tag {
  padding: var(--fi-space-6) var(--fi-space-12);
  border-radius: var(--fi-radius-2xs);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.compare-tag--common {
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.compare-tag--mine {
  background: var(--fi-color-outsider-soft);
  color: var(--fi-color-outsider);
}

.diff-list {
  display: flex;
  flex-direction: column;
  border-top: 1rpx solid var(--fi-color-divider-soft);
}

.diff-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-10);
  padding: var(--fi-space-12) 0;
  border-bottom: 1rpx solid var(--fi-color-divider-soft);
}

.diff-row:last-child {
  border-bottom: none;
}

.diff-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.diff-row__detail {
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-22);
}

.compare-split {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.split-col {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
}
</style>
