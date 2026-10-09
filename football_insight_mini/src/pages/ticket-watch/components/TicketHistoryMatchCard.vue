<template>
  <view class="panel watch-match-card">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">Selected Match</text>
        <text class="section-title"
          >{{ match.home_team_name }} VS {{ match.away_team_name }}</text
        >
      </view>
      <text class="meta-pill">{{
        switching ? '切换中' : `第 ${match.round_number} 轮`
      }}</text>
    </view>

    <view class="watch-match-card__meta">
      <text>{{ match.match_date }} {{ match.match_time }}</text>
      <text>{{ totalOccurrences }} 张累计回流</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchMatchSummary } from '../../../types/ticketWatch'

defineProps<{
  match: TicketWatchMatchSummary
  switching: boolean
  totalOccurrences: number
}>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.watch-match-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
}

.watch-match-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--fi-color-ticket-watch-match-card-meta-color);
  font-size: var(--fi-font-22);
}
</style>
