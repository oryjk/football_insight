<template>
  <HomePanelCard>
    <template #heading>
      <view class="standings__heading">
        <text class="standings__kicker">积分榜头部</text>
        <text class="standings__title">先看联赛头部格局</text>
      </view>
    </template>
    <view class="standings__caption">
      <text>近 6 场赛果 · 从左到右由旧到新</text>
      <text>积分</text>
    </view>
    <view class="standings__list">
      <view
        v-for="team in teams"
        :key="team.team_id"
        class="standings__row"
        role="button"
        :aria-label="`${team.team_name}，${team.points}积分，查看完整战绩`"
        hover-class="standings__row--pressed"
        hover-stay-time="100"
        @click="emit('select', { key: team.team_id })"
      >
        <text class="standings__rank" :class="{ 'standings__rank--first': team.rank_no === 1 }">{{ team.rank_no }}</text>
        <image class="standings__avatar" :src="team.avatar_storage_url || ''" mode="aspectFit" />
        <text class="standings__name">{{ team.team_name }}</text>
        <text class="standings__points">{{ team.points }}</text>
        <view class="standings__form">
          <HomeTeamFormTrend :outcomes="team.recent_form" />
        </view>
      </view>
    </view>
    <view class="standings__footer">
      <text>上胜 · 中平 · 下负</text>
      <text>点击球队查看完整战绩</text>
    </view>
  </HomePanelCard>
</template>

<script setup lang="ts">
import HomePanelCard from './HomePanelCard.vue'
import HomeTeamFormTrend from './HomeTeamFormTrend.vue'
import type { OverviewStanding } from '../../../types/insight'

defineProps<{ teams: OverviewStanding[] }>()
const emit = defineEmits<{ (event: 'select', row: { key: number }): void }>()
</script>

<style scoped>
.standings__heading {
  display: grid;
  gap: var(--fi-space-8);
  min-width: 0;
}
.standings__kicker {
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-medium);
  color: var(--fi-color-primary);
  letter-spacing: var(--fi-space-2);
}
.standings__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-40);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}
.standings__caption,
.standings__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}
.standings__caption {
  margin-top: var(--fi-space-18);
  padding-bottom: var(--fi-space-12);
  border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft);
}
.standings__row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-56) minmax(0, 1fr) var(--fi-space-72);
  align-items: center;
  column-gap: var(--fi-space-16);
  padding: var(--fi-space-20) var(--fi-space-4) var(--fi-space-16);
  border-bottom: var(--fi-border-width) solid var(--fi-color-divider-soft);
}
.standings__row--pressed {
  background: var(--fi-color-team-form-pressed);
}
.standings__rank {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-medium);
  font-variant-numeric: tabular-nums;
}
.standings__rank--first {
  color: var(--fi-color-primary);
}
.standings__avatar {
  width: var(--fi-space-56);
  height: var(--fi-space-56);
}
.standings__name {
  min-width: 0;
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
  color: var(--fi-color-text-strong);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.standings__points {
  font-size: var(--fi-font-44);
  line-height: var(--fi-leading-tight);
  font-weight: var(--fi-weight-bold);
  color: var(--fi-color-text-strong);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.standings__form {
  grid-column: 3 / 5;
  min-width: 0;
  margin-top: var(--fi-space-6);
}
.standings__footer {
  margin-top: var(--fi-space-16);
  flex-wrap: wrap;
  gap: var(--fi-space-8);
}
</style>
