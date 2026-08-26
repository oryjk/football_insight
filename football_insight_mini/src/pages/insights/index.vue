<template>
  <view class="page-root">
    <FiBrandNav open-on-current-page @open-ai="openAiFromBrandNav" />
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" :webp="true" />
    <view class="page-bg-fade"></view>
    <view class="page-scroll">
      <view class="page">
        <InsightsHeroCard />

        <InsightsTicketWatchEntry
          v-if="!systemConfigUnderReview"
          :locked="shouldLockTicketWatchEntry"
          @open="openTicketWatch"
        />

        <FiLoading
          v-if="!systemConfigUnderReview && userLoading"
          title="账号状态确认中"
          caption="正在确认你是否已经登录。"
        />

        <view v-else-if="systemConfigUnderReview" class="state-card">
          <text>当前版本展示基础内容。</text>
        </view>

        <InsightsLockedPanel v-else-if="!currentUser" mode="guest" />

        <InsightsLockedPanel v-else-if="membershipBenefitsLocked" mode="membership" @action="goToUserPage" />

        <FiLoading
          v-else-if="loading"
          title="洞察生成中"
          caption="正在整理球队归因结果。"
        />

        <view v-else-if="errorMessage" class="state-card state-card--error">
          <text>{{ errorMessage }}</text>
        </view>

        <template v-else-if="selectedInsight">
          <InsightsTeamSelector
            v-model:selected-team-id="selectedTeamId"
            :teams="teams"
          />

          <InsightsSummaryPanel :insight="selectedInsight" />

          <InsightsTeamBoardEntry @open="goToTeamBoard" />

          <InsightsContributionPanel
            kicker="进球贡献"
            title="对手维度"
            :rows="goalsForByOpponentRows"
            :expanded="goalsForOpponentExpanded"
            @toggle="goalsForOpponentExpanded = !goalsForOpponentExpanded"
          />

          <InsightsContributionPanel
            kicker="进球贡献"
            title="球员维度"
            :rows="goalsForByPlayerRows"
            :expanded="goalsForPlayerExpanded"
            @toggle="goalsForPlayerExpanded = !goalsForPlayerExpanded"
          />

          <InsightsContributionPanel
            kicker="助攻贡献"
            title="球员维度"
            :rows="assistsByPlayerRows"
            :expanded="assistsForPlayerExpanded"
            @toggle="assistsForPlayerExpanded = !assistsForPlayerExpanded"
          />

          <InsightsContributionPanel
            kicker="失球贡献"
            title="对手维度"
            :rows="goalsAgainstByOpponentRows"
            :expanded="goalsAgainstOpponentExpanded"
            @toggle="goalsAgainstOpponentExpanded = !goalsAgainstOpponentExpanded"
          />
        </template>

        <view v-else class="state-card">
          <text>当前还没有可展示的球队洞察数据。</text>
        </view>
    </view>
    </view>

    <FiAiChatSheet
      :visible="aiChatVisible"
      :current-user="currentAiUser"
      :ai-chat-mode="aiPublicConfig?.ai_chat_mode"
      @close="closeAiChat"
    />

    <FiLoginFloat v-if="showLoginFloat" @action="goToLogin" />
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiBrandNav from '../../components/FiBrandNav.vue'
import FiAiChatSheet from '../../components/FiAiChatSheet.vue'
import FiLoginFloat from '../../components/FiLoginFloat.vue'
import FiLoading from '../../components/FiLoading.vue'
import InsightsHeroCard from './components/InsightsHeroCard.vue'
import InsightsTicketWatchEntry from './components/InsightsTicketWatchEntry.vue'
import InsightsLockedPanel from './components/InsightsLockedPanel.vue'
import InsightsTeamSelector from './components/InsightsTeamSelector.vue'
import InsightsSummaryPanel from './components/InsightsSummaryPanel.vue'
import InsightsTeamBoardEntry from './components/InsightsTeamBoardEntry.vue'
import InsightsContributionPanel from './components/InsightsContributionPanel.vue'
import { getCurrentUser } from '../../api/auth'
import { getLiveTeamInsights } from '../../api/insight'
import type { TeamInsight, TeamInsightTeam } from '../../types/insight'
import type { CurrentUser } from '../../types/auth'
import { extractApiErrorMessage } from '../../utils/apiError'
import { PHOENIX_STADIUM_BG_IMAGE_URL as bgImage } from '../../config/assets'
import { resolveMembershipBenefitsLocked } from '../../utils/membershipBenefits'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'
import { useAiChatSheet } from '../../composables/useAiChatSheet'
import {
  buildAssistContributionRows,
  buildOpponentContributionRows,
  buildPlayerContributionRows,
} from './helpers'

const userLoading = ref(true)
const loading = ref(false)
const errorMessage = ref('')
const currentUser = ref<CurrentUser | null>(null)
const teams = ref<TeamInsightTeam[]>([])
const insights = ref<TeamInsight[]>([])
const selectedTeamId = ref<number | null>(null)
const goalsForOpponentExpanded = ref(false)
const goalsForPlayerExpanded = ref(false)
const assistsForPlayerExpanded = ref(false)
const goalsAgainstOpponentExpanded = ref(false)
const ticketWatchNavigating = ref(false)
const systemConfigUnderReview = ref(false)
const {
  aiChatVisible,
  currentAiUser,
  aiPublicConfig,
  openAiChat,
  closeAiChat,
} = useAiChatSheet()
const membershipBenefitsLocked = computed(() =>
  resolveMembershipBenefitsLocked(currentUser.value),
)
const showLoginFloat = computed(() => !systemConfigUnderReview.value && !userLoading.value && !currentUser.value)
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
  path: '/pages/insights/index',
}))

watch(selectedTeamId, () => {
  resetExpandedState()
})

function resetExpandedState(): void {
  goalsForOpponentExpanded.value = false
  goalsForPlayerExpanded.value = false
  assistsForPlayerExpanded.value = false
  goalsAgainstOpponentExpanded.value = false
}

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

function openAiFromBrandNav(): void {
  void openAiChat()
}

async function loadUser(): Promise<void> {
  userLoading.value = true

  try {
    currentUser.value = await getCurrentUser()
  } catch {
    currentUser.value = null
  } finally {
    userLoading.value = false
  }
}

async function loadInsights(): Promise<void> {
  if (!currentUser.value || membershipBenefitsLocked.value) {
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    const response = await getLiveTeamInsights()
    teams.value = response.teams
    insights.value = response.insights
    selectedTeamId.value = response.teams[0]?.team_id ?? null
    resetExpandedState()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '洞察加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
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
  reportPageActivity('insights')
  ticketWatchNavigating.value = false
  systemConfigUnderReview.value = await loadSystemConfigUnderReview()
  if (systemConfigUnderReview.value) {
    userLoading.value = false
    loading.value = false
    currentUser.value = null
    teams.value = []
    insights.value = []
    selectedTeamId.value = null
    return
  }
  await loadUser()
  await loadInsights()
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
  padding: var(--fi-space-24) var(--fi-space-16) calc(152rpx + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
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
  background: linear-gradient(180deg, transparent 45%, rgba(247, 248, 250, 0.55) 78%, var(--fi-color-page-soft) 100%);
  pointer-events: none;
  z-index: 0;
}
.state-card {
  position: relative;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}
.state-card--error text {
  font-size: var(--fi-font-28);
  color: #c03a2b;
}
</style>
