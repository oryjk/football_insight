<template>
  <scroll-view scroll-y class="page-scroll">
    <view class="page">
      <view class="hero-card support-hero">
        <view class="hero-card__top">
          <view>
            <text class="eyebrow">赛前助力</text>
            <text class="hero-card__title">{{ heroTitle }}</text>
          </view>
          <text class="hero-card__badge">{{ heroBadge }}</text>
        </view>

        <text class="hero-card__summary">{{ heroSummary }}</text>
      </view>

      <FiLoading
        v-if="loading"
        title="助力页加载中"
        caption="正在整理这场比赛的双方热度对抗。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else-if="detail">
        <view class="panel support-scoreboard">
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
        </view>

        <view class="panel support-state-card">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">站队状态</text>
              <text class="section-title">{{ stanceTitle }}</text>
            </view>
            <text class="meta-pill">{{ stanceBadge }}</text>
          </view>

          <view class="support-state-card__notes">
            <text>{{ stanceDescription }}</text>
            <text>{{ rankingNote }}</text>
          </view>

          <view class="support-actions">
            <button
              class="primary-action support-actions__primary"
              :class="{ 'primary-action--disabled': !canCastVote }"
              @click="handleVote"
            >
              {{ voteButtonText }}
            </button>
            <button class="secondary-action" open-type="share">
              转发拉票
            </button>
            <button class="secondary-action" @click="generatePoster">
              生成拉票海报
            </button>
          </view>

          <text class="support-state-card__rule">
            规则：每个用户每场比赛只能助力 1 次，只能为自己关注的主队助力，比赛开始后自动关闭。
          </text>
        </view>

        <view v-if="posterImagePath" class="panel poster-panel">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">拉票海报</text>
              <text class="section-title">已生成本地海报</text>
            </view>
          </view>

          <image :src="posterImagePath" mode="widthFix" class="poster-panel__image" />
          <button class="primary-action poster-panel__save" @click="savePosterImage">保存到相册</button>
        </view>
      </template>

      <canvas canvas-id="supportPosterCanvas" class="support-poster-canvas" />
    </view>
  </scroll-view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onHide, onLoad, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { castSupportVote, getMatchSupportDetail } from '../../api/support'
import type { SupportMatchDetail } from '../../types/support'
import { extractApiErrorMessage } from '../../utils/apiError'
import { getAccessToken, setPostLoginRedirectTarget } from '../../utils/authStorage'
import {
  resolveSupportCountdownLabel,
  resolveSupportCountdownSeconds,
  resolveSupportHeroBadge,
  resolveSupportHeroSummary,
  resolveSupportStanceDescription,
  resolveSupportVoteButtonText,
  resolveSupportWindowLabel,
} from './helpers'

const loading = ref(true)
const errorMessage = ref('')
const detail = ref<SupportMatchDetail | null>(null)
const matchId = ref<number | null>(null)
const posterImagePath = ref('')
const liveCountdownSeconds = ref(0)
let countdownTimer: ReturnType<typeof setInterval> | null = null

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

  if (detail.value.viewer.has_supported && favoriteTeam.value) {
    return `你已为 ${favoriteTeam.value.team_name} 助力`
  }

  if (favoriteTeam.value) {
    return `当前主队：${favoriteTeam.value.team_name}`
  }

  return '还没有主队身份'
})

const stanceBadge = computed(() => {
  if (!detail.value) {
    return ''
  }

  if (detail.value.viewer.has_supported) {
    return '已助力'
  }

  if (favoriteTeam.value) {
    return '待助力'
  }

  return '未关注主队'
})

const stanceDescription = computed(() => {
  return resolveSupportStanceDescription(detail.value, favoriteTeam.value)
})

const rankingNote = computed(() => {
  if (!favoriteTeam.value) {
    return '关注主队后，这里会展示它在本赛季的助力排名。'
  }

  const rank = favoriteTeam.value.season_support_rank
  return rank
    ? `${favoriteTeam.value.team_name} 当前赛季助力排名第 ${rank}。`
    : `${favoriteTeam.value.team_name} 当前赛季助力排名还在形成中。`
})

const canCastVote = computed(() => {
  if (!detail.value) {
    return false
  }

  if (!getAccessToken()) {
    return true
  }

  return detail.value.viewer.can_support && !!favoriteTeam.value
})

const voteButtonText = computed(() => {
  return resolveSupportVoteButtonText(detail.value, favoriteTeam.value, !!getAccessToken())
})

onLoad((options) => {
  const rawMatchId = typeof options?.matchId === 'string' ? Number(options.matchId) : Number.NaN
  matchId.value = Number.isFinite(rawMatchId) && rawMatchId > 0 ? rawMatchId : null
})

onShow(() => {
  void loadPage()
})

onHide(() => {
  stopCountdown()
})

onUnload(() => {
  stopCountdown()
})

onShareAppMessage(() => {
  if (!detail.value) {
    return {
      title: '来这场比赛里为你的主队站队',
      path: '/pages/home/index',
    }
  }

  const leadText = favoriteTeam.value
    ? `我已为${favoriteTeam.value.team_name}助力`
    : '来这场比赛里站队'

  return {
    title: `${leadText}，${detail.value.home_team.team_name} vs ${detail.value.away_team.team_name}`,
    path: `/pages/support/index?matchId=${detail.value.match_id}`,
  }
})

async function loadPage(): Promise<void> {
  if (!matchId.value) {
    errorMessage.value = '缺少比赛参数，无法打开助力页。'
    loading.value = false
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    detail.value = await getMatchSupportDetail(matchId.value)
    syncCountdownState()
    startCountdown()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '助力页加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

async function handleVote(): Promise<void> {
  if (!detail.value || !matchId.value) {
    return
  }

  if (!getAccessToken()) {
    setPostLoginRedirectTarget({
      type: 'navigateTo',
      url: `/pages/support/index?matchId=${matchId.value}`,
    })
    uni.showModal({
      title: '先登录再助力',
      content: '登录后才能为你的主队助力，现在去“我的”页登录吗？',
      confirmText: '去登录',
      success: ({ confirm }) => {
        if (!confirm) {
          return
        }

        uni.switchTab({ url: '/pages/user/index' })
      },
    })
    return
  }

  if (!favoriteTeam.value) {
    uni.showModal({
      title: '先关注主队',
      content: '你还没有设置主队，先去首页关注一支主队，再回来助力。',
      confirmText: '去首页',
      success: ({ confirm }) => {
        if (!confirm) {
          return
        }

        uni.switchTab({ url: '/pages/home/index' })
      },
    })
    return
  }

  if (!detail.value.viewer.can_support) {
    uni.showToast({ title: voteButtonText.value, icon: 'none' })
    return
  }

  try {
    detail.value = await castSupportVote(matchId.value, {
      supported_team_id: favoriteTeam.value.team_id,
    })
    syncCountdownState()
    uni.showToast({ title: '助力成功', icon: 'success' })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '助力失败'), icon: 'none' })
  }
}

function syncCountdownState(): void {
  if (!detail.value) {
    liveCountdownSeconds.value = 0
    return
  }

  liveCountdownSeconds.value = resolveSupportCountdownSeconds(detail.value.kickoff_at)

  if (liveCountdownSeconds.value <= 0) {
    detail.value.support_window_status = 'closed'
    detail.value.viewer.can_support = false
  }
}

function stopCountdown(): void {
  if (!countdownTimer) {
    return
  }

  clearInterval(countdownTimer)
  countdownTimer = null
}

function startCountdown(): void {
  stopCountdown()

  if (!detail.value || detail.value.support_window_status === 'closed') {
    return
  }

  countdownTimer = setInterval(() => {
    if (!detail.value) {
      stopCountdown()
      return
    }

    liveCountdownSeconds.value = Math.max(0, liveCountdownSeconds.value - 1)

    if (liveCountdownSeconds.value <= 0) {
      detail.value.support_window_status = 'closed'
      detail.value.viewer.can_support = false
      stopCountdown()
    }
  }, 1000)
}

function formatSupportCount(value: number): string {
  if (value >= 10000) {
    return `${(value / 10000).toFixed(1)}w`
  }

  return String(value)
}

async function generatePoster(): Promise<void> {
  if (!detail.value) {
    return
  }

  try {
    const canvasId = 'supportPosterCanvas'
    const width = 960
    const height = 1320
    const context = uni.createCanvasContext(canvasId)

    context.setFillStyle('#f5f0e8')
    context.fillRect(0, 0, width, height)

    context.setFillStyle('#ffffff')
    roundRect(context, 44, 44, width - 88, height - 88, 40)
    context.fill()

    context.setFillStyle('#121212')
    context.setFontSize(34)
    context.fillText('足球洞察 · 赛前助力', 96, 132)

    context.setFontSize(52)
    context.fillText(detail.value.home_team.team_name, 96, 244)
    context.fillText('VS', width / 2 - 26, 326)
    context.fillText(detail.value.away_team.team_name, 96, 408)

    context.setFillStyle('#6b707b')
    context.setFontSize(28)
    context.fillText(`${detail.value.match_date} ${detail.value.match_time} · 第 ${detail.value.round_number} 轮`, 96, 486)

    context.setFillStyle('#f97316')
    context.fillRect(96, 560, Math.max(40, (width - 192) * (detail.value.home_team.support_share_pct / 100)), 28)
    context.setFillStyle('#2563eb')
    context.fillRect(96, 616, Math.max(40, (width - 192) * (detail.value.away_team.support_share_pct / 100)), 28)

    context.setFillStyle('#121212')
    context.setFontSize(32)
    context.fillText(`${detail.value.home_team.team_name} ${detail.value.home_team.support_count}`, 96, 724)
    context.fillText(`${detail.value.away_team.team_name} ${detail.value.away_team.support_count}`, 96, 786)

    context.setFillStyle('#444b57')
    context.setFontSize(40)
    context.fillText(buildPosterHeadline(), 96, 926)

    context.setFillStyle('#6b707b')
    context.setFontSize(28)
    context.fillText('打开小程序，进入这场比赛继续站队助力。', 96, 1004)
    context.fillText('每人每场只能助力一次，只能为自己的主队投票。', 96, 1050)

    await new Promise<void>((resolve) => {
      context.draw(false, () => resolve())
    })

    posterImagePath.value = await new Promise<string>((resolve, reject) => {
      uni.canvasToTempFilePath({
        canvasId,
        width,
        height,
        destWidth: width,
        destHeight: height,
        success: (result) => resolve(result.tempFilePath),
        fail: reject,
      })
    })

    uni.showToast({ title: '海报已生成', icon: 'success' })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '海报生成失败'), icon: 'none' })
  }
}

function buildPosterHeadline(): string {
  if (!detail.value) {
    return '来为你的主队站队'
  }

  if (favoriteTeam.value) {
    const rivalTeam = favoriteTeam.value.team_id === detail.value.home_team.team_id
      ? detail.value.away_team
      : detail.value.home_team
    return `我已为${favoriteTeam.value.team_name}助力，来把${rivalTeam.team_name}压过去`
  }

  return `${detail.value.home_team.team_name} 和 ${detail.value.away_team.team_name} 的热度对抗已经打响`
}

function savePosterImage(): void {
  if (!posterImagePath.value) {
    return
  }

  uni.saveImageToPhotosAlbum({
    filePath: posterImagePath.value,
    success: () => {
      uni.showToast({ title: '已保存到相册', icon: 'success' })
    },
    fail: (error) => {
      uni.showToast({ title: extractApiErrorMessage(error, '保存失败'), icon: 'none' })
    },
  })
}

function roundRect(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  context.beginPath()
  context.moveTo(x + radius, y)
  context.arcTo(x + width, y, x + width, y + height, radius)
  context.arcTo(x + width, y + height, x, y + height, radius)
  context.arcTo(x, y + height, x, y, radius)
  context.arcTo(x, y, x + width, y, radius)
  context.closePath()
}
</script>

<style scoped lang="css">
.page-scroll { height: 100vh; }
.page {
  padding: 28rpx 24rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 24rpx;
}
.hero-card,
.panel,
.state-card {
  background: rgba(255,255,255,0.94);
  border-radius: 36rpx;
  padding: 28rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 28rpx 60rpx rgba(26,28,36,0.08);
}
.support-hero {
  background:
    radial-gradient(circle at top right, rgba(255, 144, 39, 0.16), transparent 34%),
    linear-gradient(180deg, rgba(255,255,255,0.98), rgba(250,246,239,0.96));
}
.hero-card__top,
.section-heading,
.support-scoreboard__meta,
.support-scoreboard__footer,
.support-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.hero-card__top,
.section-heading {
  align-items: flex-start;
  gap: 16rpx;
}
.eyebrow,
.section-kicker {
  color: #131313;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 2rpx;
}
.hero-card__title,
.section-title {
  display: block;
  margin-top: 10rpx;
  color: #121212;
  font-size: 56rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title { font-size: 40rpx; }
.hero-card__badge,
.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 14rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 223, 205, 0.92);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
  white-space: nowrap;
}
.hero-card__summary,
.support-state-card__notes text,
.support-state-card__rule {
  display: block;
  margin-top: 18rpx;
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
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  gap: 16rpx;
  align-items: stretch;
}
.support-team-card {
  padding: 20rpx;
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(251,251,253,0.96), rgba(255,255,255,1));
  display: grid;
  justify-items: center;
  gap: 8rpx;
}
.support-team-card--away {
  background: linear-gradient(180deg, rgba(247,250,255,0.96), rgba(255,255,255,1));
}
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
  gap: 8rpx;
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
  margin-top: 20rpx;
  height: 18rpx;
  border-radius: 999rpx;
  overflow: hidden;
  background: #ececf1;
  display: flex;
}
.support-bar__home {
  height: 100%;
  background: linear-gradient(90deg, #ff8b2b, #ffb347);
}
.support-bar__away {
  height: 100%;
  background: linear-gradient(90deg, #4d8dff, #7bb4ff);
}
.support-state-card__notes {
  margin-top: 18rpx;
  display: grid;
  gap: 10rpx;
}
.support-actions {
  margin-top: 22rpx;
  gap: 12rpx;
  flex-wrap: wrap;
  justify-content: flex-start;
}
.primary-action,
.secondary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 18rpx 28rpx;
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
  background: linear-gradient(135deg, #121318, #ff7a18);
}
.secondary-action {
  color: #5f6673;
  background: #f5f6fa;
}
.poster-panel__image {
  margin-top: 18rpx;
  width: 100%;
  border-radius: 28rpx;
}
.poster-panel__save {
  margin-top: 18rpx;
}
.state-card--error text {
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
