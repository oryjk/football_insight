<template>
  <HomeSheetShell animated kicker="球队赛季战绩" @close="emit('close')">
    <template #title>
      <view class="team-season-sheet__title">
        <image :src="team.avatar_storage_url || ''" mode="aspectFit" class="team-season-sheet__title-avatar" />
        <text class="team-season-sheet__title-name">{{ team.team_name }}</text>
      </view>
    </template>

    <view class="team-season-sheet__summary">
      <view class="team-season-sheet__summary-main">
        <view class="team-season-sheet__summary-meta-grid">
          <view class="team-season-sheet__summary-metric">
            <text class="team-season-sheet__summary-value">第 {{ team.rank_no }}</text>
            <text class="team-season-sheet__summary-label">积分榜排名</text>
          </view>
          <view class="team-season-sheet__summary-metric">
            <text class="team-season-sheet__summary-value">{{ team.points }} 分</text>
            <text class="team-season-sheet__summary-label">当前积分</text>
          </view>
          <view class="team-season-sheet__summary-metric">
            <text class="team-season-sheet__summary-value">{{ recordText }}</text>
            <text class="team-season-sheet__summary-label">赛季战绩</text>
          </view>
        </view>
      </view>
    </view>

    <FiLoading
      v-if="loading"
      title="赛季战绩加载中"
      caption="正在整理这支球队本赛季的每场比赛。"
    />

    <view v-else-if="errorMessage" class="state-card state-card--error team-season-sheet__state">
      <text>{{ errorMessage }}</text>
    </view>

    <scroll-view v-else-if="matches.length" scroll-y class="team-season-sheet__list">
      <view
        v-for="(match, index) in matches"
        :key="match.matchId"
        class="team-season-match-row"
        :style="{ '--team-match-delay': `${100 + index * 55}ms` }"
      >
        <view class="team-season-match-row__meta">
          <text>第 {{ match.roundNumber }} 轮 · {{ match.matchDate }} {{ match.matchTime }}</text>
          <view class="team-season-match-row__meta-right">
            <text class="team-season-match-row__venue" :class="match.isHomeTeam ? 'team-season-match-row__venue--home' : 'team-season-match-row__venue--away'">
              {{ match.venueLabel }}
            </text>
            <text class="team-season-match-row__result" :class="`team-season-match-row__result--${match.resultTone}`">
              {{ match.resultLabel }}
            </text>
          </view>
        </view>
        <view class="team-season-match-row__body">
          <view class="team-season-match-row__side team-season-match-row__side--left">
            <image class="team-season-match-row__avatar" :src="match.teamAvatar || ''" mode="aspectFit" />
            <text class="team-season-match-row__team team-season-match-row__team--active">{{ match.teamName }}</text>
          </view>
          <text class="team-season-match-row__score">{{ match.scoreText }}</text>
          <view class="team-season-match-row__side team-season-match-row__side--right">
            <text class="team-season-match-row__team team-season-match-row__team--away">{{ match.opponentName }}</text>
            <image class="team-season-match-row__avatar" :src="match.opponentAvatar || ''" mode="aspectFit" />
          </view>
        </view>
      </view>
    </scroll-view>

    <view v-else class="team-season-sheet__empty">
      <text>这支球队当前还没有可展示的赛季比赛记录。</text>
    </view>
  </HomeSheetShell>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import HomeSheetShell from './HomeSheetShell.vue'
import FiLoading from '../../../components/FiLoading.vue'
import type { OverviewStanding } from '../../../types/insight'
import { formatHomeTeamSeasonRecord, type HomeTeamSeasonMatch } from '../helpers'

const props = defineProps<{
  team: OverviewStanding
  matches: HomeTeamSeasonMatch[]
  loading: boolean
  errorMessage: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const recordText = computed(() => formatHomeTeamSeasonRecord(props.matches))
</script>

<style scoped lang="css">
.state-card--error text {
  font-size: 28rpx;
  color: #c03a2b;
}

.team-season-sheet__title {
  display: flex;
  align-items: center;
  gap: 14rpx;
  min-width: 0;
  margin-top: 8rpx;
}

.team-season-sheet__title-avatar {
  width: 56rpx;
  height: 56rpx;
  flex: 0 0 auto;
}

.team-season-sheet__title-name {
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: 44rpx;
  line-height: 1.16;
  font-weight: 800;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.team-season-sheet__summary {
  margin-top: 18rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: var(--fi-primitive-white);
  display: block;
}

.team-season-sheet__summary-main {
  display: block;
  min-width: 0;
}

.team-season-sheet__summary-meta-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.9fr) minmax(0, 1fr) minmax(160rpx, 1.45fr);
  align-items: center;
  gap: 12rpx;
  min-width: 0;
}

.team-season-sheet__summary-metric {
  display: grid;
  gap: 6rpx;
  min-width: 0;
  padding-left: 14rpx;
  border-left: 2rpx solid rgba(235, 236, 241, 0.95);
}

.team-season-sheet__summary-metric:first-child {
  border-left: 0;
  padding-left: 0;
}

.team-season-sheet__summary-value {
  color: var(--fi-color-text-strong);
  font-size: 28rpx;
  line-height: 1.1;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-sheet__summary-label {
  color: var(--fi-color-text-muted);
  font-size: 20rpx;
  line-height: 1.15;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-sheet__list {
  margin-top: 22rpx;
  max-height: 52vh;
}

.team-season-sheet__state,
.team-season-sheet__empty {
  margin-top: 22rpx;
}

.team-season-sheet__empty {
  padding: 28rpx 12rpx;
  color: var(--fi-color-text-muted);
  font-size: 26rpx;
  text-align: center;
}

.team-season-match-row {
  padding: 20rpx 0;
  border-top: 2rpx solid #f0f1f5;
  display: grid;
  gap: 14rpx;
  opacity: 0;
  transform: translateY(14rpx);
  animation: team-season-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--team-match-delay, 100ms);
}

.team-season-match-row:first-child {
  border-top: none;
}

.team-season-match-row__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}
.team-season-match-row__meta-right {
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
}
.team-season-match-row__venue {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}
.team-season-match-row__venue--home {
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
  color: var(--fi-color-primary);
}
.team-season-match-row__venue--away {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 56rpx;
  padding: 4rpx 12rpx;
  border-radius: 999rpx;
  font-size: 20rpx;
  font-weight: 800;
}

.team-season-match-row__result--win {
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
  color: var(--fi-color-primary);
}

.team-season-match-row__result--draw {
  background: rgba(234, 179, 8, 0.12);
  color: #854d0e;
}

.team-season-match-row__result--loss {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.team-season-match-row__result--live {
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
  color: var(--fi-color-primary);
}

.team-season-match-row__result--scheduled {
  background: rgba(59, 130, 246, 0.12);
  color: #2563eb;
}

.team-season-match-row__body {
  display: grid;
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  gap: 18rpx;
}

.team-season-match-row__side {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.team-season-match-row__side--left {
  justify-content: flex-start;
}
.team-season-match-row__side--right {
  justify-content: flex-end;
}

.team-season-match-row__avatar {
  width: 36rpx;
  height: 36rpx;
  flex-shrink: 0;
}

.team-season-match-row__team {
  color: #7b818d;
  font-size: 28rpx;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.team-season-match-row__team--active {
  color: var(--fi-color-text-strong);
}

.team-season-match-row__score {
  color: var(--fi-color-text-strong);
  font-size: 40rpx;
  line-height: 1;
  font-weight: 800;
}

@keyframes team-season-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
