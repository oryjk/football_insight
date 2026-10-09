<template>
<view class="panel">
  <view class="section-heading">
    <view>
      <text class="section-kicker">Upcoming</text>
      <text class="section-title">即将到来</text>
    </view>
  </view>
  <view v-if="sections.length" class="upcoming-sections">
    <view v-for="section in sections" :key="section.title" class="upcoming-round">
      <view class="section-heading section-heading--compact">
        <view>
          <text class="section-kicker">{{ section.title }}</text>
          <text class="section-title">第 {{ section.roundNumber }} 轮</text>
        </view>
        <button class="meta-pill meta-pill--button" @click="emit('open-round', section.roundNumber)">查看整轮</button>
      </view>
      <view class="match-stack">
        <view v-for="match in section.matches" :key="match.match_id">
          <MatchesMatchCard :match="match" :now-iso="nowIso" @open-tech-stats="emit('open-tech-stats', $event)" />
        </view>
      </view>
    </view>
  </view>
  <view v-else class="state-card state-card--empty">
    <text>当前没有可展示的即将到来比赛。</text>
  </view>
</view>
</template>

<script setup lang="ts">
import type { MatchCard } from '../../../types/insight'
import type { UpcomingSection } from '../types'
import MatchesMatchCard from './MatchesMatchCard.vue'

defineProps<{ sections: UpcomingSection[]; nowIso: string }>()
const emit = defineEmits<{
  (event: 'open-round', roundNumber: number): void
  (event: 'open-tech-stats', match: MatchCard): void
}>()
</script>

<style scoped lang="css">
@import '../matches-shared.css';

.upcoming-sections {
  margin-top: var(--fi-space-14);
  display: grid;
  gap: var(--fi-space-12);
}
</style>
