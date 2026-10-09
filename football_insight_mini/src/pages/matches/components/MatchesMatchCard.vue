<template>
<view class="match-card match-card--full" :class="showStatusAccent ? `match-card--${displayStatus}` : ''">
  <view class="match-card__meta">
    <text>{{ match.match_date }}</text>
    <text>{{ match.match_time }}</text>
  </view>
  <view class="match-card__scoreboard">
    <view class="match-card__team">
      <image :src="match.home_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
      <text class="match-card__team-name">{{ match.home_team_name }}</text>
    </view>
    <view class="match-card__score-stack">
      <text class="match-card__score" :class="{ 'match-card__score--upcoming': !finishedResult && !scoreText.includes(' : ') }">{{ scoreText }}</text>
      <text v-if="liveStatus" class="match-card__status-tag">进行中</text>
    </view>
    <view class="match-card__team match-card__team--away">
      <text class="match-card__team-name">{{ match.away_team_name }}</text>
      <image :src="match.away_team_avatar || ''" mode="aspectFit" class="match-card__team-logo" />
    </view>
  </view>
  <view v-if="hasTechStats" class="match-card__footer">
    <view />
    <button class="match-card__tech-link" @click="emit('open-tech-stats', match)">点击查看技术统计</button>
  </view>
</view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MatchCard } from '../../../types/insight'
import { formatMatchScoreboardText, hasMatchTechStats, resolveMatchDisplayStatus, shouldShowLiveStatusTag } from '../helpers'

const props = withDefaults(defineProps<{
  match: MatchCard
  nowIso: string
  showStatusAccent?: boolean
  finishedResult?: boolean
}>(), { showStatusAccent: true, finishedResult: false })
const emit = defineEmits<{ (event: 'open-tech-stats', match: MatchCard): void }>()
const displayStatus = computed(() => resolveMatchDisplayStatus(props.match, props.nowIso))
const scoreText = computed(() => props.finishedResult
  ? `${props.match.home_score} : ${props.match.away_score}`
  : formatMatchScoreboardText(props.match, props.nowIso))
const liveStatus = computed(() => shouldShowLiveStatusTag(props.match, props.nowIso))
const hasTechStats = computed(() => hasMatchTechStats(props.match))
</script>

<style scoped lang="css">
@import '../matches-shared.css';

.match-card__meta { display: flex; align-items: center; justify-content: space-between; }
.match-card {
  position: relative;
  padding: var(--fi-space-18) var(--fi-space-18) var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-lg);
  border: var(--fi-component-matches-match-card-border);
  background: var(--fi-color-ticket-recent-reflux-bucket-lock-body-background-2);
  backdrop-filter: blur(var(--fi-space-14));
  -webkit-backdrop-filter: blur(var(--fi-space-14));
  box-shadow: var(--fi-shadow-match-card-box-shadow);
  overflow: hidden;
}
.match-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: var(--fi-space-5);
  height: 100%;
  background: var(--fi-color-match-card-before-background);
}
.match-card--finished::before {
  background: var(--fi-component-matches-match-card-finished-before-background);
}
.match-card--live::before {
  background: var(--fi-component-matches-match-card-live-before-background);
}
.match-card--live {
  box-shadow: var(--fi-shadow-match-card-live-box-shadow);
}
.match-card--postponed::before {
  background: var(--fi-color-match-card-postponed-before-background);
}
.match-card--scheduled::before {
  background: var(--fi-component-matches-match-card-scheduled-before-background);
}
.match-card__meta text { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.match-card__scoreboard {
  margin-top: var(--fi-space-12);
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: var(--fi-space-18);
  align-items: center;
}
.match-card__team {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
}
.match-card__team-logo {
  width: var(--fi-space-56);
  height: var(--fi-space-56);
  flex-shrink: 0;
}
.match-card__team-name {
  color: var(--fi-color-match-card-team-name-color);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.match-card__team--away {
  justify-content: flex-end;
}
.match-card__score-stack {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-8);
}
.match-card__score {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-56);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-match-card-score-line-height);
}
.match-card__score--upcoming {
  font-size: var(--fi-font-44);
}
.match-card__status-tag {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-8) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-match-card-status-tag-background);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}
.match-card__footer {
  margin-top: var(--fi-space-14);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}
.match-card__tech-link {
  margin: 0 0 0 auto;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-snug);
}
</style>
