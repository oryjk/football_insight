<template>
  <view class="home-page-shell">
    <FiBrandNav open-on-current-page @open-ai="openAiFromBrandNav" />
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
        <HomeHeroBriefing
          :hero-guide="heroGuide"
          :hero-guide-note="heroGuideNote"
          :is-briefing-ready="isBriefingReady"
          :briefing-items="briefingItems"
          :briefing-marquee-rows="briefingMarqueeRows"
        />

        <HomeSupportPanel
          :has-auth-token="hasAuthToken"
          :loading="supportLoading"
          :profile="supportProfile"
          :error-message="supportErrorMessage"
          :teams="supportTeams"
          @select-team="openFavoriteTeamSheet"
          @open-match="openSupportMatch"
        />

        <HomeSkeleton v-if="loading" />

        <view v-else-if="errorMessage" class="state-card state-card--error">
          <text>{{ errorMessage }}</text>
        </view>

        <template v-else-if="overview">
          <HomeStoryPanel
            :headline-parts="headlineTitleParts"
            :body="headlineBody"
            :updated-at-label="updatedAtLabel"
            :pulse-matches="pulseMatches"
            :watch-points="watchPoints"
            @open-tech-stats="openPulseMatchTechStats"
          />

          <HomeRankingPanel
            kicker="积分榜头部"
            title="先看联赛头部格局"
            :rows="standingsRows"
            @select="openStandingsRowSheet"
          />

          <HomeRankingPanel
            kicker="射手榜"
            title="当前赛季累计射手榜"
            :rows="scorersRows"
          />
        </template>
      </view>
    </view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="publicConfig?.ai_chat_mode"
      @close="handleCloseAiChat"
    />

    <HomeTechStatsSheet
      v-if="selectedPulseMatch"
      :match="selectedPulseMatch"
      @close="closePulseMatchTechStats"
    />

    <HomeTeamSeasonSheet
      v-if="selectedStandingsTeam"
      :team="selectedStandingsTeam"
      :matches="selectedStandingsTeamMatches"
      :loading="teamSeasonMatchesLoading"
      :error-message="teamSeasonMatchesErrorMessage"
      @close="closeStandingsTeamSheet"
    />

    <HomeFavoriteTeamSheet
      v-if="favoriteTeamSheetVisible"
      :teams="supportTeams"
      :selected-team-id="selectedFavoriteTeamId"
      @change="selectedFavoriteTeamId = $event"
      @confirm="handleConfirmFavoriteTeam"
      @close="closeFavoriteTeamSheet"
    />

    <FiLoginFloat v-if="loginPromptVisible" @action="handleSupportLogin" />
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import HomeHeroBriefing from './components/HomeHeroBriefing.vue'
import HomeSupportPanel from './components/HomeSupportPanel.vue'
import HomeSkeleton from './components/HomeSkeleton.vue'
import HomeStoryPanel from './components/HomeStoryPanel.vue'
import HomeRankingPanel from './components/HomeRankingPanel.vue'
import HomeTechStatsSheet from './components/HomeTechStatsSheet.vue'
import HomeTeamSeasonSheet from './components/HomeTeamSeasonSheet.vue'
import HomeFavoriteTeamSheet from './components/HomeFavoriteTeamSheet.vue'
import { getCurrentUser } from '../../api/auth'
import { getAvailableRounds, getMatches, getOverview, getRankings } from '../../api/insight'
import { getSupportProfile, listSupportTeams, setFavoriteTeam } from '../../api/support'
import { getPublicSystemConfig } from '../../api/system'
import type { CurrentUser } from '../../types/auth'
import type {
  InsightOverviewResponse,
  MatchCard,
  OverviewPlayer,
  OverviewStanding,
  PlayerRankingCategory,
  RankingsViewResponse,
  RoundReference,
} from '../../types/insight'
import type { SupportProfile, SupportTeam } from '../../types/support'
import type { PublicSystemConfig } from '../../types/system'
import { extractApiErrorMessage } from '../../utils/apiError'
import { getAccessToken, setAccessToken } from '../../utils/authStorage'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import { buildHomeBriefingMarqueeMap, splitBriefingMarqueeRows } from '../../utils/homeBriefingMarquees'
import { buildHeadlineTitleParts } from '../../utils/homeViewText'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { reportPageActivity } from '../../utils/userActivity'
import { consumeOpenAiChatIntent } from '../../utils/aiEntryIntent'
import {
  buildHomeBriefingItems,
  buildHomeHeadlineBody,
  buildHomeHeroGuide,
  buildHomeWatchPoints,
  isHomeAuthExpiredMessage,
  resolveHomeGuideLeaders,
  resolveHomeGuideNote,
  resolveHomeGuideReferenceRoundNumber,
  resolveHomeHasAuthToken,
  resolveHomeLoadPlan,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  type HomePulseLeadMatch,
  type HomeTeamSeasonMatch,
} from './helpers'

const currentSeason = new Date().getFullYear()
const loading = ref(true)
const errorMessage = ref('')
const overview = ref<InsightOverviewResponse | null>(null)
const liveMatches = ref<MatchCard[]>([])
const rounds = ref<RoundReference[]>([])
const rankings = ref<{ player_categories: PlayerRankingCategory[] } | null>(null)
const guideRankings = ref<RankingsViewResponse | null>(null)
const publicConfig = ref<PublicSystemConfig | null>(null)
const currentAiUser = ref<CurrentUser | null>(null)
const aiChatVisible = ref(false)
const selectedPulseMatch = ref<HomePulseLeadMatch | null>(null)
const selectedStandingsTeam = ref<OverviewStanding | null>(null)
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
const supportLoading = ref(true)
const supportErrorMessage = ref('')
const supportProfile = ref<SupportProfile | null>(null)
const supportTeams = ref<SupportTeam[]>([])
const supportTeamsLoading = ref(false)
const favoriteTeamSheetVisible = ref(false)
const selectedFavoriteTeamId = ref<number | null>(null)
const hasAuthToken = ref(resolveHomeHasAuthToken(getAccessToken()))
const loginPromptVisible = ref(false)

const standings = computed<OverviewStanding[]>(() => overview.value?.standings_top ?? [])
const scorers = computed<OverviewPlayer[]>(() => overview.value?.top_scorers ?? [])
const recentMatches = computed(() => (overview.value?.recent_matches ?? []).slice(0, 4))
const insightSummary = computed(() => overview.value?.insight_summary ?? null)
const pulseMatches = computed(() =>
  resolveHomePulseMatches(liveMatches.value, recentMatches.value),
)
const leadMatch = computed(() =>
  pulseMatches.value[0] ?? resolveHomePulseLeadMatch(liveMatches.value, recentMatches.value),
)
const selectedStandingsTeamMatches = computed<HomeTeamSeasonMatch[]>(() => {
  if (!selectedStandingsTeam.value || !allSeasonMatches.value) {
    return []
  }

  return resolveHomeTeamSeasonMatches(selectedStandingsTeam.value, allSeasonMatches.value)
})
const topTeam = computed<OverviewStanding | null>(() => standings.value[0] ?? null)
const topScorer = computed<OverviewPlayer | null>(() => scorers.value[0] ?? null)
const guideLeaders = computed(() => resolveHomeGuideLeaders({
  rounds: rounds.value,
  liveStandings: standings.value,
  liveScorers: scorers.value,
  referenceRankings: guideRankings.value,
}))
const assistCategory = computed<PlayerRankingCategory | null>(() =>
  rankings.value?.player_categories.find((item) => item.slug === 'assists') ?? null,
)
const topAssist = computed(() => assistCategory.value?.entries[0] ?? null)
const isBriefingReady = computed(() => !loading.value && !errorMessage.value && !!overview.value)
const supportFavoriteTeam = computed(() => supportProfile.value?.favorite_team ?? null)

const leadingTeams = computed(() => {
  if (!topTeam.value) {
    return []
  }

  return standings.value.filter((team) => team.points === topTeam.value?.points)
})

const leadingScorers = computed(() => {
  if (!topScorer.value) {
    return []
  }

  return scorers.value.filter((player) => player.score_value === topScorer.value?.score_value)
})

const leadingAssists = computed(() => {
  if (!topAssist.value || !assistCategory.value) {
    return []
  }

  return assistCategory.value.entries.filter((player) => player.score_value === topAssist.value?.score_value)
})

const updatedAtLabel = computed(() => {
  if (!overview.value?.latest_scrape_finished_at) {
    return '等待同步'
  }

  const date = new Date(overview.value.latest_scrape_finished_at)
  return `${date.getMonth() + 1}/${date.getDate()} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
})

const headlineTitleParts = computed(() =>
  buildHeadlineTitleParts({
    headline: insightSummary.value?.headline ?? null,
    leadMatch: leadMatch.value
      ? {
          homeTeamName: leadMatch.value.home_team_name,
          awayTeamName: leadMatch.value.away_team_name,
          homeScore: leadMatch.value.home_score,
          awayScore: leadMatch.value.away_score,
        }
      : null,
    topTeamName: topTeam.value?.team_name ?? null,
  }),
)

const headlineBody = computed(() =>
  buildHomeHeadlineBody({
    summary: insightSummary.value?.summary ?? null,
    topScorer: topScorer.value,
  }),
)

const heroGuide = computed(() =>
  buildHomeHeroGuide({ guideLeaders: guideLeaders.value, leadMatch: leadMatch.value }),
)

const heroGuideNote = computed(() => resolveHomeGuideNote(guideLeaders.value.source))

const briefingItems = computed(() =>
  buildHomeBriefingItems({
    topTeam: topTeam.value,
    leadingTeams: leadingTeams.value,
    topScorer: topScorer.value,
    leadingScorers: leadingScorers.value,
    topAssist: topAssist.value,
    leadingAssists: leadingAssists.value,
  }),
)

const briefingMarqueeRows = computed(() => {
  const map = buildHomeBriefingMarqueeMap(publicConfig.value?.home_briefing_marquees)
  return {
    leader: splitBriefingMarqueeRows(map.leader ?? []),
    scorer: splitBriefingMarqueeRows(map.scorer ?? []),
    assist: splitBriefingMarqueeRows(map.assist ?? []),
  }
})

const watchPoints = computed(() =>
  buildHomeWatchPoints({
    bullets: insightSummary.value?.bullets ?? [],
    leadMatch: leadMatch.value,
    topTeam: topTeam.value,
    topScorer: topScorer.value,
    secondScorer: scorers.value[1] ?? null,
  }),
)

const standingsRows = computed(() =>
  standings.value.map(team => ({
    key: team.team_id,
    rankNo: team.rank_no,
    avatar: team.avatar_storage_url,
    name: team.team_name,
    note: '点击查看赛季战绩',
    metricValue: team.points,
    metricNote: '积分',
    interactive: true,
  })),
)

const scorersRows = computed(() =>
  scorers.value.map(player => ({
    key: player.player_id,
    rankNo: player.rank_no,
    avatar: player.avatar_storage_url,
    name: player.player_name,
    note: player.team_name,
    metricValue: player.score_value,
    metricNote: '累计进球',
    avatarMode: 'fill' as const,
  })),
)

onShareAppMessage(() => ({
  title: supportFavoriteTeam.value
    ? `${supportFavoriteTeam.value.team_name} 下一场先为谁站队？`
    : '足球洞察：这一轮之后，谁在改变联赛格局',
  path: '/pages/home/index',
}))

function openPulseMatchTechStats(match: HomePulseLeadMatch) {
  if (!resolveHomePulseTechStats(match).length) {
    uni.showToast({ title: '这场比赛暂时还没有技术统计', icon: 'none' })
    return
  }

  selectedPulseMatch.value = match
}

function closePulseMatchTechStats() {
  selectedPulseMatch.value = null
}

async function ensureAllSeasonMatchesLoaded() {
  if (allSeasonMatches.value || teamSeasonMatchesLoading.value) {
    return
  }

  teamSeasonMatchesLoading.value = true
  teamSeasonMatchesErrorMessage.value = ''

  try {
    const availableRounds = rounds.value.length ? rounds.value : await getAvailableRounds(currentSeason)
    if (!rounds.value.length) {
      rounds.value = availableRounds
    }

    const responses = await Promise.all(
      availableRounds.map((round) =>
        getMatches({ mode: 'round', season: currentSeason, roundNumber: round.round_number }),
      ),
    )

    const matchMap = new Map<number, MatchCard>()
    for (const response of responses) {
      for (const match of response.matches) {
        matchMap.set(match.match_id, match)
      }
    }

    allSeasonMatches.value = Array.from(matchMap.values())
  } catch (error) {
    teamSeasonMatchesErrorMessage.value = extractApiErrorMessage(error, '球队赛季战绩加载失败，请稍后重试。')
  } finally {
    teamSeasonMatchesLoading.value = false
  }
}

async function openStandingsRowSheet(row: { key: string | number }) {
  const team = standings.value.find(item => item.team_id === row.key)
  if (!team) {
    return
  }

  selectedStandingsTeam.value = team
  await ensureAllSeasonMatchesLoaded()
}

function closeStandingsTeamSheet() {
  selectedStandingsTeam.value = null
  teamSeasonMatchesErrorMessage.value = ''
}

async function loadPage() {
  loading.value = true
  errorMessage.value = ''

  try {
    const loadPlan = resolveHomeLoadPlan(hasAuthToken.value)
    if (!loadPlan.critical.includes('overview')) {
      throw new Error('首页首屏加载计划缺少 overview')
    }

    overview.value = await getOverview({ mode: 'live', season: currentSeason, roundNumber: null })
    void loadDeferredHomeData()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '首页数据加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

async function loadDeferredHomeData() {
  try {
    const [rankingsResponse, liveMatchesResponse, roundsResponse, publicSystemConfig] = await Promise.all([
      getRankings({ mode: 'live', season: currentSeason, roundNumber: null }),
      getMatches({ mode: 'live', season: currentSeason, roundNumber: null }),
      getAvailableRounds(currentSeason),
      getPublicSystemConfig(),
    ])

    rankings.value = rankingsResponse
    liveMatches.value = liveMatchesResponse.matches
    rounds.value = roundsResponse
    publicConfig.value = publicSystemConfig

    const guideReferenceRoundNumber = resolveHomeGuideReferenceRoundNumber(roundsResponse)
    guideRankings.value = guideReferenceRoundNumber === null
      ? null
      : await getRankings({ mode: 'round', season: currentSeason, roundNumber: guideReferenceRoundNumber })
  } catch (error) {
    console.warn('[home] deferred data load failed', error)
  }
}

async function loadSupportTeams() {
  if (supportTeams.value.length || supportTeamsLoading.value) {
    return
  }

  supportTeamsLoading.value = true
  try {
    supportTeams.value = await listSupportTeams()
  } catch (error) {
    console.warn('[home] support teams load failed', error)
  } finally {
    supportTeamsLoading.value = false
  }
}

async function loadSupportData() {
  supportErrorMessage.value = ''
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())
  loginPromptVisible.value = false

  if (!hasAuthToken.value) {
    supportProfile.value = null
    supportTeams.value = []
    loginPromptVisible.value = true
    supportLoading.value = false
    return
  }

  supportLoading.value = true

  try {
    supportProfile.value = await getSupportProfile()
    selectedFavoriteTeamId.value = supportProfile.value.favorite_team?.team_id ?? selectedFavoriteTeamId.value
    void loadSupportTeams()
  } catch (error) {
    const message = extractApiErrorMessage(error, '助力入口加载失败，请稍后重试。')

    if (isHomeAuthExpiredMessage(message)) {
      setAccessToken(null)
      hasAuthToken.value = false
      supportProfile.value = null
      supportErrorMessage.value = ''
      loginPromptVisible.value = true
      return
    }

    supportErrorMessage.value = message
  } finally {
    supportLoading.value = false
  }
}

async function loadCurrentAiUser() {
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())

  if (!hasAuthToken.value) {
    currentAiUser.value = null
    loginPromptVisible.value = true
    return
  }

  try {
    currentAiUser.value = await getCurrentUser()
  } catch {
    currentAiUser.value = null
  }

  if (!currentAiUser.value && hasAuthToken.value) {
    setAccessToken(null)
    hasAuthToken.value = false
    loginPromptVisible.value = true
  }
}

async function ensureAiUser(): Promise<CurrentUser | null> {
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())

  if (!hasAuthToken.value) {
    return null
  }

  if (currentAiUser.value) {
    return currentAiUser.value
  }

  await loadCurrentAiUser()
  return currentAiUser.value
}

function promptAiLogin() {
  uni.showModal({
    title: '先登录再聊天',
    content: '登录后才可以和小罗继续对话，现在去“我的”页登录吗？',
    confirmText: '去登录',
    success: ({ confirm }) => {
      if (!confirm) {
        return
      }

      uni.switchTab({
        url: '/pages/user/index',
      })
    },
  })
}

async function openAiChatDirectly(): Promise<void> {
  const user = await ensureAiUser()
  if (!user) {
    promptAiLogin()
    return
  }

  aiChatVisible.value = true
}

function openAiFromBrandNav(): void {
  void openAiChatDirectly()
}

function handleSupportLogin() {
  rememberPostLoginRedirect({
    type: 'switchTab',
    url: '/pages/home/index',
  })
  uni.switchTab({
    url: '/pages/user/index',
  })
}

async function openFavoriteTeamSheet() {
  await loadSupportTeams()
  selectedFavoriteTeamId.value = supportFavoriteTeam.value?.team_id ?? supportTeams.value[0]?.team_id ?? null
  favoriteTeamSheetVisible.value = true
}

function closeFavoriteTeamSheet() {
  favoriteTeamSheetVisible.value = false
}

async function handleConfirmFavoriteTeam() {
  if (!selectedFavoriteTeamId.value) {
    uni.showToast({ title: '请先选择一支主队', icon: 'none' })
    return
  }

  try {
    await setFavoriteTeam({ team_id: selectedFavoriteTeamId.value })
    favoriteTeamSheetVisible.value = false
    uni.showToast({ title: '主队已更新', icon: 'success' })
    await loadSupportData()
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '主队设置失败'), icon: 'none' })
  }
}

function openSupportMatch() {
  const nextMatch = supportProfile.value?.next_match
  if (!nextMatch) {
    return
  }

  uni.navigateTo({
    url: `/pages/support/index?matchId=${nextMatch.match_id}`,
  })
}

function handleCloseAiChat() {
  aiChatVisible.value = false
}

onShow(() => {
  reportPageActivity('home')
  hasAuthToken.value = resolveHomeHasAuthToken(getAccessToken())
  void loadPage()
  void loadSupportData()

  if (consumeOpenAiChatIntent()) {
    void openAiChatDirectly()
  }
})
</script>

<style scoped lang="css">
.home-page-shell {
  min-height: 100vh;
  position: relative;
}

.page-scroll {
  padding-top: var(--fi-brand-nav-height);
  position: relative;
  z-index: 1;
  box-sizing: border-box;
}

.page {
  padding: 24rpx 16rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}

.state-card {
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

.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 600rpx;
  background: linear-gradient(180deg, transparent 45%, rgba(247,248,250,0.55) 78%, var(--fi-color-page-soft) 100%);
  pointer-events: none;
  z-index: 0;
}
</style>
