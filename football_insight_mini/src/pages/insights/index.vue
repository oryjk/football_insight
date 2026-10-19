<template>
  <page-meta :page-style="navigationPageStyle" />
  <view class="page-root">
    <FiBrandNav />
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
        <InsightsTicketWatchEntry
          v-if="!systemConfigUnderReview"
          :locked="shouldLockTicketWatchEntry"
          @open="openTicketWatch"
        />

        <InsightsTeamSelector
          v-if="outlookReviewReady && currentUser && selectableTeams.length"
          :teams="selectableTeams"
          :selected-team-id="selectedTeamId"
          @update:selected-team-id="outlookLoader.selectTeam"
        />

        <InsightsMatchOutlook
          v-if="outlookReviewReady && !systemConfigUnderReview && currentUser"
          :matches="outlookMatches"
          :selected-match-id="outlookSelectedMatchId"
          :outlook="matchOutlook"
          :loading="outlookLoading"
          :error="outlookError"
          @select-match="outlookLoader.selectMatch"
        />

        <InsightsSkeleton
          v-if="userLoading"
        />

        <InsightsLockedPanel v-else-if="!currentUser" mode="guest" />

        <InsightsLockedPanel v-else-if="membershipBenefitsLocked" mode="membership" @action="goToUserPage" />

        <InsightsSkeleton
          v-else-if="loading"
        />

        <view v-else-if="errorMessage" class="state-card state-card--error">
          <text>{{ errorMessage }}</text>
        </view>

        <template v-else-if="selectedInsight">
          <InsightsTeamPanel
            :insight="selectedInsight"
            @open-board="goToTeamBoard"
          />

          <InsightsContributionSurface
            :rows="{
              'goals-for-opponent': goalsForByOpponentRows,
              'goals-for-player': goalsForByPlayerRows,
              'assists-player': assistsByPlayerRows,
              'goals-against-opponent': goalsAgainstByOpponentRows,
            }"
            :expanded="{
              'goals-for-opponent': goalsForOpponentExpanded,
              'goals-for-player': goalsForPlayerExpanded,
              'assists-player': assistsForPlayerExpanded,
              'goals-against-opponent': goalsAgainstOpponentExpanded,
            }"
            @toggle="key => {
              if (key === 'goals-for-opponent') goalsForOpponentExpanded = !goalsForOpponentExpanded
              else if (key === 'goals-for-player') goalsForPlayerExpanded = !goalsForPlayerExpanded
              else if (key === 'assists-player') assistsForPlayerExpanded = !assistsForPlayerExpanded
              else if (key === 'goals-against-opponent') goalsAgainstOpponentExpanded = !goalsAgainstOpponentExpanded
            }"
          />
        </template>

        <view v-else class="state-card">
          <text>当前还没有可展示的球队洞察数据。</text>
        </view>
    </view>
    </view>

    <FiLoginFloat v-if="showLoginFloat" :desc="systemConfigUnderReview ? '登录后，可以关注主队并参与赛前助力。' : undefined" @action="goToLogin" />
  </view>
</template>

<script setup lang="ts">
import { buildSharePath } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, ref, watch } from 'vue'
import { onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import InsightsSkeleton from './components/InsightsSkeleton.vue'
import InsightsTicketWatchEntry from './components/InsightsTicketWatchEntry.vue'
import InsightsLockedPanel from './components/InsightsLockedPanel.vue'
import InsightsMatchOutlook from './components/InsightsMatchOutlook.vue'
import { createOutlookLoader } from './useMatchOutlook'
import InsightsTeamPanel from './components/InsightsTeamPanel.vue'
import InsightsTeamSelector from './components/InsightsTeamSelector.vue'
import InsightsContributionSurface from './components/InsightsContributionSurface.vue'
import type { TeamInsight } from '../../types/insight'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import { useAppSession } from '../../stores/appSession'
import { resolveMembershipBenefitsLocked } from '../../utils/membershipBenefits'
import { isMembershipTierAtLeast } from '../../utils/membershipRules'
import { beginPaywallVisit, reportPaywallImpression, reportPaywallTap } from '../../composables/usePaywallTelemetry'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import {
  buildAssistContributionRows,
  buildOpponentContributionRows,
  buildPlayerContributionRows,
} from './helpers'
import { createInsightsSessionSync, runInsightsShow } from './sessionEffects'
import { useInsightsData } from './useInsightsData'

const { navigationPageStyle } = useNavigationLayout()

// 登录用户共享自 appSession：读取走会话缓存，401 清理由 session 统一处理。
const session = useAppSession()
const currentUser = session.currentUser
const ticketWatchNavigating = ref(false)
const systemConfigUnderReview = ref(false)
const outlookReviewReady = ref(false)
const membershipBenefitsLocked = computed(() =>
  resolveMembershipBenefitsLocked(currentUser.value),
)
// 会员洞察（账号私有数据）的加载编排：onShow 与 currentUser watcher 同会话合并请求，
// 旧账号结果不发布、旧 finally 不清新 loading；会话身份变化同步清旧缓存。
const {
  userLoading,
  loading,
  errorMessage,
  teams,
  insights,
  selectedTeamId,
  goalsForOpponentExpanded,
  goalsForPlayerExpanded,
  assistsForPlayerExpanded,
  goalsAgainstOpponentExpanded,
  loadUser,
  loadInsights,
  resetExpandedState,
  resetSessionScopedState: resetInsightsSessionState,
} = useInsightsData({
  session,
  isMembershipLocked: () => membershipBenefitsLocked.value,
})
const outlookLoader = createOutlookLoader({
  authenticated: () => Boolean(currentUser.value),
  identity: () => currentUser.value,
  underReview: () => !outlookReviewReady.value || systemConfigUnderReview.value,
  revision: session.revision,
  selectedTeamId,
})
const {
  teams: outlookTeams,
  matches: outlookMatches,
  outlook: matchOutlook,
  selectedMatchId: outlookSelectedMatchId,
  loading: outlookLoading,
  error: outlookError,
} = outlookLoader
const selectableTeams = computed(() => outlookTeams.value.length ? outlookTeams.value : teams.value)
onUnload(outlookLoader.dispose)

const showLoginFloat = computed(() => !userLoading.value && !currentUser.value)
const shouldLockTicketWatchEntry = computed(() =>
  !userLoading.value && (!currentUser.value || membershipBenefitsLocked.value),
)

const selectedInsight = computed<TeamInsight | null>(() =>
  insights.value.find((item) => item.team_id === selectedTeamId.value) ?? null,
)
const goalsForByOpponentRows = computed(() =>
  buildOpponentContributionRows(
    'goals-for-opponent',
    selectedTeamId.value,
    selectedInsight.value?.goals_for_by_opponent ?? [],
    'ink',
  ),
)
const goalsForByPlayerRows = computed(() =>
  buildPlayerContributionRows(
    'goals-for-player',
    selectedTeamId.value,
    selectedInsight.value?.goals_for_by_player ?? [],
    'red',
  ),
)
const assistsByPlayerRows = computed(() =>
  buildAssistContributionRows(
    'assists-for-player',
    selectedTeamId.value,
    selectedInsight.value?.assists_for_by_player ?? [],
  ),
)
const goalsAgainstByOpponentRows = computed(() =>
  buildOpponentContributionRows(
    'goals-against-opponent',
    selectedTeamId.value,
    selectedInsight.value?.goals_against_by_opponent ?? [],
    'danger',
  ),
)

onShareAppMessage(() => ({
  title: selectedInsight.value
    ? `${selectedInsight.value.team_name} 的贡献结构，看看这队靠谁赢球`
    : '球队洞察：不是榜单，而是贡献结构',
  path: buildSharePath('/pages/insights/index', 'insights'),
}))

watch(selectedTeamId, () => {
  resetExpandedState()
})

// 后台静默恢复/登录发布已认证用户后，直接刷新会员洞察，无需再次切 Tab。
// 会话身份变化（revision）先同步清掉旧账号私有缓存，跨账号不沿用。
// 单一组合 watch 保证同一次发布里 reset 先于 load，且多次 revision 递增只触发一轮。
const insightsSessionSync = createInsightsSessionSync({
  loadInsights: () => loadInsights(),
  resetSessionScopedState: resetInsightsSessionState,
})
watch([session.revision, currentUser], ([revision, user], [previousRevision, previousUser]) => {
  if (revision !== previousRevision) {
    insightsSessionSync.handleSessionRevisionChange()
  }
  // 只有真正发布了新用户（引用变化）才刷新私有数据：beginAuthIntent 只递增 revision、
  // currentUser 仍是旧账号，此时不能发旧 token 的私有请求。
  if (user !== previousUser) {
    insightsSessionSync.handleSessionUserPublished(user)
  }
})

// 付费引导度量：只统计登录、权益未暂停且尚未达到 V6 的用户，这类用户才会在回流看板里遇到升级引导。
function shouldMeasureTicketWatchEntry(): boolean {
  return Boolean(currentUser.value)
    && !userLoading.value
    && !systemConfigUnderReview.value
    && !membershipBenefitsLocked.value
    && !isMembershipTierAtLeast(currentUser.value?.membership_tier, 'V6')
}

function reportTicketWatchEntryImpression(): void {
  if (shouldMeasureTicketWatchEntry()) reportPaywallImpression('insights_ticket_watch_entry')
}

watch([currentUser, userLoading], reportTicketWatchEntryImpression)

function openTicketWatch(): void {
  if (!currentUser.value) {
    return
  }

  if (membershipBenefitsLocked.value) {
    uni.showToast({
      title: '当前账号已取关公众号，会员权益已暂停',
      icon: 'none',
      duration: 2200,
    })
    return
  }

  if (ticketWatchNavigating.value) {
    return
  }

  if (shouldMeasureTicketWatchEntry()) reportPaywallTap('insights_ticket_watch_entry')

  ticketWatchNavigating.value = true
  uni.navigateTo({
    url: '/pages/ticket-watch/index',
    animationType: 'none',
    animationDuration: 0,
    fail: () => {
      ticketWatchNavigating.value = false
    },
  })
}

function goToUserPage(): void {
  uni.switchTab({ url: '/pages/user/index' })
}

function goToLogin(): void {
  rememberPostLoginRedirect({
    type: 'switchTab',
    url: '/pages/insights/index',
  })
  goToUserPage()
}

function goToTeamBoard(): void {
  if (!selectedInsight.value) {
    return
  }

  uni.navigateTo({
    url: `/pages/team-board/index?teamId=${selectedInsight.value.team_id}`,
  })
}

onShow(async () => {
  outlookReviewReady.value = false
  reportPageActivity('insights')
  beginPaywallVisit()
  reportTicketWatchEntryImpression()
  ticketWatchNavigating.value = false
  // 审核状态只决定是否隐藏回流看板入口与赛前倾向卡片，会员洞察照常加载。
  await runInsightsShow({
    revision: session.revision,
    loadUnderReview: loadSystemConfigUnderReview,
    publishUnderReview: (underReview) => {
      systemConfigUnderReview.value = underReview
      outlookReviewReady.value = true
    },
    loadUser,
    loadInsights,
  })
})
</script>

<style scoped lang="css">
.page-root { position: relative; }
.page-scroll {
  padding-top: var(--fi-brand-nav-height);
  position: relative;
  z-index: 1;
}

.page {
  padding: var(--fi-space-24) var(--fi-space-16) calc(var(--fi-space-insights-bottom) + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
}
.page-bg-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-insights-backdrop-height);
  pointer-events: none;
  z-index: 0;
}
.page-bg-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: var(--fi-space-insights-backdrop-height);
  background: var(--fi-color-insights-page-wash);
  pointer-events: none;
  z-index: 0;
}
.state-card {
  position: relative;
  background: var(--fi-color-surface-panel);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}
.state-card--error text {
  font-size: var(--fi-font-28);
  color: var(--fi-color-error);
}
</style>
