<template>
  <view class="page-root">
    <image class="page-bg-img" :src="bgImage" mode="aspectFill" />
    <view class="page-bg-fade"></view>
    <scroll-view scroll-y class="page-scroll">
      <view class="page">
        <view class="hero-card">
        <view class="hero-card__top">
          <view>
            <text class="eyebrow">Insights</text>
            <text class="hero-card__title">洞察不是榜单，而是贡献结构</text>
          </view>
          <text class="meta-note meta-note--hero">球队洞察</text>
        </view>

        <text class="hero-card__summary">
          先按球队看赛季累计贡献：这个队的进球主要打在谁身上，谁在本队内部贡献了更多进球和助攻，以及失球主要来自哪些对手。
        </text>
      </view>

      <view v-if="!systemConfigUnderReview" class="panel insights-ticket-watch-entry" @click="openTicketWatch">
        <view class="section-heading section-heading--compact">
          <view>
            <text class="section-kicker">Ticket Watch</text>
            <text class="section-title">回流看板</text>
          </view>
          <text class="meta-note">实时 + 历史</text>
        </view>

        <text class="insights-ticket-watch-entry__copy">
          当前比赛按秒刷新分区回流，历史比赛按场次回看库存热区。这个入口放在洞察页，更符合“比赛供给变化也是一种观察信号”的产品语义。
        </text>
      </view>

      <FiLoading
        v-if="!systemConfigUnderReview && userLoading"
        title="账号状态确认中"
        caption="正在确认你是否已经登录。"
      />

      <view v-else-if="systemConfigUnderReview" class="state-card">
        <text>当前版本展示基础内容。</text>
      </view>

      <view v-else-if="!currentUser" class="panel insights-locked">
        <view class="insights-locked__preview">
          <view class="insights-locked__chips">
            <text v-for="name in previewTeams" :key="name">{{ name }}</text>
          </view>

          <view class="insights-locked__summary">
            <view class="insights-locked__metric">
              <text>总进球</text>
              <text class="insights-locked__metric-value">14</text>
              <text>赛季累计打进</text>
            </view>
            <view class="insights-locked__metric">
              <text>总失球</text>
              <text class="insights-locked__metric-value">5</text>
              <text>赛季累计丢失</text>
            </view>
          </view>

          <view class="insights-locked__rows">
            <view class="insights-locked__section">
              <text class="insights-locked__section-title">进球贡献 · 对手维度</text>
              <view v-for="row in previewGoalAgainstRows" :key="row.name" class="insights-locked__row">
                <text>{{ row.name }}</text>
                <view><view :style="{ width: row.width }" /></view>
              </view>
            </view>

            <view class="insights-locked__section">
              <text class="insights-locked__section-title">进球贡献 · 球员维度</text>
              <view v-for="row in previewGoalPlayerRows" :key="row.name" class="insights-locked__row">
                <text>{{ row.name }}</text>
                <view><view :style="{ width: row.width }" /></view>
              </view>
            </view>

            <view class="insights-locked__section">
              <text class="insights-locked__section-title">助攻贡献 · 球员维度</text>
              <view v-for="row in previewAssistRows" :key="row.name" class="insights-locked__row">
                <text>{{ row.name }}</text>
                <view><view :style="{ width: row.width }" /></view>
              </view>
            </view>
          </view>
        </view>

        <view class="insights-lock-overlay">
          <view class="insights-lock-overlay__card">
            <text class="section-kicker">登录后查看</text>
            <text class="insights-lock-overlay__title">球队洞察需要先登录</text>
            <text class="insights-lock-overlay__copy">
              登录后可查看球队进球贡献、助攻贡献和失球贡献结构。如果你还没有完成首次微信登录，可以先关注公众号获取邀请码，再补齐微信绑定。
            </text>
            <button class="insights-lock-overlay__action" @click="goToLogin">去登录查看</button>
          </view>
        </view>
      </view>

      <view v-else-if="membershipBenefitsLocked" class="panel insights-locked">
        <view class="insights-locked__preview">
          <view class="insights-locked__chips">
            <text v-for="name in previewTeams" :key="`locked-${name}`">{{ name }}</text>
          </view>

          <view class="insights-locked__summary">
            <view class="insights-locked__metric">
              <text>洞察权限</text>
              <text class="insights-locked__metric-value">暂停</text>
              <text>取关后自动冻结</text>
            </view>
            <view class="insights-locked__metric">
              <text>恢复方式</text>
              <text class="insights-locked__metric-value">重新关注</text>
              <text>返回这里刷新即可</text>
            </view>
          </view>
        </view>

        <view class="insights-lock-overlay">
          <view class="insights-lock-overlay__card">
            <text class="section-kicker">会员权益已暂停</text>
            <text class="insights-lock-overlay__title">当前账号已取关公众号</text>
            <text class="insights-lock-overlay__copy">
              洞察页和回流看板属于会员权益。重新关注公众号后，回到小程序刷新即可恢复；会员等级和你已推荐的人不会受影响。
            </text>
            <button class="insights-lock-overlay__action" @click="goToUserPage">去我的页查看</button>
          </view>
        </view>
      </view>

      <FiLoading
        v-else-if="loading"
        title="洞察生成中"
        caption="正在整理球队归因结果。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else-if="selectedInsight">
        <view class="panel insights-team-selector">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">球队列表</text>
              <text class="section-title">先选一个球队</text>
            </view>
            <text class="meta-note">{{ teams.length }} 支球队</text>
          </view>

          <scroll-view
            scroll-x
            class="insights-team-selector__scroll"
            :scroll-left="teamSelectorScrollLeft"
            scroll-with-animation
          >
            <view class="insights-team-selector__list">
              <button
                v-for="team in teams"
                :id="`team-chip-${team.team_id}`"
                :key="team.team_id"
                class="insights-team-chip"
                :class="{ active: selectedTeamId === team.team_id }"
                @click="selectedTeamId = team.team_id"
              >
                <image
                  v-if="team.avatar_storage_url"
                  :src="team.avatar_storage_url"
                  mode="aspectFit"
                  class="insights-team-chip__avatar"
                />
                <view v-else class="insights-team-chip__avatar insights-team-chip__avatar--fallback">
                  {{ team.team_name.charAt(0) }}
                </view>
                <text>#{{ team.rank_no }} {{ team.team_name }}</text>
              </button>
            </view>
          </scroll-view>
        </view>

        <view class="panel insights-summary">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">当前球队</text>
              <text class="section-title">{{ selectedInsight.team_name }}</text>
            </view>
            <text class="meta-note">第 {{ selectedInsight.rank_no }} 名</text>
          </view>

          <view class="insights-summary__grid">
            <view class="briefing-card briefing-card--metric">
              <text class="briefing-card__label">总进球</text>
              <view class="briefing-card__body briefing-card__body--summary">
                <view class="briefing-card__main">
                  <text
                    :key="`goals-for-${selectedTeamId ?? 'none'}-${animatedGoalsForTotal}`"
                    class="insights-summary__value insights-summary__value--animated"
                  >
                    {{ animatedGoalsForTotal }}
                  </text>
                  <text class="briefing-card__subvalue">赛季累计打进</text>
                </view>
              </view>
            </view>

            <view class="briefing-card briefing-card--metric">
              <text class="briefing-card__label">总失球</text>
              <view class="briefing-card__body briefing-card__body--summary">
                <view class="briefing-card__main">
                  <text
                    :key="`goals-against-${selectedTeamId ?? 'none'}-${animatedGoalsAgainstTotal}`"
                    class="insights-summary__value insights-summary__value--animated"
                  >
                    {{ animatedGoalsAgainstTotal }}
                  </text>
                  <text class="briefing-card__subvalue">赛季累计丢失</text>
                </view>
              </view>
            </view>
          </view>
        </view>

        <view class="panel insights-board-entry">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">球队战术板</text>
              <text class="section-title">把这张洞察卡变成观点</text>
            </view>
            <button class="meta-pill meta-pill--button" @click="goToTeamBoard">进入战术板</button>
          </view>

          <text class="insights-board-entry__copy">
            不发普通灌水帖，只能围绕当前球队的进球贡献、助攻贡献和失球贡献发布观点。先用数据站住，再去讨论判断。
          </text>
        </view>

        <view class="panel insights-breakdown">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">进球贡献</text>
              <text class="section-title">对手维度</text>
            </view>
          </view>

          <view class="contribution-list">
            <view
              v-for="item in visibleGoalsForByOpponent"
              :key="contributionInstanceKey('goals-for-opponent', item)"
              class="contribution-row"
            >
              <view class="contribution-row__header">
                <view class="contribution-row__identity">
                  <image :src="item.opponent_avatar_storage_url || ''" mode="aspectFit" class="contribution-row__avatar" />
                  <view>
                    <text class="contribution-row__name">{{ item.opponent_team_name }}</text>
                    <text class="contribution-row__note">{{ item.goals }} 球</text>
                  </view>
                </view>
                <text class="contribution-row__share">{{ formatShare(item.share) }}</text>
              </view>
              <view class="contribution-row__bar">
                <view class="contribution-row__fill" :style="{ width: barWidth(item.share) }" />
              </view>
            </view>
          </view>

          <button
            v-if="shouldShowToggle(selectedInsight.goals_for_by_opponent)"
            class="insights-breakdown__toggle"
            @click="goalsForOpponentExpanded = !goalsForOpponentExpanded"
          >
            {{ goalsForOpponentExpanded ? '收起' : '更多' }}
          </button>
        </view>

        <view class="panel insights-breakdown">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">进球贡献</text>
              <text class="section-title">球员维度</text>
            </view>
          </view>

          <view class="contribution-list">
            <view
              v-for="item in visibleGoalsForByPlayer"
              :key="contributionInstanceKey('goals-for-player', item)"
              class="contribution-row"
            >
              <view class="contribution-row__header">
                <view class="contribution-row__identity">
                  <image :src="item.avatar_storage_url || ''" mode="aspectFill" class="contribution-row__avatar contribution-row__avatar--player" />
                  <view>
                    <text class="contribution-row__name">{{ item.player_name }}</text>
                    <text class="contribution-row__note">{{ item.goals }} 球</text>
                  </view>
                </view>
                <text class="contribution-row__share">{{ formatShare(item.share) }}</text>
              </view>
              <view class="contribution-row__bar">
                <view class="contribution-row__fill contribution-row__fill--orange" :style="{ width: barWidth(item.share) }" />
              </view>
            </view>
          </view>

          <button
            v-if="shouldShowToggle(selectedInsight.goals_for_by_player)"
            class="insights-breakdown__toggle"
            @click="goalsForPlayerExpanded = !goalsForPlayerExpanded"
          >
            {{ goalsForPlayerExpanded ? '收起' : '更多' }}
          </button>
        </view>

        <view class="panel insights-breakdown">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">助攻贡献</text>
              <text class="section-title">球员维度</text>
            </view>
          </view>

          <view class="contribution-list">
            <view
              v-for="item in visibleAssistsForByPlayer"
              :key="assistContributionInstanceKey(item)"
              class="contribution-row"
            >
              <view class="contribution-row__header">
                <view class="contribution-row__identity">
                  <image :src="item.avatar_storage_url || ''" mode="aspectFill" class="contribution-row__avatar contribution-row__avatar--player" />
                  <view>
                    <text class="contribution-row__name">{{ item.player_name }}</text>
                    <text class="contribution-row__note">{{ item.assists }} 次</text>
                  </view>
                </view>
                <text class="contribution-row__share">{{ formatShare(item.share) }}</text>
              </view>
              <view class="contribution-row__bar contribution-row__bar--assist">
                <view class="contribution-row__fill contribution-row__fill--green" :style="{ width: barWidth(item.share) }" />
              </view>
            </view>
          </view>

          <button
            v-if="shouldShowToggle(selectedInsight.assists_for_by_player)"
            class="insights-breakdown__toggle"
            @click="assistsForPlayerExpanded = !assistsForPlayerExpanded"
          >
            {{ assistsForPlayerExpanded ? '收起' : '更多' }}
          </button>
        </view>

        <view class="panel insights-breakdown">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">失球贡献</text>
              <text class="section-title">对手维度</text>
            </view>
          </view>

          <view class="contribution-list">
            <view
              v-for="item in visibleGoalsAgainstByOpponent"
              :key="contributionInstanceKey('goals-against-opponent', item)"
              class="contribution-row"
            >
              <view class="contribution-row__header">
                <view class="contribution-row__identity">
                  <image :src="item.opponent_avatar_storage_url || ''" mode="aspectFit" class="contribution-row__avatar" />
                  <view>
                    <text class="contribution-row__name">{{ item.opponent_team_name }}</text>
                    <text class="contribution-row__note">{{ item.goals }} 球</text>
                  </view>
                </view>
                <text class="contribution-row__share">{{ formatShare(item.share) }}</text>
              </view>
              <view class="contribution-row__bar contribution-row__bar--danger">
                <view class="contribution-row__fill contribution-row__fill--danger" :style="{ width: barWidth(item.share) }" />
              </view>
            </view>
          </view>

          <button
            v-if="shouldShowToggle(selectedInsight.goals_against_by_opponent)"
            class="insights-breakdown__toggle"
            @click="goalsAgainstOpponentExpanded = !goalsAgainstOpponentExpanded"
          >
            {{ goalsAgainstOpponentExpanded ? '收起' : '更多' }}
          </button>
        </view>
      </template>

      <view v-else class="state-card">
        <text>当前还没有可展示的球队洞察数据。</text>
      </view>
    </view>
  </scroll-view>
  </view>
</template>

<script setup lang="ts">
import { computed, getCurrentInstance, nextTick, ref, watch } from 'vue'
import { onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import { getCurrentUser } from '../../api/auth'
import { getLiveTeamInsights } from '../../api/insight'
import type {
  AssistContribution,
  OpponentContribution,
  PlayerContribution,
  TeamInsight,
  TeamInsightTeam,
} from '../../types/insight'
import type { CurrentUser } from '../../types/auth'
import { extractApiErrorMessage } from '../../utils/apiError'
import bgImage from '../../static/insights/bg.webp'
import { useAnimatedInteger } from '../../composables/useAnimatedInteger'
import { resolveMembershipBenefitsLocked } from '../../utils/membershipBenefits'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'
import { reportPageActivity } from '../../utils/userActivity'

const instance = getCurrentInstance()
const userLoading = ref(true)
const loading = ref(false)
const errorMessage = ref('')
const currentUser = ref<CurrentUser | null>(null)
const teams = ref<TeamInsightTeam[]>([])
const insights = ref<TeamInsight[]>([])
const selectedTeamId = ref<number | null>(null)
const teamSelectorScrollLeft = ref(0)
const goalsForOpponentExpanded = ref(false)
const goalsForPlayerExpanded = ref(false)
const assistsForPlayerExpanded = ref(false)
const goalsAgainstOpponentExpanded = ref(false)
const ticketWatchNavigating = ref(false)
const systemConfigUnderReview = ref(false)
const membershipBenefitsLocked = computed(() =>
  resolveMembershipBenefitsLocked(currentUser.value),
)

const previewTeams = ['成都蓉城', '上海申花', '武汉三镇', '山东泰山']
const previewGoalAgainstRows = [
  { name: '青岛西海岸', width: '76%' },
  { name: '深圳新鹏城', width: '58%' },
  { name: '重庆铜梁龙', width: '34%' },
]
const previewGoalPlayerRows = [
  { name: '席尔瓦', width: '54%' },
  { name: '费利佩', width: '30%' },
  { name: '其他 / 未归因', width: '18%' },
]
const previewAssistRows = [
  { name: '费利佩', width: '64%' },
  { name: '席尔瓦', width: '46%' },
  { name: '其他 / 未归因', width: '40%' },
]

const selectedInsight = computed<TeamInsight | null>(() =>
  insights.value.find((item) => item.team_id === selectedTeamId.value) ?? null,
)
const animatedGoalsForTotal = useAnimatedInteger(() => selectedInsight.value?.goals_for_total ?? 0)
const animatedGoalsAgainstTotal = useAnimatedInteger(() => selectedInsight.value?.goals_against_total ?? 0)

const visibleGoalsForByOpponent = computed<OpponentContribution[]>(() =>
  getVisibleContributions(selectedInsight.value?.goals_for_by_opponent ?? [], goalsForOpponentExpanded.value),
)

const visibleGoalsForByPlayer = computed<PlayerContribution[]>(() =>
  getVisibleContributions(selectedInsight.value?.goals_for_by_player ?? [], goalsForPlayerExpanded.value),
)

const visibleAssistsForByPlayer = computed<AssistContribution[]>(() =>
  getVisibleContributions(selectedInsight.value?.assists_for_by_player ?? [], assistsForPlayerExpanded.value),
)

const visibleGoalsAgainstByOpponent = computed<OpponentContribution[]>(() =>
  getVisibleContributions(selectedInsight.value?.goals_against_by_opponent ?? [], goalsAgainstOpponentExpanded.value),
)

onShareAppMessage(() => ({
  title: selectedInsight.value
    ? `${selectedInsight.value.team_name} 的贡献结构，看看这队靠谁赢球`
    : '球队洞察：不是榜单，而是贡献结构',
  path: '/pages/insights/index',
}))

function getVisibleContributions<T>(items: T[], expanded: boolean): T[] {
  return expanded ? items : items.slice(0, 3)
}

function shouldShowToggle<T>(items: T[]): boolean {
  return items.length > 3
}

function formatShare(value: number): string {
  return `${(value * 100).toFixed(1)}%`
}

function barWidth(value: number): string {
  if (value <= 0) {
    return '0%'
  }

  return `${Math.max(value * 100, 8)}%`
}

function contributionKey(item: OpponentContribution | PlayerContribution): string {
  if ('opponent_team_id' in item) {
    return `team-${item.opponent_team_id}`
  }

  return `player-${item.player_id ?? item.player_name}`
}

function assistContributionKey(item: AssistContribution): string {
  return `assist-${item.player_id ?? item.player_name}`
}

function contributionInstanceKey(
  scope: 'goals-for-opponent' | 'goals-for-player' | 'goals-against-opponent',
  item: OpponentContribution | PlayerContribution,
): string {
  return `${selectedTeamId.value ?? 'none'}-${scope}-${contributionKey(item)}`
}

function assistContributionInstanceKey(item: AssistContribution): string {
  return `${selectedTeamId.value ?? 'none'}-assists-for-player-${assistContributionKey(item)}`
}

function openTicketWatch(): void {
  if (systemConfigUnderReview.value) {
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

function hasRectShape(value: unknown): value is { left: number; width: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { left?: unknown }).left === 'number'
    && typeof (value as { width?: unknown }).width === 'number'
}

function hasScrollLeft(value: unknown): value is { scrollLeft: number } {
  return !!value && typeof value === 'object'
    && typeof (value as { scrollLeft?: unknown }).scrollLeft === 'number'
}

async function centerSelectedChip(): Promise<void> {
  if (!instance || selectedTeamId.value === null) {
    return
  }

  await nextTick()

  const query = uni.createSelectorQuery().in(instance)
  query.select('.insights-team-selector__scroll').boundingClientRect()
  query.select('.insights-team-selector__scroll').scrollOffset(() => {})
  query.select(`#team-chip-${selectedTeamId.value}`).boundingClientRect()
  query.exec((result) => {
    const [rawScrollRect, rawScrollOffset, rawChipRect] = (result ?? []) as unknown[]

    if (!hasRectShape(rawScrollRect) || !hasScrollLeft(rawScrollOffset) || !hasRectShape(rawChipRect)) {
      return
    }

    const scrollRect = rawScrollRect
    const scrollOffset = rawScrollOffset
    const chipRect = rawChipRect
    const delta = (chipRect.left + chipRect.width / 2) - (scrollRect.left + scrollRect.width / 2)
    const nextScrollLeft = Math.max(0, Math.round(scrollOffset.scrollLeft + delta))

    if (nextScrollLeft !== teamSelectorScrollLeft.value) {
      teamSelectorScrollLeft.value = nextScrollLeft
    }
  })
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
    await centerSelectedChip()
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '洞察加载失败，请稍后重试。')
  } finally {
    loading.value = false
  }
}

function resetExpandedState(): void {
  goalsForOpponentExpanded.value = false
  goalsForPlayerExpanded.value = false
  assistsForPlayerExpanded.value = false
  goalsAgainstOpponentExpanded.value = false
}

watch(selectedTeamId, () => {
  resetExpandedState()
  void centerSelectedChip()
})

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
  height: 100vh;
  position: relative;
  z-index: 1;
}

.page {
  padding: 24rpx 16rpx 40rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
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
  background: linear-gradient(180deg, transparent 45%, rgba(247,248,250,0.55) 78%, #f7f8fa 100%);
  pointer-events: none;
  z-index: 0;
}

.hero-card,
.panel,
.state-card {
  background: rgba(255, 255, 255, 0.94);
  border-radius: 36rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
  padding: 20rpx;
}

.hero-card__top,
.section-heading,
.contribution-row__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hero-card__top,
.section-heading {
  align-items: flex-start;
  gap: 12rpx;
}

.eyebrow,
.section-kicker {
  margin: 0;
  color: #8f9198;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.hero-card__title,
.section-title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: 48rpx;
  line-height: 1.08;
  font-weight: 800;
}

.section-title {
  font-size: 44rpx;
}

.hero-card__badge,
.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 14rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 223, 205, 0.92);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: #a3916f;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}

.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: 10rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(214, 184, 131, 0.9), rgba(197, 163, 103, 0.72));
  box-shadow: 0 0 0 6rpx rgba(214, 184, 131, 0.14);
}

.meta-note--hero {
  padding-top: 14rpx;
}

.meta-pill--button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
}

.hero-card__summary,
.insights-lock-overlay__copy,
.insights-board-entry__copy,
.insights-ticket-watch-entry__copy {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}

.insights-locked {
  position: relative;
  overflow: hidden;
  min-height: 1080rpx;
  padding: 0;
}

.insights-locked__preview {
  padding: 20rpx;
  filter: blur(4rpx);
  opacity: 0.9;
}

.insights-locked__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 12rpx;
}

.insights-locked__chips text {
  padding: 14rpx 22rpx;
  border-radius: 999rpx;
  background: #f1f2f6;
  color: #6d7280;
  font-size: 24rpx;
}

.insights-locked__summary {
  margin-top: 20rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16rpx;
}

.insights-locked__metric {
  display: grid;
  gap: 10rpx;
  padding: 22rpx;
  border-radius: 24rpx;
  background: linear-gradient(180deg, rgba(255, 247, 234, 0.95), rgba(255, 255, 255, 0.94));
  color: #7c8089;
  font-size: 22rpx;
}

.insights-locked__metric-value {
  color: #15161b;
  font-size: 56rpx;
  font-weight: 800;
}

.insights-locked__rows {
  margin-top: 20rpx;
  display: grid;
  gap: 16rpx;
}

.insights-locked__section {
  display: grid;
  gap: 12rpx;
  padding: 22rpx;
  border-radius: 24rpx;
  background: rgba(255, 255, 255, 0.82);
}

.insights-locked__section-title {
  color: #15161b;
  font-size: 24rpx;
  font-weight: 700;
}

.insights-locked__row {
  display: grid;
  grid-template-columns: 180rpx 1fr;
  gap: 16rpx;
  align-items: center;
}

.insights-locked__row text {
  color: #6b707b;
  font-size: 22rpx;
}

.insights-locked__row > view {
  height: 14rpx;
  border-radius: 999rpx;
  background: #edf0f5;
  overflow: hidden;
}

.insights-locked__row > view > view {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #15161b, #515563);
}

.insights-lock-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 28rpx;
  background: linear-gradient(180deg, rgba(243, 243, 246, 0.18), rgba(243, 243, 246, 0.58));
}

.insights-lock-overlay__card {
  width: 100%;
  padding: 30rpx 28rpx;
  border-radius: 32rpx;
  background: rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(18rpx);
  box-shadow: 0 24rpx 50rpx rgba(18, 18, 18, 0.1);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.insights-lock-overlay__title {
  display: block;
  margin-top: 10rpx;
  color: #121212;
  font-size: 42rpx;
  font-weight: 800;
}

.insights-lock-overlay__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 20rpx;
  align-self: flex-start;
  padding: 20rpx 30rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 28rpx;
  white-space: nowrap;
  line-height: 1;
}

.insights-team-selector__scroll {
  margin-top: 18rpx;
  white-space: nowrap;
}

.insights-team-selector__list {
  display: inline-flex;
  gap: 12rpx;
  padding-bottom: 6rpx;
}

.insights-team-chip {
  display: inline-flex;
  align-items: center;
  gap: 12rpx;
  padding: 16rpx 22rpx;
  border-radius: 999rpx;
  background: #f1f2f6;
  color: #6d7280;
  font-size: 24rpx;
  line-height: 1;
  transition: transform 220ms ease, box-shadow 220ms ease, background-color 220ms ease, color 220ms ease;
}

.insights-team-chip.active {
  background: #15161b;
  color: #ffffff;
  transform: translateY(-2rpx);
  box-shadow: 0 10rpx 24rpx rgba(21, 22, 27, 0.14);
}

.insights-team-chip__avatar {
  width: 38rpx;
  height: 38rpx;
}

.insights-team-chip__avatar--fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: linear-gradient(135deg, #f97316, #ea580c);
  color: #ffffff;
  font-size: 22rpx;
  font-weight: 700;
  line-height: 1;
}

.insights-summary__grid {
  margin-top: 18rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12rpx;
}

.briefing-card {
  border-radius: 28rpx;
  border: 2rpx solid #ececf1;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(248, 250, 253, 0.98));
  padding: 18rpx 20rpx;
}

.briefing-card__label {
  color: #8f9198;
  font-size: 24rpx;
}

.briefing-card__body--summary {
  display: grid;
  min-height: 130rpx;
  align-items: end;
}

.briefing-card__main {
  display: grid;
  gap: 6rpx;
}

.insights-summary__value {
  display: inline-block;
  color: #121212;
  font-size: 60rpx;
  font-weight: 800;
  line-height: 0.92;
}

.insights-summary__value--animated {
  animation: fi-value-pop 420ms cubic-bezier(0.22, 1, 0.36, 1);
  transform-origin: center bottom;
}

.briefing-card__subvalue {
  color: #8f9198;
  font-size: 22rpx;
}

.contribution-list {
  margin-top: 14rpx;
  display: grid;
  gap: 12rpx;
}

.contribution-row {
  display: grid;
  gap: 10rpx;
  animation: fi-rise-in 220ms ease both;
}

.contribution-row__identity {
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.contribution-row__avatar {
  width: 48rpx;
  height: 48rpx;
}

.contribution-row__avatar--player {
  border-radius: 999rpx;
}

.contribution-row__name {
  display: block;
  color: #121212;
  font-size: 26rpx;
  font-weight: 700;
}

.contribution-row__note,
.contribution-row__share {
  color: #767a84;
  font-size: 22rpx;
}

.contribution-row__bar {
  height: 14rpx;
  border-radius: 999rpx;
  background: #edf0f5;
  overflow: hidden;
}

.contribution-row__fill {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #15161b, #515563);
  transition: width 320ms cubic-bezier(0.22, 1, 0.36, 1);
}

.contribution-row__fill--orange {
  background: linear-gradient(90deg, #ff6a00, #ffb347);
}

.contribution-row__fill--green {
  background: linear-gradient(90deg, #0dbd73, #59df9c);
}

.contribution-row__bar--danger .contribution-row__fill--danger {
  background: linear-gradient(90deg, #ef4444, #fb7185);
}

.insights-breakdown__toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-top: 18rpx;
  align-self: flex-start;
  padding: 12rpx 22rpx;
  border-radius: 999rpx;
  background: rgba(255, 106, 0, 0.12);
  color: #ff6a00;
  font-size: 24rpx;
  white-space: nowrap;
  line-height: 1;
}

.state-card--error text {
  font-size: 28rpx;
  color: #c03a2b;
}
</style>
