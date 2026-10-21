<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBackFromSecondaryPage('/pages/insights/index')" />
  <view class="tactics-page">
    <view class="tactics-heading">
      <text class="tactics-title">排阵容</text>
      <text class="tactics-caption">看看大家怎么排，也排一套你的阵容</text>
    </view>
    <FiPanel>
      <view class="tactics-workspace">
        <InsightsTeamSelector
          v-if="selectableTeams.length"
          :teams="selectableTeams"
          :selected-team-id="selectedTeamId"
          @update:selected-team-id="teamBrowser.selectTeam"
        />
        <view v-else class="state-card">
          <text>{{ teamError || '正在加载球队…' }}</text>
          <button v-if="teamError" @tap="teamBrowser.load()">重试</button>
        </view>
        <InsightsTacticsPanel
          :team-name="selectedTeamName"
          :authenticated="tacticsAuthenticated"
          :mine-loading="mineLoading" :mine-error="mineError" :draft="mineDraft" :published="minePublished"
          :items="communityItems" :total="communityTotal" :sort="communitySort"
          :loading="communityLoading || teamLoading" :error="teamError || communityError" :pending-likes="communityPendingLikes"
          :consensus="consensus" :my-overlap="myOverlap" :my-player-ids="myPlayerIds" @explain="showHint"
          @edit="goToTeamBoard" @retry-mine="tactics.retryMine()" @retry-list="retryTacticsList"
          @sort="tactics.changeSort" @like="tactics.like" @dislike="tactics.dislike" @open="openPublicTactics" @more="tactics.more()"
        />
      </view>
    </FiPanel>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { onHide, onReachBottom, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import InsightsTeamSelector from '../../components/InsightsTeamSelector.vue'
import InsightsTacticsPanel from './components/InsightsTacticsPanel.vue'
import { useInsightsTeams } from '../../composables/useInsightsTeams'
import { useInsightsTactics } from './useInsightsTactics'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { createLineupConsensus } from '../../composables/useLineupConsensus'
import { lineupOverlap, overlapText } from '../../utils/lineupConsensus'
import { buildLineupPageUrl, buildLineupViewUrl } from '../../utils/lineupLinks'
import { buildSharePath } from '../../utils/shareSource'
import { goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { reportPageActivity } from '../../utils/userActivity'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const pageVisible = ref(false)
const teamBrowser = useInsightsTeams(session)
const { teams: selectableTeams, selectedTeamId, loading: teamLoading, error: teamError } = teamBrowser
const tactics = useInsightsTactics({ session, selectedTeamId, active: pageVisible })
const { mineLoading, mineError, mineDraft, minePublished, authenticated: tacticsAuthenticated } = tactics
const { items: communityItems, total: communityTotal, sort: communitySort, loading: communityLoading, error: communityError, pendingLikes: communityPendingLikes } = tactics.community
const lineupConsensus = createLineupConsensus()
const consensus = lineupConsensus.consensus
// 球队变化或页面重新可见（含 onShow）时刷新共识
watch([selectedTeamId, pageVisible], ([id, active]) => {
  if (active && id !== null) void lineupConsensus.load(id)
}, { immediate: true })
const myPlayerIds = computed(() => (minePublished.value?.slots ?? []).map(slot => slot.player_id).filter((id): id is number => id !== null))
const myOverlap = computed(() => minePublished.value
  ? overlapText(lineupOverlap(minePublished.value.slots.map(slot => slot.player_id), consensus.value), '你')
  : '')
const selectedTeamName = computed(() => selectableTeams.value.find(team => team.team_id === selectedTeamId.value)?.team_name ?? '')
function showHint(message: string): void { uni.showToast({ title: message, icon: 'none' }) }
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
onShareAppMessage(() => ({
  title: `${selectedTeamName.value || '球队'}战术，看看大家怎么排`,
  path: buildSharePath('/pages/insights-tactics/index', 'insights'),
}))
onReachBottom(() => { void tactics.more() })
onHide(() => { pageVisible.value = false })
onUnload(() => { tactics.dispose(); teamBrowser.dispose() })
onShow(() => {
  pageVisible.value = true
  void teamBrowser.load()
  void tactics.refresh()
  reportPageActivity('insights')
})
</script>

<style scoped lang="css">
.tactics-page { min-height: 100vh; box-sizing: border-box; display: flex; flex-direction: column; gap: var(--fi-space-card-gap); padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-16) calc(var(--fi-space-64) + env(safe-area-inset-bottom)); background: var(--fi-color-page-soft); }
.tactics-heading { padding: 0 var(--fi-space-8); }
.tactics-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); }
.tactics-caption { display: block; margin-top: var(--fi-space-12); color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.tactics-workspace { display: flex; flex-direction: column; gap: var(--fi-space-24); min-width: 0; }
.state-card { padding: var(--fi-space-16) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); }
</style>
