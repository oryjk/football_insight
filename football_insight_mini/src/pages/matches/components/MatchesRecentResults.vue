<template>
<view class="recent-results">
  <view v-for="group in groups" :key="group.roundNumber" class="panel">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">最近完赛</text>
        <text class="section-title">第 {{ group.roundNumber }} 轮</text>
      </view>
    </view>
    <view class="match-stack">
      <view v-for="match in group.items" :key="match.match_id">
        <MatchesMatchCard :match="match" :now-iso="nowIso" finished-result @open-tech-stats="emit('open-tech-stats', $event)" />
      </view>
    </view>
  </view>
</view>
</template>

<script setup lang="ts">
import type { MatchCard } from '../../../types/insight'
import type { MatchRoundGroup } from '../types'
import MatchesMatchCard from './MatchesMatchCard.vue'

defineProps<{ groups: MatchRoundGroup[]; nowIso: string }>()
const emit = defineEmits<{ (event: 'open-tech-stats', match: MatchCard): void }>()
</script>

<style scoped lang="css">
@import '../matches-shared.css';

.recent-results { display: flex; flex-direction: column; gap: var(--fi-space-16); }
</style>
