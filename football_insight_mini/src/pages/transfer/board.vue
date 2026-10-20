<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" />
  <view class="transfer-page">
    <text class="transfer-title">{{ board?.team.team_name ?? '' }}球迷的转会表态</text>
    <text v-if="!teamId" class="transfer-caption">先在首页选择你的主队，再来给转会表态。</text>
    <template v-else>
      <text v-if="board?.window" class="transfer-caption">{{ board.window.name }} · {{ board.window.open ? '表态中，提名一位球员或给大家的提名点赞成 / 反对' : '本期已结束' }}</text>
      <FiSegmented :items="intentItems" :model-value="intent" aria-label="转会榜" @update:model-value="changeIntent($event as TransferIntent)" />
      <text v-if="error" class="transfer-error">{{ error }}</text>
      <FiLoading v-if="loading && !items.length" title="转会榜加载中" caption="正在读取球迷的提名。" />
      <view v-if="items.length" class="transfer-panel">
        <view v-for="card in items" :key="card.share_code" class="transfer-item">
          <FiTransferCard :card="card" :pending="pending.includes(card.share_code)" @open="openCard(card.share_code)" @vote="vote(card, $event)" />
        </view>
      </view>
      <text v-if="!loading && !error && !items.length" class="transfer-caption">还没人提名，你来说第一句。</text>
      <button v-if="error || hasMore" class="transfer-action" :disabled="loading" @tap="retry">{{ loading ? '加载中…' : error ? '重试加载' : '加载更多' }}</button>
      <view v-if="board?.window?.open" class="transfer-dock">
        <button class="transfer-action transfer-action--active" @tap="openNominate">我要提名</button>
      </view>
    </template>
    <TransferNominateSheet v-model:visible="nominate.visible.value" :intent="nominate.intent.value" :groups="nominate.groups.value"
      :loading="nominate.loading.value" :error="nominate.error.value" :submitting="nominate.submitting.value"
      @update:intent="nominate.changeIntent" @retry="nominate.retry()" @pick="pick" />
  </view>
</template>
<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { onLoad, onReachBottom, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiSegmented from '../../components/FiSegmented.vue'
import FiTransferCard from '../../components/FiTransferCard.vue'
import TransferNominateSheet from './components/TransferNominateSheet.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import type { TransferIntent } from '../../types/transfer'
import { buildSharePath } from '../../utils/shareSource'
import { TRANSFER_BOARD_LABELS, TRANSFER_INTENTS, buildTransferBoardUrl, buildTransferViewUrl, parseTransferBoardQuery, transferBoardShareTitle } from '../../utils/transferLinks'
import { createTransferBoard } from './useTransferBoard'
import { createTransferNominate } from './useTransferNominate'

const { navigationPageStyle } = useNavigationLayout()
const state = createTransferBoard()
const { teamId, intent, board, items, loading, error, hasMore, pending, vote } = state
const nominate = createTransferNominate()
const intentItems = TRANSFER_INTENTS.map(key => ({ key, label: TRANSFER_BOARD_LABELS[key] }))
const wantsNominate = ref(false)
let started = false

onLoad(query => {
  const parsed = parseTransferBoardQuery(query as Record<string, unknown>)
  teamId.value = parsed.teamId; intent.value = parsed.intent; wantsNominate.value = parsed.nominate
})
onShow(() => {
  if (!teamId.value) return
  // 返回本页（例如从单卡页赞踩后）重新读当前榜，保持名次与票数一致。
  void state.load(teamId.value, intent.value).then(() => {
    if (!started && wantsNominate.value && board.value?.window?.open) openNominate()
    started = true
  })
})
onReachBottom(() => { void state.more() })
onShareAppMessage(() => ({
  title: board.value?.window ? transferBoardShareTitle(board.value.team.team_name, board.value.window.name, intent.value) : '球迷转会榜',
  path: buildSharePath(buildTransferBoardUrl(teamId.value, intent.value), 'transfer_board'),
}))
onBeforeUnmount(state.dispose)

function changeIntent(which: TransferIntent): void { void state.changeIntent(which) }
function retry(): void { if (error.value) void state.load(teamId.value, intent.value); else void state.more() }
function openCard(code: string): void { const url = buildTransferViewUrl(code); if (url) uni.navigateTo({ url }) }
function openNominate(): void {
  // 已登录却不能投（非主队）时只提示，不打开提名弹层；游客由 nominate.open 引导登录。
  const blocked = state.nominateBlocked()
  if (blocked) { uni.showToast({ title: blocked, icon: 'none' }); return }
  void nominate.open(teamId.value, intent.value)
}
async function pick(playerId: number): Promise<void> {
  const result = await nominate.submit(playerId)
  if (result) void state.load(teamId.value, nominate.intent.value)
}
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
</script>
<style scoped src="./transfer.css"></style>
