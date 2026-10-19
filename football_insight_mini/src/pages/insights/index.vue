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
        <FiPanel>
          <view class="insights-workspace">
            <InsightsTeamSelector
              v-if="selectableTeams.length"
              :teams="selectableTeams"
              :selected-team-id="selectedTeamId"
              @update:selected-team-id="teamBrowser.selectTeam"
            />
            <view v-else class="state-card team-loading">
              <text>{{ teamError || '正在加载球队…' }}</text>
              <button v-if="teamError" @tap="teamBrowser.load()">重试</button>
            </view>
            <view>
              <FiSegmented :items="mainTabs" :model-value="mainTab" aria-label="洞察内容" @update:model-value="changeMainTab" />
            </view>
            <view class="insights-content">
              <InsightsTacticsPanel
                v-if="mainTab === 'tactics'"
                :authenticated="tacticsAuthenticated"
                :mine-loading="mineLoading" :mine-error="mineError" :draft="mineDraft" :published="minePublished"
                :items="communityItems" :total="communityTotal" :sort="communitySort"
                :loading="communityLoading || teamLoading" :error="teamError || communityError" :pending-likes="communityPendingLikes"
                @edit="goToTeamBoard" @retry-mine="tactics.retryMine()" @retry-list="retryTacticsList"
                @sort="tactics.changeSort" @like="tactics.like" @dislike="tactics.dislike" @open="openPublicTactics" @more="tactics.more()"
              />
              <template v-else>
                <view v-if="outlookReviewReady && !systemConfigUnderReview && currentUser" class="workspace-section">
                  <InsightsMatchOutlook
                    :matches="outlookMatches"
                    :selected-match-id="outlookSelectedMatchId"
                    :outlook="matchOutlook"
                    :loading="outlookLoading"
                    :error="outlookError"
                    @select-match="outlookLoader.selectMatch"
                  />
                </view>

                <view class="workspace-section">
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
                    />

                    <view class="contribution-section">
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
                    </view>
                  </template>

                  <view v-else class="state-card">
                    <text>当前还没有可展示的球队洞察数据。</text>
                  </view>
                </view>
              </template>
            </view>
          </view>
        </FiPanel>
      </view>
    </view>

    <FiLoginFloat v-if="showLoginFloat" :desc="systemConfigUnderReview ? '登录后，可以关注主队并参与赛前助力。' : undefined" @action="goToLogin" />
  </view>
</template>

<script setup lang="ts">
import { buildSharePath } from '../../utils/shareSource'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, ref, watch } from 'vue'
import { onHide, onReachBottom, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import FiSegmented from '../../components/FiSegmented.vue'
import FiPanel from '../../components/FiPanel.vue'
import InsightsTacticsPanel from './components/InsightsTacticsPanel.vue'
import { useInsightsTeams } from './useInsightsTeams'
import { useInsightsTactics } from './useInsightsTactics'
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
import { buildLineupPageUrl, buildLineupViewUrl } from '../../utils/lineupLinks'
import { createInsightsSessionSync, runInsightsShow } from './sessionEffects'
import { useInsightsData } from './useInsightsData'

const { navigationPageStyle } = useNavigationLayout()

// 登录用户共享自 appSession：读取走会话缓存，401 清理由 session 统一处理。
const session = useAppSession()
const currentUser = session.currentUser
const mainTab = ref<'data' | 'tactics'>('data')
const mainTabs = [{ key: 'data', label: '数据洞察' }, { key: 'tactics', label: '战术' }]
function changeMainTab(value: string): void {
  if (value === 'data' || value === 'tactics') mainTab.value = value
}
const pageVisible = ref(false)
const teamBrowser = useInsightsTeams(session)
const { teams: selectableTeams, selectedTeamId, loading: teamLoading, error: teamError } = teamBrowser
const tactics = useInsightsTactics({ session, selectedTeamId, active: computed(() => pageVisible.value && mainTab.value === 'tactics') })
const { mineLoading, mineError, mineDraft, minePublished, authenticated: tacticsAuthenticated } = tactics
const { items: communityItems, total: communityTotal, sort: communitySort, loading: communityLoading, error: communityError, pendingLikes: communityPendingLikes } = tactics.community
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
  insights,
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
  selectedTeamId,
})
const outlookLoader = createOutlookLoader({
  authenticated: () => Boolean(currentUser.value),
  identity: () => currentUser.value,
  underReview: () => !outlookReviewReady.value || systemConfigUnderReview.value,
  revision: session.revision,
  selectedTeamId,
  externalTeamSelection: true,
})
const {
  matches: outlookMatches,
  outlook: matchOutlook,
  selectedMatchId: outlookSelectedMatchId,
  loading: outlookLoading,
  error: outlookError,
} = outlookLoader
onUnload(() => { outlookLoader.dispose(); tactics.dispose(); teamBrowser.dispose() })
onHide(() => { pageVisible.value = false })
onReachBottom(() => { if (mainTab.value === 'tactics') void tactics.more() })

const showLoginFloat = computed(() => mainTab.value === 'data' && !userLoading.value && !currentUser.value)
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
  title: mainTab.value === 'tactics' ? `${selectableTeams.value.find(team => team.team_id === selectedTeamId.value)?.team_name ?? '球队'}战术，看看大家怎么排` : selectedInsight.value
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
  return pageVisible.value
    && outlookReviewReady.value
    && Boolean(currentUser.value)
    && !userLoading.value
    && !systemConfigUnderReview.value
    && !membershipBenefitsLocked.value
    && !isMembershipTierAtLeast(currentUser.value?.membership_tier, 'V6')
}

function reportTicketWatchEntryImpression(): void {
  if (shouldMeasureTicketWatchEntry()) reportPaywallImpression('insights_ticket_watch_entry')
}

watch([currentUser, userLoading, pageVisible, outlookReviewReady], reportTicketWatchEntryImpression)

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
  if (selectedTeamId.value === null) return
  uni.navigateTo({ url: buildLineupPageUrl(selectedTeamId.value) })
}
function openPublicTactics(code: string): void {
  uni.navigateTo({ url: buildLineupViewUrl(code) })
}
function retryTacticsList(): void {
  if (teamError.value) void teamBrowser.load()
  else void tactics.refresh()
}

onShow(async () => {
  pageVisible.value = true
  void teamBrowser.load()
  void tactics.refresh()
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
.team-loading { color:var(--fi-color-text-caption); font-size:var(--fi-font-24); }
.page-root { position: relative; min-height: 100vh; background: var(--fi-color-page-soft); }
.insights-workspace { display: flex; flex-direction: column; gap: var(--fi-space-24); min-width: 0; }
.insights-content {
  /* FiPanel 的组件 token 可继承穿过小程序宿主节点，内层作为同一底板的内容区。 */
  --fi-component-panel-bg: transparent;
  --fi-component-panel-radius: 0;
  --fi-component-panel-border: 0;
  --fi-component-panel-shadow: none;
  --fi-component-panel-padding: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-28);
}
.workspace-section { min-width: 0; padding-top: var(--fi-space-24); border-top: var(--fi-border-width) solid var(--fi-color-divider); }
.contribution-section { margin-top: var(--fi-space-28); padding-top: var(--fi-space-24); border-top: var(--fi-border-width) solid var(--fi-color-divider); }
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
  background: var(--fi-color-page-backdrop-fade);
  pointer-events: none;
  z-index: 0;
}
.state-card {
  position: relative;
  padding: var(--fi-space-16) 0;
}
.state-card--error text {
  font-size: var(--fi-font-28);
  color: var(--fi-color-error);
}
</style>
