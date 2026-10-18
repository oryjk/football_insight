<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBackFromSecondaryPage()" @share="copyPageShareLink(detail ? `/pages/support/index?matchId=${detail.match_id}` : '/pages/home/index')" />
  <view class="page-root">
    <image class="page-bg-img" :src="phoenixStadiumBgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
      <FiPanel>
        <FiSectionHeading kicker="赛前助力" :title="heroTitle" size="xl">
          <template #meta>
            <text class="meta-pill">{{ heroBadge }}</text>
          </template>
        </FiSectionHeading>

        <text class="hero-summary">{{ heroSummary }}</text>
      </FiPanel>

      <FiLoading
        v-if="loading"
        title="助力页加载中"
        caption="正在整理这场比赛的双方热度对抗。"
      />

      <FiPanel v-else-if="errorMessage">
        <text class="state-error">{{ errorMessage }}</text>
      </FiPanel>

      <template v-else-if="detail">
        <FiPanel>
          <view class="support-scoreboard__meta">
            <text>第 {{ detail.round_number }} 轮</text>
            <text>{{ detail.match_date }} {{ detail.match_time }}</text>
          </view>

          <view class="support-scoreboard__teams">
            <view class="support-team-card" :class="{ 'support-team-card--favorite': favoriteTeamId === detail.home_team.team_id }">
              <image :src="detail.home_team.avatar_storage_url || ''" mode="aspectFit" class="support-team-card__avatar" />
              <text class="support-team-card__name">{{ detail.home_team.team_name }}</text>
              <text class="support-team-card__count">{{ formatSupportCount(detail.home_team.support_count) }}</text>
              <text class="support-team-card__share">{{ detail.home_team.support_share_pct }}%</text>
            </view>

            <view class="support-scoreboard__vs">
              <text class="support-scoreboard__vs-mark">VS</text>
              <text class="support-scoreboard__vs-state">{{ supportWindowLabel }}</text>
            </view>

            <view class="support-team-card support-team-card--away" :class="{ 'support-team-card--favorite': favoriteTeamId === detail.away_team.team_id }">
              <image :src="detail.away_team.avatar_storage_url || ''" mode="aspectFit" class="support-team-card__avatar" />
              <text class="support-team-card__name">{{ detail.away_team.team_name }}</text>
              <text class="support-team-card__count">{{ formatSupportCount(detail.away_team.support_count) }}</text>
              <text class="support-team-card__share">{{ detail.away_team.support_share_pct }}%</text>
            </view>
          </view>

          <view class="support-bar">
            <view
              class="support-bar__home"
              :style="{ width: `${detail.home_team.support_share_pct}%` }"
            />
            <view
              class="support-bar__away"
              :style="{ width: `${detail.away_team.support_share_pct}%` }"
            />
          </view>

          <view class="support-scoreboard__footer">
            <text>总助力值 {{ formatSupportCount(detail.total_support_count) }}</text>
            <text>{{ countdownLabel }}</text>
          </view>
        </FiPanel>

        <FiPanel>
          <FiSectionHeading kicker="站队状态" :title="stanceTitle">
            <template #meta>
              <text class="meta-pill">{{ stanceBadge }}</text>
            </template>
          </FiSectionHeading>

          <view class="support-state-card__notes">
            <text>{{ stanceDescription }}</text>
            <text>{{ rankingNote }}</text>
          </view>

          <view class="support-actions">
            <button
              v-for="team in matchTeams"
              :key="team.team_id"
              class="primary-action support-actions__primary"
              :class="{ 'primary-action--disabled': !canTakeVoteAction(team.team_id) }"
              :disabled="!canTakeVoteAction(team.team_id)"
              @click="handleVote(team.team_id)"
            >
              {{ voteButtonLabel(team) }}
            </button>
            <button class="secondary-action" open-type="share">
              转发拉票
            </button>
            <button class="secondary-action" :disabled="posterGenerating" @click="generatePoster">
              {{ posterGenerating ? '生成中' : '生成拉票海报' }}
            </button>
          </view>

          <text class="support-state-card__rule">
            规则：主客队任选，开球前可点击已助力取消并重新选择；同一时间只能保留一队的助力，比赛开始后自动截止。
          </text>
        </FiPanel>

        <FiPanel v-if="posterImagePath">
          <FiSectionHeading kicker="拉票海报" title="已生成本地海报" />

          <image :src="posterImagePath" mode="widthFix" class="poster-panel__image" />
          <button class="primary-action poster-panel__save" @click="savePosterImage">保存到相册</button>
        </FiPanel>
      </template>

      <canvas canvas-id="supportPosterCanvas" class="support-poster-canvas" />
      </view>
    </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { buildSharePath } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, ref } from 'vue'
import { onHide, onLoad, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import FiPanel from '../../components/FiPanel.vue'
import FiSectionHeading from '../../components/FiSectionHeading.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { PHOENIX_STADIUM_BG_IMAGE_URL as phoenixStadiumBgImage } from '../../config/assets'
import type { SupportMatchTeam } from '../../types/support'
import { setPostLoginRedirectTarget } from '../../utils/authStorage'
import { savePosterToAlbum } from '../../utils/canvasPoster'
import { useAppSession } from '../../stores/appSession'
import {
  resolveSupportCountdownLabel,
  resolveSupportHeroBadge,
  resolveSupportHeroSummary,
  resolveSupportStanceDescription,
  resolveSupportWindowLabel,
} from './helpers'
import { useSupportPage } from './useSupportPage'
import { useSupportPoster } from './useSupportPoster'

const { navigationPageStyle } = useNavigationLayout()

// 登录标记共享自 appSession，退出/换号后自动更新。
const session = useAppSession()
const matchId = ref<number | null>(null)
const poster = useSupportPoster(session, matchId)
const { posterImagePath, posterGenerating } = poster
const {
  loading,
  errorMessage,
  detail,
  supportedTeam,
  voteSubmittingTeamId,
  liveCountdownSeconds,
  setMatchId,
  voteState,
  loadPage,
  submitVote,
  pausePage,
} = useSupportPage({ session, matchId, onDetailInvalidated: poster.invalidatePoster })

const matchTeams = computed(() => detail.value ? [detail.value.home_team, detail.value.away_team] : [])

const favoriteTeamId = computed(() => detail.value?.viewer.favorite_team_id ?? null)
const favoriteTeam = computed(() => {
  if (!detail.value || favoriteTeamId.value === null) {
    return null
  }

  if (detail.value.home_team.team_id === favoriteTeamId.value) {
    return detail.value.home_team
  }

  if (detail.value.away_team.team_id === favoriteTeamId.value) {
    return detail.value.away_team
  }

  return null
})

const heroTitle = computed(() => {
  if (!detail.value) {
    return '两队赛前热度对抗'
  }

  return `${detail.value.home_team.team_name} VS ${detail.value.away_team.team_name}`
})

const heroBadge = computed(() => {
  return resolveSupportHeroBadge(detail.value)
})

const heroSummary = computed(() => {
  return resolveSupportHeroSummary(detail.value)
})

const supportWindowLabel = computed(() => {
  return resolveSupportWindowLabel(detail.value)
})

const countdownLabel = computed(() => {
  if (!detail.value) {
    return ''
  }

  return resolveSupportCountdownLabel({
    ...detail.value,
    countdown_seconds: liveCountdownSeconds.value,
  })
})

const stanceTitle = computed(() => {
  if (!detail.value) {
    return '站队状态'
  }

  if (supportedTeam.value) {
    return `你已为 ${supportedTeam.value.team_name} 助力`
  }

  if (favoriteTeam.value) {
    return `当前主队：${favoriteTeam.value.team_name}`
  }

  return '选择球队，为它助力'
})

const stanceBadge = computed(() => {
  if (!detail.value) {
    return ''
  }

  return detail.value.viewer.has_supported ? '已助力' : '待助力'
})

const stanceDescription = computed(() => {
  return resolveSupportStanceDescription(detail.value, favoriteTeam.value)
})

const rankingNote = computed(() => {
  if (!favoriteTeam.value) {
    return '关注主队后可查看本赛季助力排名。'
  }

  const rank = favoriteTeam.value.season_support_rank
  return rank
    ? `${favoriteTeam.value.team_name} 当前赛季助力排名第 ${rank}。`
    : `${favoriteTeam.value.team_name} 当前赛季助力排名还在形成中。`
})

function canTakeVoteAction(teamId: number) {
  return voteSubmittingTeamId.value === null && (!session.hasAccessToken.value || !voteState(teamId).disabled)
}

function voteButtonLabel(team: SupportMatchTeam) {
  if (!session.hasAccessToken.value) return '登录后助力'
  const state = voteState(team.team_id)
  if (state.supported && !state.disabled) return '已助力（点击取消）'
  return state.label === '助力' ? `为 ${team.team_name} 助力` : state.label
}

onLoad((options) => {
  setMatchId(options?.matchId)
})

onShow(() => {
  void loadPage()
})

onHide(() => {
  pausePage()
})

onUnload(() => {
  pausePage()
})

onShareAppMessage(() => {
  if (!detail.value) {
    return {
      title: '来这场比赛里为你的主队站队',
      path: buildSharePath('/pages/home/index', 'support'),
    }
  }

  const leadText = supportedTeam.value
    ? `我已为${supportedTeam.value.team_name}助力`
    : '来这场比赛里站队'

  return {
    title: `${leadText}，${detail.value.home_team.team_name} vs ${detail.value.away_team.team_name}`,
    path: buildSharePath(`/pages/support/index?matchId=${detail.value.match_id}`, 'support'),
  }
})

async function handleVote(teamId: number): Promise<void> {
  const outcome = await submitVote(teamId)
  switch (outcome.kind) {
    case 'login-required':
      setPostLoginRedirectTarget({
        type: 'navigateTo',
        url: `/pages/support/index?matchId=${outcome.matchId}`,
      })
      uni.showModal({
        title: '先登录再助力',
        content: '登录后即可选择主队或客队助力，现在去“我的”页登录吗？',
        confirmText: '去登录',
        success: ({ confirm }) => {
          if (confirm) {
            uni.switchTab({ url: '/pages/user/index' })
          }
        },
      })
      return
    case 'blocked':
      uni.showToast({ title: '本场已助力或助力已截止', icon: 'none' })
      return
    case 'cast':
    case 'canceled':
      uni.showToast({ title: outcome.kind === 'canceled' ? '已取消助力' : '助力成功', icon: 'success' })
      return
    case 'failed':
      uni.showToast({ title: outcome.message, icon: 'none' })
      return
  }
}

function generatePoster(): void {
  void poster.generatePoster(detail.value, supportedTeam.value)
}

function savePosterImage(): void {
  savePosterToAlbum(posterImagePath.value)
}

function formatSupportCount(value: number): string {
  if (value >= 10000) {
    return `${(value / 10000).toFixed(1)}w`
  }

  return String(value)
}
</script>

<style scoped lang="css">
.page-root {
  position: relative;
  min-height: 100vh;
  background: #f7f8fa;
}
.page-bg-img {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background: linear-gradient(180deg, rgba(246, 247, 244, 0.24) 0%, rgba(247, 248, 250, 0.74) 32%, #f7f8fa 58%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}
.page-scroll { height: 100vh; padding-top: var(--fi-brand-nav-height); }
.page {
  position: relative;
  z-index: 1;
  padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-40);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
}
.support-scoreboard__meta,
.support-scoreboard__footer,
.support-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-14) var(--fi-space-24);
  border-radius: 999rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
  white-space: nowrap;
}
.hero-summary,
.support-state-card__notes text,
.support-state-card__rule {
  display: block;
  margin-top: var(--fi-space-18);
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.support-scoreboard__meta,
.support-scoreboard__footer {
  color: #8f9198;
  font-size: 24rpx;
}
.support-scoreboard__teams {
  margin-top: var(--fi-space-22);
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: var(--fi-space-16);
  align-items: stretch;
}
.support-team-card {
  padding: var(--fi-space-20);
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: #ffffff;
  display: grid;
  justify-items: center;
  gap: var(--fi-space-8);
}
.support-team-card--away { background: #ffffff; }
.support-team-card--favorite {
  border-color: rgba(255, 140, 43, 0.96);
  box-shadow: 0 18rpx 36rpx rgba(255, 140, 43, 0.14);
}
.support-team-card__avatar {
  width: 88rpx;
  height: 88rpx;
}
.support-team-card__name {
  color: #121212;
  font-size: 28rpx;
  font-weight: 700;
  text-align: center;
}
.support-team-card__count {
  color: #121212;
  font-size: 48rpx;
  font-weight: 800;
  line-height: 1;
}
.support-team-card__share {
  color: #8f9198;
  font-size: 22rpx;
}
.support-scoreboard__vs {
  display: grid;
  align-content: center;
  justify-items: center;
  gap: var(--fi-space-8);
}
.support-scoreboard__vs-mark {
  color: #121212;
  font-size: 28rpx;
  font-weight: 800;
}
.support-scoreboard__vs-state {
  color: #8f9198;
  font-size: 22rpx;
}
.support-bar {
  margin-top: var(--fi-space-20);
  height: 18rpx;
  border-radius: 999rpx;
  overflow: hidden;
  background: #ececf1;
  display: flex;
}
.support-bar__home {
  height: 100%;
  background: #15161b;
}
.support-bar__away {
  height: 100%;
  background: #8f96a3;
}
.support-state-card__notes {
  margin-top: var(--fi-space-18);
  display: grid;
  gap: var(--fi-space-10);
}
.support-actions {
  margin-top: var(--fi-space-22);
  gap: var(--fi-space-12);
  flex-wrap: wrap;
  justify-content: flex-start;
}
.primary-action,
.secondary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-18) var(--fi-space-28);
  border-radius: 999rpx;
  font-size: 26rpx;
  line-height: 1;
  white-space: nowrap;
}
.primary-action {
  background: #15161b;
  color: #ffffff;
}
.primary-action--disabled {
  opacity: 0.48;
}
.support-actions__primary {
  background: var(--fi-color-primary);
}
.secondary-action {
  color: #5f6673;
  background: #f5f6fa;
}
.poster-panel__image {
  margin-top: var(--fi-space-18);
  width: 100%;
  border-radius: 28rpx;
}
.poster-panel__save {
  margin-top: var(--fi-space-18);
}
.state-error {
  color: #c03a2b;
  font-size: 28rpx;
}
.support-poster-canvas {
  position: fixed;
  left: -9999px;
  top: -9999px;
  width: 960px;
  height: 1320px;
}
</style>
