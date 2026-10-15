<template>
  <view class="panel standings-launcher">
    <view
      v-for="table in tables"
      :key="table.slug"
      class="standings-launcher-card"
      role="button"
      :aria-label="`查看${table.label}`"
      hover-class="standings-launcher-card--pressed"
      hover-stay-time="80"
      @tap="emit('open', table.slug)"
    >
      <view class="standings-launcher-card__icon" :class="{ 'standings-launcher-card__icon--penalty': isPenalty(table) }">
        <image :src="isPenalty(table) ? penaltyIcon : theoryIcon" class="standings-launcher-card__icon-img" mode="aspectFit" />
      </view>

      <view class="standings-launcher-card__body">
        <text class="standings-launcher-card__title">{{ table.label }}</text>
        <view class="standings-launcher-card__facts">
          <view class="standings-launcher-card__fact">
            <image :src="crownIcon" class="standings-launcher-card__fact-icon" mode="aspectFit" />
            <image v-if="facts(table).leaderAvatar" :src="facts(table).leaderAvatar || ''" class="standings-launcher-card__avatar" mode="aspectFit" />
            <text class="standings-launcher-card__fact-text">{{ facts(table).leaderName }}</text>
          </view>
          <view v-if="isPenalty(table) && facts(table).adjustedCount > 0" class="standings-launcher-card__fact standings-launcher-card__fact--alert">
            <image :src="alertIcon" class="standings-launcher-card__fact-icon" mode="aspectFit" />
            <text class="standings-launcher-card__fact-text">{{ facts(table).adjustedCount }} 队</text>
          </view>
        </view>
      </view>

      <image :src="chevronIcon" class="standings-launcher-card__chevron" mode="aspectFit" />
    </view>
  </view>
</template>

<script setup lang="ts">
import penaltyIcon from '../../../static/rankings/standings-penalty.svg'
import theoryIcon from '../../../static/rankings/standings-theory.svg'
import crownIcon from '../../../static/rankings/crown.svg'
import alertIcon from '../../../static/rankings/triangle-alert.svg'
import chevronIcon from '../../../static/rankings/chevron-right.svg'
import type { StandingsTable } from '../../../types/insight'
import { buildStandingsPreviewFacts } from '../helpers'

defineProps<{
  tables: StandingsTable[]
}>()

const emit = defineEmits<{
  (e: 'open', slug: string): void
}>()

function isPenalty(table: StandingsTable): boolean {
  return table.slug !== 'standings_without_penalty'
}

function facts(table: StandingsTable) {
  return buildStandingsPreviewFacts(table)
}
</script>

<style scoped lang="css">
.panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-8) var(--fi-space-20);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.standings-launcher-card {
  display: flex;
  align-items: center;
  gap: var(--fi-space-20);
  padding: var(--fi-space-20) 0;
  border-bottom: var(--fi-border-divider);
}

.standings-launcher-card:last-child {
  border-bottom: 0;
}

.standings-launcher-card--pressed {
  opacity: 0.7;
}

.standings-launcher-card__icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 80rpx;
  height: 80rpx;
  border-radius: 24rpx;
  background: var(--fi-color-page);
}

.standings-launcher-card__icon--penalty {
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
}

.standings-launcher-card__icon-img {
  width: 44rpx;
  height: 44rpx;
}

.standings-launcher-card__body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: var(--fi-space-10);
}

.standings-launcher-card__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: 1.18;
  font-weight: var(--fi-weight-extrabold);
}

.standings-launcher-card__facts {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--fi-space-16);
}

.standings-launcher-card__fact {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  min-width: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: 1;
}

.standings-launcher-card__fact--alert {
  color: var(--fi-color-primary);
}

.standings-launcher-card__fact-icon {
  width: 26rpx;
  height: 26rpx;
}

.standings-launcher-card__avatar {
  width: 32rpx;
  height: 32rpx;
}

.standings-launcher-card__fact-text {
  white-space: nowrap;
}

.standings-launcher-card__chevron {
  flex-shrink: 0;
  width: 36rpx;
  height: 36rpx;
}
</style>
