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
        <view v-if="daysRemaining !== null || saleDaysRemaining !== null" class="support-home-panel__countdowns">
          <view
            v-if="saleDaysRemaining !== null"
            class="support-home-panel__countdown support-home-panel__countdown--sale"
            :class="{ 'support-home-panel__countdown--clock': saleCountdownClock }"
          >
            <template v-if="saleCountdownClock">
              <text>距抢票开始</text>
              <template v-for="part in saleCountdownParts" :key="part.unit">
                <text class="support-home-panel__countdown-clock">{{ part.value }}</text><text>{{ part.unit }}</text>
              </template>
            </template>
            <template v-else>
              <text>距抢票</text><text class="support-home-panel__countdown-days">{{ saleDaysRemaining }}</text><text>天</text>
            </template>
          </view>
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
        <text class="support-home-panel__favorite-name">{{ favoriteTeam.team_name }}</text>
        <button class="support-home-panel__switch" :disabled="voteSubmittingTeamId !== null || favoriteSubmitting" @click="emit('select-team')">切换主队</button>
      </view>

      <view class="support-home-match-card" @click="emit('open-match')">
        <view class="support-home-match-card__teams">
          <view class="support-home-match-card__team-block">
            <image v-if="homeCrest" :src="homeCrest" mode="aspectFit" class="support-home-match-card__crest" />
            <text class="support-home-match-card__team">{{ nextMatch.home_team.team_name }}</text>
            <text class="support-home-match-card__rank">{{ homeRankLabel }}</text>
            <HomeSupportVoteButton
              :team="nextMatch.home_team"
              v-bind="homeVoteState"
              @vote="emit('vote', $event)"
            />
          </view>
          <view class="support-home-match-card__center">
            <text class="support-home-match-card__round">第 {{ nextMatch.round_number }} 轮</text>
            <text class="support-home-match-card__vs">{{ windowShortLabel }}</text>
            <text class="support-home-match-card__time">{{ weekdayLabel }} {{ nextMatch.match_time }}</text>
            <text class="support-home-match-card__date">{{ nextMatch.match_date }}</text>
          </view>
          <view class="support-home-match-card__team-block">
            <image v-if="awayCrest" :src="awayCrest" mode="aspectFit" class="support-home-match-card__crest" />
            <text class="support-home-match-card__team">{{ nextMatch.away_team.team_name }}</text>
            <text class="support-home-match-card__rank">{{ awayRankLabel }}</text>
            <HomeSupportVoteButton
              :team="nextMatch.away_team"
              v-bind="awayVoteState"
              @vote="emit('vote', $event)"
            />
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

    <!-- 排阵容同样围绕主队，收进本面板底部；加载/出错时不显示。 -->
    <view
      v-if="!showLoading && !errorMessage"
      class="support-home-panel__lineup"
      hover-class="support-home-panel__lineup--pressed"
      @click="emit('open-lineup')"
    >
      <text class="support-home-panel__lineup-title">排{{ favoriteTeam ? `${favoriteTeam.team_name}的` : '我的' }}最佳阵容</text>
      <text class="support-home-panel__lineup-action">前往战术板</text>
    </view>

    <!-- 转会投票同样围绕主队，第二行入口；加载/出错时不显示。 -->
    <view
      v-if="!showLoading && !errorMessage"
      class="support-home-panel__lineup support-home-panel__lineup--follow"
      hover-class="support-home-panel__lineup--pressed"
      @click="emit('open-transfer')"
    >
      <text class="support-home-panel__lineup-title">{{ favoriteTeam ? `${favoriteTeam.team_name}` : '' }}转会投票</text>
      <text class="support-home-panel__lineup-action">去表态</text>
    </view>
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
  resolveHomeTicketSaleCountdownClock,
  resolveHomeTicketSaleDaysRemaining,
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
  /** 下一场的抢票开始时间（余票监控比赛才有）。 */
  saleStartAt: string | null
}>()

const emit = defineEmits<{
  (e: 'select-team'): void
  (e: 'open-match'): void
  (e: 'vote', teamId: number): void
  (e: 'open-lineup'): void
  (e: 'open-transfer'): void
}>()

const favoriteTeam = computed(() => props.profile?.favorite_team ?? null)
// 列表头像仅 85px，放大水印使用同款高清队徽，避免拉伸后模糊。
const favoriteTeamCrest = computed(() => favoriteTeam.value?.team_id === 77680
  ? CHENGDU_RONGCHENG_CREST_IMAGE_URL
  : favoriteTeam.value?.avatar_storage_url || '')
const nextMatch = computed(() => props.profile?.next_match ?? null)
function resolveMatchTeamCrest(team: { team_id: number, avatar_storage_url: string | null } | undefined): string {
  if (!team) return ''
  return team.team_id === 77680 ? CHENGDU_RONGCHENG_CREST_IMAGE_URL : team.avatar_storage_url || ''
}
const homeCrest = computed(() => resolveMatchTeamCrest(nextMatch.value?.home_team))
const awayCrest = computed(() => resolveMatchTeamCrest(nextMatch.value?.away_team))
const daysRemaining = computed(() => props.hasAuthToken && favoriteTeam.value && !props.errorMessage
  ? resolveHomeSupportMatchDaysRemaining(nextMatch.value, props.now) : null)
const countdownClock = computed(() => daysRemaining.value === 0
  ? resolveHomeSupportMatchCountdownClock(nextMatch.value, props.now) : null)
// 同一天内的倒计时拆成「xx 小时 xx 分 xx 秒」；数字加粗，单位保持常规字重。
function splitCountdownClock(clock: string | null) {
  const [hours, minutes, seconds] = (clock ?? '').split(':')
  return [
    { value: hours, unit: '小时' },
    { value: minutes, unit: '分' },
    { value: seconds, unit: '秒' },
  ]
}
const countdownParts = computed(() => splitCountdownClock(countdownClock.value))
// 抢票倒计时只在已展示下一场（登录、有主队、无错误）且该场有开票时间、尚未开票时出现。
const saleDaysRemaining = computed(() => daysRemaining.value !== null
  ? resolveHomeTicketSaleDaysRemaining(props.saleStartAt, props.now) : null)
// 不足 24 小时切换为时分秒（可能跨自然日，所以不按 saleDaysRemaining === 0 判断）。
const saleCountdownClock = computed(() => saleDaysRemaining.value !== null
  ? resolveHomeTicketSaleCountdownClock(props.saleStartAt, props.now) : null)
const saleCountdownParts = computed(() => splitCountdownClock(saleCountdownClock.value))
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
/* 两个倒计时右对齐竖排：抢票在上（墨色，更紧迫），比赛在下。 */
.support-home-panel__countdowns {
  margin-left: auto;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--fi-space-8);
}
.support-home-panel__countdown {
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
.support-home-panel__countdown--sale {
  color: var(--fi-primitive-white);
  background: var(--fi-primitive-ink);
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
/* 主队信息：一行文字，不再用分割线和竖条圈起来；切换主队降级为文字链接。 */
.support-home-panel__favorite {
  margin-top: 14rpx;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-14);
}
/* 排名已在下方对阵里展示，这一行只放主队名和切换入口。 */
.support-home-panel__favorite-name {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  line-height: 1;
  font-weight: 800;
}
.support-home-panel__switch {
  margin: 0;
  padding: 0;
  background: transparent;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  font-weight: 700;
  line-height: 1;
}
.support-home-panel__switch::after { border: 0; }
/* 对阵：左右对称，中间放轮次/VS/时间；无底色、描边、药丸，与整块面板同一层背景。 */
.support-home-match-card {
  margin-top: 26rpx;
}
.support-home-match-card__teams {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 190rpx minmax(0, 1fr);
  align-items: center;
}
.support-home-match-card__team-block {
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.support-home-match-card__crest {
  width: 104rpx;
  height: 104rpx;
}
.support-home-match-card__team {
  max-width: 100%;
  margin-top: var(--fi-space-12);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  line-height: 1.15;
  font-weight: 800;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.support-home-match-card__rank {
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: 1;
}
.support-home-match-card__center {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}
.support-home-match-card__round {
  padding: var(--fi-space-8) var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: 800;
  line-height: 1;
}
.support-home-match-card__vs {
  margin-top: var(--fi-space-14);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-44);
  font-weight: 900;
  font-style: italic;
  letter-spacing: 2rpx;
  line-height: 1;
}
.support-home-match-card__time {
  margin-top: var(--fi-space-14);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: 800;
  line-height: 1;
}
.support-home-match-card__date {
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: 1;
}
.support-home-match-card__bar {
  margin-top: 22rpx;
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
.support-home-match-card__footer {
  margin-top: var(--fi-space-14);
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}
.support-home-panel__lineup {
  position: relative;
  z-index: 1;
  margin-top: var(--fi-space-22);
  padding-top: var(--fi-space-20);
  border-top: var(--fi-border-card);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}
.support-home-panel__lineup--pressed { opacity: var(--fi-opacity-pressed); }
/* 转会投票是第二行入口，跟在排阵容后面，不再重复分割线与上内边距。 */
.support-home-panel__lineup--follow {
  margin-top: var(--fi-space-16);
  padding-top: 0;
  border-top: 0;
}
.support-home-panel__lineup-title {
  min-width: 0;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.support-home-panel__lineup-action {
  flex-shrink: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  font-weight: 700;
}
.support-home-match-card__action {
  color: var(--fi-color-primary);
  font-weight: 800;
}
</style>
