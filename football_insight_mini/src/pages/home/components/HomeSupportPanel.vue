<template>
  <FiPanel clip>
    <!-- 主队队徽斜放作背景水印：与「今日话题」的蓉城水印同一手法（右侧外溢、旋转 12°、低透明度）。 -->
    <image
      v-if="favoriteTeamCrest"
      class="support-home-panel__crest"
      :src="favoriteTeamCrest"
      mode="aspectFit"
      aria-hidden="true"
    />
    <view class="support-home-panel__header">
      <view class="support-home-panel__heading">
        <text class="support-home-panel__title">我的主队</text>
        <view
          v-if="daysRemaining !== null"
          class="support-home-panel__countdown"
          :class="{ 'support-home-panel__countdown--clock': countdownClock }"
        >
          <template v-if="daysRemaining === 0">
            <template v-if="countdownClock">
              <text>距比赛开始</text>
              <template v-for="part in countdownParts" :key="part.unit">
                <text class="support-home-panel__countdown-clock">{{ part.value }}</text><text>{{ part.unit }}</text>
              </template>
            </template>
            <text v-else>今天比赛</text>
          </template>
          <template v-else>
            <text>距下场</text><text class="support-home-panel__countdown-days">{{ daysRemaining }}</text><text>天</text>
          </template>
        </view>
      </view>

      <view v-if="!favoriteTeam" class="support-home-panel__context">
        <view class="support-home-panel__context-dot"></view>
        <text class="support-home-panel__context-label">{{ badge }}</text>
        <text class="support-home-panel__context-note">{{ contextNote }}</text>
      </view>
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
          <image :src="favoriteTeamCrest" mode="aspectFit" class="support-home-panel__favorite-avatar" />
        </view>
        <view class="support-home-panel__favorite-body">
          <text class="support-home-panel__favorite-name">{{ favoriteTeam.team_name }}</text>
          <text class="support-home-panel__favorite-note">{{ favoriteTeamLabel }}</text>
        </view>
        <button class="support-home-panel__switch" :disabled="voteSubmittingTeamId !== null || favoriteSubmitting" @click="emit('select-team')">切换主队</button>
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
            <view class="support-home-match-card__vote">
              <HomeSupportVoteButton
                :team="nextMatch.home_team"
                v-bind="homeVoteState"
                @vote="emit('vote', $event)"
              />
            </view>
          </view>
          <text class="support-home-match-card__vs">{{ windowShortLabel }}</text>
          <view class="support-home-match-card__team-block support-home-match-card__team-block--away">
            <text class="support-home-match-card__team support-home-match-card__team--away">{{ nextMatch.away_team.team_name }}</text>
            <text class="support-home-match-card__rank support-home-match-card__rank--away">{{ awayRankLabel }}</text>
            <view class="support-home-match-card__vote support-home-match-card__vote--away">
              <HomeSupportVoteButton
                :team="nextMatch.away_team"
                v-bind="awayVoteState"
                @vote="emit('vote', $event)"
              />
            </view>
          </view>
        </view>
        <view class="support-home-match-card__bar">
          <view class="support-home-match-card__bar-home" :style="{ width: `${nextMatch.home_team.support_share_pct}%` }" />
          <view class="support-home-match-card__bar-away" :style="{ width: `${nextMatch.away_team.support_share_pct}%` }" />
        </view>
        <view class="support-home-match-card__footer">
          <text>{{ voteHint }}</text>
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
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import { computed } from 'vue'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL } from '../../../config/assets'
import FiLoading from '../../../components/FiLoading.vue'
import HomeSupportVoteButton from './HomeSupportVoteButton.vue'
import type { SupportProfile, SupportTeam } from '../../../types/support'
import {
  resolveHomeSupportMatchWeekdayLabel,
  resolveHomeSupportVoteState,
  resolveHomeSupportMatchCountdownClock,
  resolveHomeSupportMatchDaysRemaining,
  resolveHomeSupportTeamRankLabel,
  resolveHomeSupportWindowShortLabel,
  shouldShowHomeSupportLoading,
} from '../helpers'

const props = defineProps<{
  now: number
  hasAuthToken: boolean
  loading: boolean
  profile: SupportProfile | null
  errorMessage: string
  teams: SupportTeam[]
  voteSubmittingTeamId: number | null
  favoriteSubmitting: boolean
}>()

const emit = defineEmits<{
  (e: 'select-team'): void
  (e: 'open-match'): void
  (e: 'vote', teamId: number): void
}>()

const favoriteTeam = computed(() => props.profile?.favorite_team ?? null)
// 列表头像仅 85px，放大水印使用同款高清队徽，避免拉伸后模糊。
const favoriteTeamCrest = computed(() => favoriteTeam.value?.team_id === 77680
  ? CHENGDU_RONGCHENG_CREST_IMAGE_URL
  : favoriteTeam.value?.avatar_storage_url || '')
const nextMatch = computed(() => props.profile?.next_match ?? null)
const daysRemaining = computed(() => props.hasAuthToken && favoriteTeam.value && !props.errorMessage
  ? resolveHomeSupportMatchDaysRemaining(nextMatch.value, props.now) : null)
const countdownClock = computed(() => daysRemaining.value === 0
  ? resolveHomeSupportMatchCountdownClock(nextMatch.value, props.now) : null)
// 同一天内的倒计时拆成「xx 小时 xx 分 xx 秒」；数字加粗，单位保持常规字重。
const countdownParts = computed(() => {
  const [hours, minutes, seconds] = (countdownClock.value ?? '').split(':')
  return [
    { value: hours, unit: '小时' },
    { value: minutes, unit: '分' },
    { value: seconds, unit: '秒' },
  ]
})
const showLoading = computed(() =>
  shouldShowHomeSupportLoading({ loading: props.loading, hasCachedProfile: !!props.profile }),
)
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
const homeVoteState = computed(() => resolveHomeSupportVoteState(
  nextMatch.value, nextMatch.value?.home_team.team_id ?? 0, props.voteSubmittingTeamId, props.now, props.favoriteSubmitting,
))
const awayVoteState = computed(() => resolveHomeSupportVoteState(
  nextMatch.value, nextMatch.value?.away_team.team_id ?? 0, props.voteSubmittingTeamId, props.now, props.favoriteSubmitting,
))
const voteHint = computed(() => {
  const match = nextMatch.value
  if (!match || match.support_window_status !== 'open' || Date.parse(match.kickoff_at) <= props.now) return '助力已截止'
  return match.viewer.has_supported ? '点击已助力可取消' : '任选一队，每场保留一份助力'
})
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
.support-home-panel__crest {
  position: absolute;
  z-index: 0;
  top: 50%;
  right: -90rpx;
  width: 560rpx;
  height: 560rpx;
  opacity: 0.1;
  transform: translateY(-50%) rotate(12deg);
  pointer-events: none;
}
/* 水印在最底层，内容整体抬到它之上。 */
.support-home-panel__header,
.support-home-panel__summary,
.support-home-panel__favorite,
.support-home-panel__action,
.support-home-match-card {
  position: relative;
  z-index: 1;
}

.support-home-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: 1.7;
}
.support-home-panel__header {
  display: grid;
  gap: var(--fi-space-12);
}
.support-home-panel__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}
.support-home-panel__title {
  color: var(--fi-color-text-strong);
  flex-shrink: 0;
  font-weight: var(--fi-component-title-weight);
  font-size: var(--fi-component-title-size);
  line-height: var(--fi-component-title-leading);
}
.support-home-panel__countdown {
  margin-left: auto;
  display: inline-flex;
  align-items: baseline;
  gap: var(--fi-space-6);
  padding: var(--fi-space-6) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  color: var(--fi-color-primary);
  background: var(--fi-color-topic-audio-surface);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
  white-space: nowrap;
}
/* Match-day clock: the label and HH:MM:SS share one size, only the digits are heavier. */
.support-home-panel__countdown--clock {
  align-items: center;
  gap: var(--fi-space-8);
  font-size: var(--fi-font-26);
}
.support-home-panel__countdown-clock {
  font-weight: var(--fi-weight-extrabold);
  font-variant-numeric: tabular-nums;
  letter-spacing: 1rpx;
}
.support-home-panel__countdown-days {
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
}
.support-home-panel__context {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--fi-space-8);
  padding-top: var(--fi-space-14);
  border-top: 2rpx solid rgba(236, 236, 241, 0.92);
}
.support-home-panel__context-dot {
  width: 10rpx;
  height: 10rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-success-accent);
  box-shadow: 0 0 0 6rpx rgba(38, 162, 105, 0.12);
}
.support-home-panel__context-label {
  color: #2f7f5f;
  font-size: var(--fi-font-24);
  font-weight: 700;
  line-height: 1.2;
}
.support-home-panel__context-note {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: 1.2;
}
.support-home-panel__action {
  margin-top: 18rpx;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-18) var(--fi-space-30);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-26);
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
  gap: var(--fi-space-18);
  padding: var(--fi-space-22) 0 var(--fi-space-24);
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
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-success-accent);
}
.support-home-panel__favorite-avatar-shell {
  margin-left: 18rpx;
  width: 88rpx;
  height: 88rpx;
  border-radius: var(--fi-radius-md);
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
  font-size: var(--fi-font-36);
  line-height: 1;
  font-weight: 800;
}
.support-home-panel__favorite-note {
  display: block;
  margin-top: 6rpx;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}
.support-home-panel__switch {
  padding: var(--fi-space-14) var(--fi-space-22);
  border-radius: var(--fi-radius-round);
  background: #f5f6fa;
  color: #5f6673;
  font-size: var(--fi-font-23);
  font-weight: 700;
  line-height: 1;
}
.support-home-match-card {
  margin-top: 18rpx;
  padding: var(--fi-space-22) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  border: 2rpx solid rgba(229, 231, 236, 0.98);
  /* 半透明白底：让整张卡片的队徽水印能透出来。 */
  background: rgba(255, 255, 255, 0.72);
  box-shadow: 0 10rpx 28rpx rgba(18, 20, 28, 0.04);
}
.support-home-match-card__meta,
.support-home-match-card__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}
.support-home-match-card__meta {
  gap: var(--fi-space-12);
}
.support-home-match-card__meta-pill {
  min-height: 44rpx;
  border-radius: var(--fi-radius-round);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  font-size: var(--fi-font-22);
  line-height: 1;
  font-weight: 800;
}
.support-home-match-card__meta-pill--round {
  padding: 0 var(--fi-space-18);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
}
.support-home-match-card__meta-pill--time {
  margin-left: auto;
  gap: var(--fi-space-10);
  padding: var(--fi-space-6) var(--fi-space-14) var(--fi-space-6) var(--fi-space-8);
  background: #f5f6fa;
  color: #5d6673;
}
.support-home-match-card__weekday {
  min-width: 58rpx;
  padding: var(--fi-space-9) var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  box-sizing: border-box;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-24);
  font-weight: 950;
  line-height: 1;
}
.support-home-match-card__datetime {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  white-space: nowrap;
}
.support-home-match-card__teams {
  margin-top: 18rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: var(--fi-space-16);
  align-items: center;
}
.support-home-match-card__team-block {
  min-width: 0;
  display: grid;
  gap: var(--fi-space-8);
}
.support-home-match-card__team-block--away {
  justify-items: end;
}
.support-home-match-card__vote {
  display: flex;
  width: 100%;
  min-width: 0;
  justify-content: flex-start;
}
.support-home-match-card__vote--away {
  justify-content: flex-end;
}
.support-home-match-card__team {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
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
  padding: 0 var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: #f5f6fa;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: 800;
  line-height: 1;
}
.support-home-match-card__rank--away {
  justify-self: end;
}
.support-home-match-card__vs {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  font-weight: 900;
  line-height: 1;
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-red-rgb), 0.08);
}
.support-home-match-card__bar {
  margin-top: 18rpx;
  height: 12rpx;
  border-radius: var(--fi-radius-round);
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
