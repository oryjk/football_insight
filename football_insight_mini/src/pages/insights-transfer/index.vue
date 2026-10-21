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
        <view v-if="homeTeam.loading.value && !homeTeam.teams.value.length" class="state-card"><text>正在加载…</text></view>
        <view v-else-if="homeTeam.error.value && !homeTeam.needsPick.value && !homeTeam.teamId.value" class="state-card">
          <text>{{ homeTeam.error.value }}</text>
          <button @tap="homeTeam.load()">重试</button>
        </view>
        <view v-else-if="homeTeam.needsPick.value" class="pick">
          <text class="pick__title">先选择你的主队</text>
          <text class="pick__caption">转会表态只对你的主队，选定后会同步为首页的「我的主队」。</text>
          <view class="pick__grid">
            <view v-for="item in homeTeam.teams.value" :key="item.team_id" class="pick__item" :class="{ 'pick__item--selected': pendingTeamId === item.team_id }" role="button" :aria-label="item.team_name" hover-class="pick__item--pressed" @tap="pendingTeamId = item.team_id">
              <image v-if="item.avatar_storage_url" :src="item.avatar_storage_url" class="pick__avatar" mode="aspectFit" />
              <text class="pick__name">{{ item.team_name }}</text>
            </view>
          </view>
          <text v-if="homeTeam.error.value" class="pick__error">{{ homeTeam.error.value }}</text>
          <button class="pick__confirm" :disabled="pendingTeamId === null || homeTeam.submitting.value" @tap="confirmPick">{{ homeTeam.submitting.value ? '保存中…' : '确认主队' }}</button>
        </view>
        <template v-else-if="homeTeam.team.value">
          <view class="home-team">
            <image v-if="homeTeam.team.value.avatar_storage_url" :src="homeTeam.team.value.avatar_storage_url" class="home-team__avatar" mode="aspectFit" />
            <text class="home-team__name">{{ homeTeam.team.value.team_name }}</text>
            <text class="home-team__tag">{{ homeTeam.loggedIn.value ? '我的主队' : '登录后为主队表态' }}</text>
          </view>
          <InsightsTransferPanel
            :summary="transferSummary" :loading="transferLoading" :error="transferError"
            :hint="transferHint" :pending="transferPending"
            @retry="retryTransfer" @vote="transfer.vote" @open="openTransferCard" @more="openTransferBoard" @nominate="openTransferNominate"
          />
        </template>
      </view>
    </FiPanel>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { onHide, onShareAppMessage, onShow, onUnload } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiPanel from '../../components/FiPanel.vue'
import InsightsTransferPanel from './components/InsightsTransferPanel.vue'
import { useTransferHomeTeam } from './useTransferHomeTeam'
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
const homeTeam = useTransferHomeTeam(session)
const selectedTeamId = homeTeam.teamId
const pendingTeamId = ref<number | null>(null)
const teamActive = computed(() => pageVisible.value && !homeTeam.needsPick.value)
const transfer = useInsightsTransfer({ session, selectedTeamId, active: teamActive })
const { summary: transferSummary, loading: transferLoading, error: transferError, hint: transferHint, pending: transferPending } = transfer
const selectedTeamName = computed(() => homeTeam.team.value?.team_name ?? '球队')
async function confirmPick(): Promise<void> {
  if (pendingTeamId.value === null) return
  await homeTeam.pick(pendingTeamId.value)
  if (!homeTeam.needsPick.value) void transfer.refresh()
}
function retryTransfer(): void { void transfer.refresh() }
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
onUnload(() => { transfer.dispose(); homeTeam.dispose() })
onShow(() => {
  pageVisible.value = true
  void homeTeam.load().then(() => transfer.refresh())
  reportPageActivity('insights')
})
</script>

<style scoped lang="css">
.home-team { display: flex; align-items: center; gap: var(--fi-space-12); padding: var(--fi-space-12) var(--fi-space-16); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); align-self: flex-start; }
.home-team__avatar { width: var(--fi-space-44); height: var(--fi-space-44); }
.home-team__name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.home-team__tag { color: var(--fi-color-primary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.pick { display: flex; flex-direction: column; gap: var(--fi-space-16); }
.pick__title { color: var(--fi-color-text-strong); font-size: var(--fi-font-32); font-weight: var(--fi-weight-extrabold); }
.pick__caption { color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.pick__grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: var(--fi-space-12); }
.pick__item { display: flex; flex-direction: column; align-items: center; gap: var(--fi-space-8); padding: var(--fi-space-16) var(--fi-space-8); border-radius: var(--fi-radius-md); background: var(--fi-color-page); border: var(--fi-border-width) solid transparent; }
.pick__item--selected { border-color: var(--fi-color-primary); background: var(--fi-color-surface); }
.pick__item--pressed { opacity: var(--fi-opacity-pressed); }
.pick__avatar { width: var(--fi-space-56); height: var(--fi-space-56); }
.pick__name { max-width: 100%; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; color: var(--fi-color-text-strong); font-size: var(--fi-font-24); }
.pick__error { color: var(--fi-color-primary); font-size: var(--fi-font-24); }
.pick__confirm { margin: 0; padding: var(--fi-space-20) var(--fi-space-24); border: 0; border-radius: var(--fi-radius-round); background: var(--fi-color-primary); color: var(--fi-color-surface); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.pick__confirm::after { border: 0; }
.pick__confirm[disabled] { background: var(--fi-color-divider); color: var(--fi-color-text-muted); }
.transfer-entry-page { min-height: 100vh; box-sizing: border-box; display: flex; flex-direction: column; gap: var(--fi-space-card-gap); padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-16) calc(var(--fi-space-64) + env(safe-area-inset-bottom)); background: var(--fi-color-page-soft); }
.transfer-entry-heading { padding: 0 var(--fi-space-8); }
.transfer-entry-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); }
.transfer-entry-caption { display: block; margin-top: var(--fi-space-12); color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.transfer-entry-workspace { display: flex; flex-direction: column; gap: var(--fi-space-24); min-width: 0; }
.state-card { padding: var(--fi-space-16) 0; color: var(--fi-color-text-caption); font-size: var(--fi-font-24); }
</style>
