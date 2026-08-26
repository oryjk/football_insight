<template>
  <view class="panel support-home-panel">
    <view class="support-home-panel__header">
      <view class="support-home-panel__heading">
        <text class="support-home-panel__title">我的主队</text>
      </view>

      <view v-if="!favoriteTeam" class="support-home-panel__context">
        <view class="support-home-panel__context-dot"></view>
        <text class="support-home-panel__context-label">{{ badge }}</text>
        <text class="support-home-panel__context-note">{{ contextNote }}</text>
      </view>
      <text v-if="refreshing" class="support-home-panel__refreshing">更新中</text>
    </view>

    <FiLoading
      v-if="showLoading"
      title="助力入口加载中"
      caption="正在确认你的主队和下一场比赛。"
    />

    <view v-else-if="errorMessage" class="state-card state-card--error">
      <text>{{ errorMessage }}</text>
    </view>

    <template v-else-if="!hasAuthToken">
      <text class="support-home-panel__summary">
        登录后才能关注主队、参与赛前助力，并把比赛页面转发出去拉票。
      </text>
    </template>

    <template v-else-if="!favoriteTeam">
      <text class="support-home-panel__summary">
        先选择一支主队，首页就会把它的下一场比赛和助力入口放到第一屏。
      </text>
      <button class="support-home-panel__action" @click="emit('select-team')">选择主队</button>
    </template>

    <template v-else-if="nextMatch">
      <view class="support-home-panel__favorite">
        <view class="support-home-panel__favorite-avatar-shell">
          <image :src="favoriteTeam.avatar_storage_url || ''" mode="aspectFit" class="support-home-panel__favorite-avatar" />
        </view>
        <view class="support-home-panel__favorite-body">
          <text class="support-home-panel__favorite-name">{{ favoriteTeam.team_name }}</text>
          <text class="support-home-panel__favorite-note">{{ favoriteTeamLabel }}</text>
        </view>
        <button class="support-home-panel__switch" @click="emit('select-team')">切换主队</button>
      </view>

      <view class="support-home-match-card" @click="emit('open-match')">
        <view class="support-home-match-card__meta">
          <view class="support-home-match-card__meta-pill support-home-match-card__meta-pill--round">
            <text>第 {{ nextMatch.round_number }} 轮</text>
          </view>
          <view class="support-home-match-card__meta-pill support-home-match-card__meta-pill--time">
            <text v-if="weekdayLabel" class="support-home-match-card__weekday">{{ weekdayLabel }}</text>
            <view class="support-home-match-card__datetime">
              <text>{{ nextMatch.match_date }}</text>
              <text>{{ nextMatch.match_time }}</text>
            </view>
          </view>
        </view>
        <view class="support-home-match-card__teams">
          <view class="support-home-match-card__team-block">
            <text class="support-home-match-card__team">{{ nextMatch.home_team.team_name }}</text>
            <text class="support-home-match-card__rank">{{ homeRankLabel }}</text>
          </view>
          <text class="support-home-match-card__vs">{{ windowShortLabel }}</text>
          <view class="support-home-match-card__team-block support-home-match-card__team-block--away">
            <text class="support-home-match-card__team support-home-match-card__team--away">{{ nextMatch.away_team.team_name }}</text>
            <text class="support-home-match-card__rank support-home-match-card__rank--away">{{ awayRankLabel }}</text>
          </view>
        </view>
        <view class="support-home-match-card__bar">
          <view class="support-home-match-card__bar-home" :style="{ width: `${nextMatch.home_team.support_share_pct}%` }" />
          <view class="support-home-match-card__bar-away" :style="{ width: `${nextMatch.away_team.support_share_pct}%` }" />
        </view>
        <view class="support-home-match-card__footer">
          <text>{{ nextMatchLabel }}</text>
          <text class="support-home-match-card__action">进入助力页</text>
        </view>
      </view>
    </template>

    <template v-else>
      <text class="support-home-panel__summary">
        {{ favoriteTeam.team_name }} 当前还没有可展示的下一场助力比赛，等赛程刷新后这里会自动出现。
      </text>
      <button class="support-home-panel__action support-home-panel__action--ghost" @click="emit('select-team')">切换主队</button>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiLoading from '../../../components/FiLoading.vue'
import type { SupportProfile, SupportTeam } from '../../../types/support'
import {
  resolveHomeSupportMatchWeekdayLabel,
  resolveHomeSupportNextMatchLabel,
  resolveHomeSupportTeamRankLabel,
  resolveHomeSupportWindowShortLabel,
  shouldShowHomeSupportLoading,
} from '../helpers'

const props = defineProps<{
  hasAuthToken: boolean
  loading: boolean
  profile: SupportProfile | null
  errorMessage: string
  teams: SupportTeam[]
}>()

const emit = defineEmits<{
  (e: 'select-team'): void
  (e: 'open-match'): void
}>()

const favoriteTeam = computed(() => props.profile?.favorite_team ?? null)
const nextMatch = computed(() => props.profile?.next_match ?? null)
const showLoading = computed(() =>
  shouldShowHomeSupportLoading({ loading: props.loading, hasCachedProfile: !!props.profile }),
)
const refreshing = computed(() => props.loading && !!props.profile)
const badge = computed(() => {
  if (!props.hasAuthToken) {
    return '登录后开启'
  }

  return favoriteTeam.value?.team_name ?? '待选主队'
})
const contextNote = computed(() => {
  if (!props.hasAuthToken) {
    return '登录后查看主队助力入口'
  }

  if (!favoriteTeam.value) {
    return '先选主队，再把比赛入口固定到首页'
  }

  return '当前关注球队'
})
const favoriteTeamLabel = computed(() => {
  if (!favoriteTeam.value) {
    return ''
  }

  return favoriteTeam.value.rank_no
    ? `当前积分榜第 ${favoriteTeam.value.rank_no}`
    : '已关注主队'
})
const windowShortLabel = computed(() => resolveHomeSupportWindowShortLabel(nextMatch.value))
const nextMatchLabel = computed(() => resolveHomeSupportNextMatchLabel(nextMatch.value))
const weekdayLabel = computed(() => resolveHomeSupportMatchWeekdayLabel(nextMatch.value?.match_date))
const teamRankMap = computed(() => new Map(props.teams.map((team) => [team.team_id, team.rank_no])))
const homeRankLabel = computed(() => {
  const teamId = nextMatch.value?.home_team.team_id
  return resolveHomeSupportTeamRankLabel(teamId ? teamRankMap.value.get(teamId) : null)
})
const awayRankLabel = computed(() => {
  const teamId = nextMatch.value?.away_team.team_id
  return resolveHomeSupportTeamRankLabel(teamId ? teamRankMap.value.get(teamId) : null)
})
</script>

<style scoped lang="css">
.panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: 36rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
  padding: 20rpx;
}

.state-card--error text {
  font-size: 28rpx;
  color: #c03a2b;
}

.support-home-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.support-home-panel__header {
  display: grid;
  gap: 12rpx;
}
.support-home-panel__heading {
  display: grid;
  gap: 0;
}
.support-home-panel__title {
  color: var(--fi-color-text-strong);
  font-weight: 800;
  font-size: 36rpx;
  line-height: 1.12;
}
.support-home-panel__context {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8rpx;
  padding-top: 14rpx;
  border-top: 2rpx solid rgba(236, 236, 241, 0.92);
}
.support-home-panel__context-dot {
  width: 10rpx;
  height: 10rpx;
  border-radius: 999rpx;
  background: var(--fi-color-success-accent);
  box-shadow: 0 0 0 6rpx rgba(38, 162, 105, 0.12);
}
.support-home-panel__context-label {
  color: #2f7f5f;
  font-size: 24rpx;
  font-weight: 700;
  line-height: 1.2;
}
.support-home-panel__context-note {
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  line-height: 1.2;
}
.support-home-panel__refreshing {
  justify-self: start;
  color: var(--fi-color-text-secondary);
  font-size: 20rpx;
  line-height: 1;
  padding: 8rpx 14rpx;
  border-radius: 999rpx;
  background: var(--fi-color-page);
}
.support-home-panel__action {
  margin-top: 18rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 18rpx 30rpx;
  border-radius: 999rpx;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: 26rpx;
  line-height: 1;
}
.support-home-panel__action--ghost {
  background: #f5f6fa;
  color: #5f6673;
}
.support-home-panel__favorite {
  position: relative;
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 18rpx;
  padding: 22rpx 0 24rpx;
  border-top: 2rpx solid rgba(231, 232, 238, 0.92);
  border-bottom: 2rpx solid rgba(231, 232, 238, 0.92);
}
.support-home-panel__favorite::before {
  content: '';
  position: absolute;
  left: 0;
  top: 22rpx;
  bottom: 22rpx;
  width: 4rpx;
  border-radius: 999rpx;
  background: var(--fi-color-success-accent);
}
.support-home-panel__favorite-avatar-shell {
  margin-left: 18rpx;
  width: 88rpx;
  height: 88rpx;
  border-radius: 24rpx;
  background: #f7f9f8;
  border: 2rpx solid rgba(223, 229, 226, 0.96);
  display: flex;
  align-items: center;
  justify-content: center;
}
.support-home-panel__favorite-avatar {
  width: 72rpx;
  height: 72rpx;
}
.support-home-panel__favorite-body {
  flex: 1;
  min-width: 0;
}
.support-home-panel__favorite-name {
  color: var(--fi-color-text-strong);
  display: block;
  font-size: 36rpx;
  line-height: 1;
  font-weight: 800;
}
.support-home-panel__favorite-note {
  display: block;
  margin-top: 6rpx;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}
.support-home-panel__switch {
  padding: 14rpx 22rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  color: #5f6673;
  font-size: 23rpx;
  font-weight: 700;
  line-height: 1;
}
.support-home-match-card {
  margin-top: 18rpx;
  padding: 22rpx 20rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  background: var(--fi-primitive-white);
  box-shadow: 0 10rpx 28rpx rgba(18, 20, 28, 0.04);
}
.support-home-match-card__meta,
.support-home-match-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}
.support-home-match-card__meta {
  gap: 12rpx;
}
.support-home-match-card__meta-pill {
  min-height: 44rpx;
  border-radius: 999rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  font-size: 22rpx;
  line-height: 1;
  font-weight: 800;
}
.support-home-match-card__meta-pill--round {
  padding: 0 18rpx;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
}
.support-home-match-card__meta-pill--time {
  margin-left: auto;
  gap: 10rpx;
  padding: 6rpx 14rpx 6rpx 8rpx;
  background: #f5f6fa;
  color: #5d6673;
}
.support-home-match-card__weekday {
  min-width: 58rpx;
  padding: 9rpx 12rpx;
  border-radius: 999rpx;
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: 24rpx;
  font-weight: 950;
  line-height: 1;
}
.support-home-match-card__datetime {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8rpx;
  white-space: nowrap;
}
.support-home-match-card__teams {
  margin-top: 18rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 16rpx;
  align-items: center;
}
.support-home-match-card__team-block {
  min-width: 0;
  display: grid;
  gap: 8rpx;
}
.support-home-match-card__team-block--away {
  justify-items: end;
}
.support-home-match-card__team {
  color: var(--fi-color-text-strong);
  font-size: 32rpx;
  line-height: 1.12;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.support-home-match-card__team--away {
  text-align: right;
}
.support-home-match-card__rank {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  justify-self: start;
  min-height: 32rpx;
  padding: 0 12rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
  color: var(--fi-color-text-secondary);
  font-size: 20rpx;
  font-weight: 800;
  line-height: 1;
}
.support-home-match-card__rank--away {
  justify-self: end;
}
.support-home-match-card__vs {
  color: var(--fi-color-primary);
  font-size: 24rpx;
  font-weight: 900;
  line-height: 1;
  padding: 8rpx 14rpx;
  border-radius: 999rpx;
  background: rgba(var(--fi-primitive-red-rgb), 0.08);
}
.support-home-match-card__bar {
  margin-top: 18rpx;
  height: 12rpx;
  border-radius: 999rpx;
  overflow: hidden;
  background: #eef0f4;
  display: flex;
}
.support-home-match-card__bar-home {
  height: 100%;
  background: var(--fi-color-success-accent);
}
.support-home-match-card__bar-away {
  height: 100%;
  background: #a7adb8;
}
.support-home-match-card__action {
  color: var(--fi-color-primary);
  font-weight: 800;
}
</style>
