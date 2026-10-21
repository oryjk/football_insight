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
      <view class="standings-launcher-card__header">
        <view class="standings-launcher-card__icon" :class="{ 'standings-launcher-card__icon--penalty': isPenalty(table) }">
          <image :src="isPenalty(table) ? penaltyIcon : theoryIcon" class="standings-launcher-card__icon-img" mode="aspectFit" />
        </view>
        <text class="standings-launcher-card__title">{{ table.label }}</text>
        <view v-if="isPenalty(table) && facts(table).adjustedCount > 0" class="standings-launcher-card__alert">
          <image :src="alertIcon" class="standings-launcher-card__alert-icon" mode="aspectFit" />
          <text>{{ facts(table).adjustedCount }} 队</text>
        </view>
        <image :src="chevronIcon" class="standings-launcher-card__chevron" mode="aspectFit" />
      </view>

      <view class="standings-launcher-card__podium">
        <view v-for="item in facts(table).podium" :key="item.teamId" class="standings-launcher-card__place" :class="item.rank <= 2 ? `standings-launcher-card__place--acl-${item.rank}` : ''">
          <text class="standings-launcher-card__rank" :class="`standings-launcher-card__rank--${item.rank}`">{{ item.rank }}</text>
          <image v-if="item.avatar" :src="item.avatar" class="standings-launcher-card__avatar" mode="aspectFit" />
          <text class="standings-launcher-card__name">{{ item.teamName }}</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import penaltyIcon from '../../../static/rankings/standings-penalty.svg'
import theoryIcon from '../../../static/rankings/standings-theory.svg'
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
  display: grid;
  gap: var(--fi-space-16);
  padding: var(--fi-space-20) 0;
  border-bottom: var(--fi-border-divider);
}

.standings-launcher-card:last-child {
  border-bottom: 0;
}

.standings-launcher-card--pressed {
  opacity: 0.7;
}

.standings-launcher-card__header {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.standings-launcher-card__icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64rpx;
  height: 64rpx;
  border-radius: 20rpx;
  background: var(--fi-color-page);
}

.standings-launcher-card__icon--penalty {
  background: rgba(var(--fi-primitive-red-rgb), 0.1);
}

.standings-launcher-card__icon-img {
  width: 36rpx;
  height: 36rpx;
}

.standings-launcher-card__title {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: 1.18;
  font-weight: var(--fi-weight-extrabold);
}

.standings-launcher-card__alert {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: 1;
}

.standings-launcher-card__alert-icon {
  width: 26rpx;
  height: 26rpx;
}

.standings-launcher-card__chevron {
  flex-shrink: 0;
  width: 36rpx;
  height: 36rpx;
}

.standings-launcher-card__podium {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-12);
}

.standings-launcher-card__place {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  min-width: 0;
  padding: var(--fi-space-12);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.standings-launcher-card__place--acl-1 { background: linear-gradient(90deg, rgba(212, 160, 23, 0.2), rgba(212, 160, 23, 0.04)); box-shadow: inset 4rpx 0 0 #d4a017; }
.standings-launcher-card__place--acl-2 { background: linear-gradient(90deg, rgba(84, 118, 168, 0.18), rgba(84, 118, 168, 0.04)); box-shadow: inset 4rpx 0 0 #6b8cc0; }

.standings-launcher-card__rank {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32rpx;
  height: 32rpx;
  border-radius: var(--fi-radius-round);
  background: #9ca3af;
  color: #ffffff;
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-extrabold);
  line-height: 1;
}

.standings-launcher-card__rank--1 {
  background: #d4a017;
}

.standings-launcher-card__rank--3 {
  background: #b45309;
}

.standings-launcher-card__avatar {
  flex-shrink: 0;
  width: 40rpx;
  height: 40rpx;
}

.standings-launcher-card__name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: 1.2;
}
</style>
