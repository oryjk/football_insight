<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBackFromSecondaryPage('/pages/insights/index')" />
  <view class="transfer-entry-page">
    <view class="transfer-entry-heading">
      <text class="transfer-entry-title">转会表态</text>
      <text class="transfer-entry-caption">送走谁、留下谁、买回谁，说出你的想法</text>
    </view>
    <FiPanel>
      <view class="transfer-entry-workspace">
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
        <InsightsTransferPanel
          :summary="transferSummary" :loading="transferLoading || teamLoading" :error="teamError || transferError"
          :hint="transferHint" :pending="transferPending"
          @retry="retryTransfer" @vote="transfer.vote" @open="openTransferCard" @more="openTransferBoard" @nominate="openTransferNominate"
        />
      </view>
    </FiPanel>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onHide, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import InsightsTeamSelector from '../../components/InsightsTeamSelector.vue'
import InsightsTransferPanel from './components/InsightsTransferPanel.vue'
import { useInsightsTeams } from '../../composables/useInsightsTeams'
import { useInsightsTransfer } from './useInsightsTransfer'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import type { TransferIntent } from '../../types/transfer'
import { buildTransferBoardUrl, buildTransferViewUrl, transferBoardShareTitle } from '../../utils/transferLinks'
import { buildSharePath } from '../../utils/shareSource'
import { goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { reportPageActivity } from '../../utils/userActivity'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const pageVisible = ref(false)
const teamBrowser = useInsightsTeams(session)
const { teams: selectableTeams, selectedTeamId, loading: teamLoading, error: teamError } = teamBrowser
const transfer = useInsightsTransfer({ session, selectedTeamId, active: pageVisible })
const { summary: transferSummary, loading: transferLoading, error: transferError, hint: transferHint, pending: transferPending } = transfer
const selectedTeamName = computed(() => selectableTeams.value.find(team => team.team_id === selectedTeamId.value)?.team_name ?? '球队')
function retryTransfer(): void {
  if (teamError.value) void teamBrowser.load()
  else void transfer.refresh()
}
function openTransferCard(code: string): void {
  const url = buildTransferViewUrl(code)
  if (url) uni.navigateTo({ url })
}
function openTransferBoard(intent: TransferIntent): void {
  if (selectedTeamId.value !== null) uni.navigateTo({ url: buildTransferBoardUrl(selectedTeamId.value, intent) })
}
function openTransferNominate(): void {
  if (selectedTeamId.value === null) return
  // 已登录却不能投（非主队）时只提示，不跳去打开提名弹层。
  const blocked = transfer.nominateBlocked()
  if (blocked) { uni.showToast({ title: blocked, icon: 'none' }); return }
  uni.navigateTo({ url: buildTransferBoardUrl(selectedTeamId.value, 'out', { nominate: true }) })
}
onShareAppMessage(() => {
  const windowName = transferSummary.value?.window?.name
  if (windowName && selectedTeamId.value !== null) {
    return { title: transferBoardShareTitle(selectedTeamName.value, windowName, 'out'), path: buildSharePath(buildTransferBoardUrl(selectedTeamId.value, 'out'), 'transfer_board') }
  }
  return { title: `${selectedTeamName.value}转会表态，说说你的想法`, path: buildSharePath('/pages/insights-transfer/index', 'insights') }
})
onHide(() => { pageVisible.value = false })
onUnload(() => { transfer.dispose(); teamBrowser.dispose() })
onShow(() => {
  pageVisible.value = true
  void teamBrowser.load()
  void transfer.refresh()
  reportPageActivity('insights')
})
</script>

<style scoped lang="css">
.transfer-entry-page { min-height: 100vh; box-sizing: border-box; display: flex; flex-direction: column; gap: var(--fi-space-card-gap); padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-16) calc(var(--fi-space-64) + env(safe-area-inset-bottom)); background: var(--fi-color-page-soft); }
.transfer-entry-heading { padding: 0 var(--fi-space-8); }
.transfer-entry-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); }
.transfer-entry-caption { display: block; margin-top: var(--fi-space-12); color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.transfer-entry-workspace { display: flex; flex-direction: column; gap: var(--fi-space-24); min-width: 0; }
.state-card { padding: var(--fi-space-16) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); }
</style>
