<template>
  <view class="home-page-shell">
    <FiBrandNav />
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
        <HomeDailyTopics
          :edition="topicEdition"
          :loading="topicsLoading"
          :error-message="topicsErrorMessage"
          :available-audio-ids="topicAudio.availableIds.value"
          :active-audio-id="topicAudio.activeId.value"
          :audio-status="topicAudio.status.value"
          @read="openDailyTopic"
          @play="topicAudio.toggle"
          @retry="loadDailyTopics"
          @more="openMoreTopics"
        />

        <FiTopicAudioPlayer
          v-if="topicAudio.activeId.value"
          :title="topicAudio.activeTitle.value"
          :status="topicAudio.status.value"
          :current-time="topicAudio.currentTime.value"
          :duration="topicAudio.duration.value"
          :error-message="topicAudio.errorMessage.value"
          @toggle="toggleCurrentTopicAudio"
          @seek="topicAudio.seek"
          @close="topicAudio.stop"
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
import { computed, ref, watch } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import HomeDailyTopics from './components/HomeDailyTopics.vue'
import FiTopicAudioPlayer from '../../components/topics/FiTopicAudioPlayer.vue'
import HomeSupportPanel from './components/HomeSupportPanel.vue'
import HomeSkeleton from './components/HomeSkeleton.vue'
import HomeStoryPanel from './components/HomeStoryPanel.vue'
import HomeRankingPanel from './components/HomeRankingPanel.vue'
import HomeTechStatsSheet from './components/HomeTechStatsSheet.vue'
import HomeTeamSeasonSheet from './components/HomeTeamSeasonSheet.vue'
import HomeFavoriteTeamSheet from './components/HomeFavoriteTeamSheet.vue'
import { getAvailableRounds, getMatches, getOverview } from '../../api/insight'
import type {
  InsightOverviewResponse,
  MatchCard,
  OverviewPlayer,
  OverviewStanding,
  RoundReference,
} from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import { useRuntimeConfig } from '../../stores/runtimeConfig'
import { buildHeadlineTitleParts } from '../../utils/homeViewText'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildHomeHeadlineBody,
  buildHomeWatchPoints,
  resolveHomeLoadPlan,
  resolveHomePulseLeadMatch,
  resolveHomePulseMatches,
  resolveHomePulseTechStats,
  resolveHomeTeamSeasonMatches,
  type HomePulseLeadMatch,
  type HomeTeamSeasonMatch,
} from './helpers'
import { createHomeSessionSync } from './sessionEffects'
import { useHomeSupportData } from './useHomeSupportData'
import { useHomeDailyTopics } from './useHomeDailyTopics'
import { useDailyTopicAudio } from '../../composables/useDailyTopicAudio'
import { topicDetailUrl } from './dailyTopics'

const currentSeason = new Date().getFullYear()
let homeLoadRevision = 0
const loading = ref(true)
const errorMessage = ref('')
const overview = ref<InsightOverviewResponse | null>(null)
const liveMatches = ref<MatchCard[]>([])
const rounds = ref<RoundReference[]>([])
// 公共配置共享自 runtimeConfig store（失败保留上次成功值）。
const runtimeConfig = useRuntimeConfig()
const selectedPulseMatch = ref<HomePulseLeadMatch | null>(null)
const selectedStandingsTeam = ref<OverviewStanding | null>(null)
const allSeasonMatches = ref<MatchCard[] | null>(null)
const teamSeasonMatchesLoading = ref(false)
const teamSeasonMatchesErrorMessage = ref('')
// 登录标记共享自 appSession：token 只由 session 写/清，页面不再读存储或自行清 token。
const session = useAppSession()
const hasAuthToken = session.hasAccessToken
// 助力面板（账号私有数据）的加载编排：私有读取先等共享 ensure 确认用户，
// onShow/currentUser watcher 同会话合并请求，旧账号结果不发布、旧 finally 不清新 loading。
const {
  supportLoading,
  supportErrorMessage,
  supportProfile,
  supportTeams,
  loginPromptVisible,
  favoriteTeamSheetVisible,
  selectedFavoriteTeamId,
  submitFavoriteTeam,
  loadSupportData,
  loadSupportTeams,
  resetSessionScopedState: resetHomeSupportSessionState,
} = useHomeSupportData({ session })
// 后台静默恢复/登录/退出发布 currentUser 后，助力面板（私有）随会话刷新，无需重新进入页面；
// 会话身份变化（revision）先同步清掉旧账号私有缓存，跨账号不沿用。
// 单一组合 watch 保证同一次发布里 reset 先于 load，且多次 revision 递增只触发一轮。
const homeSessionSync = createHomeSessionSync({
  loadSupportData: () => loadSupportData(),
  resetSessionScopedState: resetHomeSupportSessionState,
})
watch([session.revision, session.currentUser], ([revision, user], [previousRevision, previousUser]) => {
  if (revision !== previousRevision) {
    homeSessionSync.handleSessionRevisionChange()
  }
  // 只有真正发布了新用户（引用变化）才刷新私有数据：beginAuthIntent 只递增 revision、
  // currentUser 仍是旧账号，此时不能发旧 token 的私有请求。
  if (user !== previousUser) {
    homeSessionSync.handleSessionUserPublished(user)
  }
})

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
const supportFavoriteTeam = computed(() => supportProfile.value?.favorite_team ?? null)
const {
  edition: topicEdition,
  loading: topicsLoading,
  errorMessage: topicsErrorMessage,
  load: loadDailyTopics,
} = useHomeDailyTopics()
const topicAudio = useDailyTopicAudio(topicEdition)

function openDailyTopic(id: string) { uni.navigateTo({ url: topicDetailUrl(id) }) }
function openMoreTopics() { uni.navigateTo({ url: '/pages/topics/index' }) }

function toggleCurrentTopicAudio() {
  if (topicAudio.activeId.value) topicAudio.toggle(topicAudio.activeId.value)
}

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
    : '足球洞察：今日话题，关注成都蓉城',
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
  const revision = ++homeLoadRevision
  loading.value = true
  errorMessage.value = ''

  try {
    const loadPlan = resolveHomeLoadPlan(hasAuthToken.value)
    if (!loadPlan.critical.includes('overview')) {
      throw new Error('首页首屏加载计划缺少 overview')
    }

    const response = await getOverview({ mode: 'live', season: currentSeason, roundNumber: null })
    if (revision !== homeLoadRevision) return
    liveMatches.value = []
    overview.value = response
    void loadDeferredHomeData(revision)
  } catch (error) {
    if (revision === homeLoadRevision) errorMessage.value = extractApiErrorMessage(error, '首页数据加载失败，请稍后重试。')
  } finally {
    if (revision === homeLoadRevision) loading.value = false
  }
}

async function loadDeferredHomeData(revision: number) {
  try {
    const [liveMatchesResponse, roundsResponse] = await Promise.all([
      getMatches({ mode: 'live', season: currentSeason, roundNumber: null }),
      getAvailableRounds(currentSeason),
      // 公共配置走共享 store（60 秒缓存/失败保留上次成功值），结果由 store 发布。
      runtimeConfig.ensureRuntimeConfig(),
    ])

    if (revision !== homeLoadRevision) return
    liveMatches.value = liveMatchesResponse.matches
    rounds.value = roundsResponse
  } catch (error) {
    if (revision === homeLoadRevision) console.warn('[home] deferred data load failed', error)
  }
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
  const teamId = selectedFavoriteTeamId.value
  if (!teamId) {
    uni.showToast({ title: '请先选择一支主队', icon: 'none' })
    return
  }

  try {
    // 写操作身份从 submit 调用瞬间（PUT 发出前）建立；返回 false = 等待期间会话已切换，
    // 旧写成功不得关闭新账号的 sheet/toast，也不得 force 刷新替换新账号在途的读取。
    const completed = await submitFavoriteTeam(teamId)
    if (!completed) {
      return
    }
    favoriteTeamSheetVisible.value = false
    uni.showToast({ title: '主队已更新', icon: 'success' })
    // 写后强制刷新：不与写前在途的 profile GET 合并——旧快照迟到后不能覆盖刚写入的选择。
    await loadSupportData({ force: true })
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

onShow(() => {
  reportPageActivity('home')
  void loadPage()
  void loadDailyTopics()
  void loadSupportData()
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
  padding: var(--fi-space-24) var(--fi-space-16) var(--fi-space-40);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
}

.state-card {
  background: var(--fi-color-surface-frosted);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.state-card--error text {
  font-size: var(--fi-font-28);
  color: var(--fi-color-error);
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
