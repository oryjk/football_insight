<template>
  <HomePanelCard>
    <template #heading>
      <view class="standings__heading">
        <text class="standings__kicker">积分榜头部</text>
        <text class="standings__title">先看联赛头部格局</text>
      </view>
    </template>
    <FiStandingsTable :teams="teams" @select="emit('select', $event)" />
  </HomePanelCard>
</template>

<script setup lang="ts">
import HomePanelCard from './HomePanelCard.vue'
import FiStandingsTable from '../../../components/FiStandingsTable.vue'
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
</style>
