<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" />
  <view class="transfer-page">
    <FiLoading v-if="loading && !view" title="提名加载中" caption="正在读取这张提名。" />
    <text v-else-if="error && !view" class="transfer-error">{{ error }}</text>
    <template v-else-if="view">
      <text class="transfer-title">{{ view.team.team_name }}球迷希望 {{ view.card.player.name }} {{ transferStance(view.card) }}</text>
      <text class="transfer-caption">{{ view.window.name }} · {{ TRANSFER_BOARD_LABELS[view.card.intent] }}{{ view.card.rank ? ` 第 ${view.card.rank}` : '' }}{{ view.window.open ? '' : ' · 本期投票已结束' }}</text>
      <view class="transfer-panel">
        <FiTransferCard size="md" :card="view.card" :show-rank="false" :pending="pending.includes(view.card.share_code)" @vote="vote" />
      </view>
      <text v-if="hint" class="transfer-caption">{{ hint }}</text>
      <view class="transfer-row">
        <button class="transfer-action transfer-action--active" open-type="share">分享拉票</button>
        <button class="transfer-action" @tap="openBoard">看完整榜单</button>
      </view>
      <button v-if="view.window.open" class="transfer-action" @tap="openNominate">我也提名</button>
    </template>
  </view>
</template>
<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import FiTransferCard from '../../components/FiTransferCard.vue'
import { getTransferCard } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps } from '../../composables/useTransferVotes'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import type { TransferCardView, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage } from '../../utils/apiError'
import { buildSharePath } from '../../utils/shareSource'
import { TRANSFER_BOARD_LABELS, applyTally, buildTransferBoardUrl, buildTransferViewUrl, parseTransferCode, transferBlockedMessage, transferCardShareTitle, transferHint, transferStance } from '../../utils/transferLinks'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const voteDeps = transferVoteDeps()
const votes = createTransferVotes(voteDeps)
const pending = votes.pending
const code = ref(''), view = ref<TransferCardView | null>(null), loading = ref(false), error = ref('')
let generation = 0, loadedRevision = -1
const eligibility = () => view.value
  ? { loggedIn: voteDeps.loggedIn(), canVote: view.value.can_vote, open: view.value.window.open }
  : null
const hint = computed(() => {
  const input = eligibility()
  return input ? transferHint(input) : ''
})
/** 已登录却不能投时返回原因；游客与能投的人返回空。 */
function blockedMessage(): string {
  const input = eligibility()
  return input ? transferBlockedMessage(input) : ''
}

async function load(): Promise<void> {
  if (!code.value) { error.value = '分享链接无效'; return }
  const owner = ++generation
  loading.value = true; error.value = ''
  try {
    const result = await getTransferCard(code.value)
    if (owner === generation) view.value = result
  } catch (cause) {
    if (owner === generation) error.value = extractApiErrorMessage(cause, '这张提名暂时打不开，请稍后重试')
  } finally {
    if (owner === generation) loading.value = false
  }
}
async function vote(value: TransferVoteValue): Promise<void> {
  const current = view.value
  if (!current) return
  const blocked = blockedMessage()
  if (blocked) {
    uni.showToast({ title: blocked, icon: 'none' })
    return
  }
  const owner = generation
  const message = await votes.toggle(current.card, value, {
    returnUrl: buildTransferViewUrl(code.value),
    current: () => owner === generation,
    publish: tally => { if (view.value) view.value = { ...view.value, card: applyTally(view.value.card, tally) } },
  })
  if (message) uni.showToast({ title: message, icon: 'none' })
}

onLoad(query => { code.value = parseTransferCode(query as Record<string, unknown>) })
onShow(() => {
  if (loadedRevision !== session.revision.value) { loadedRevision = session.revision.value; void load() }
})
onShareAppMessage(() => ({
  title: view.value ? transferCardShareTitle(view.value.card) : '球迷转会投票',
  path: buildSharePath(buildTransferViewUrl(code.value), 'transfer_card'),
}))
onBeforeUnmount(() => { generation++; votes.reset() })

function openBoard(): void { if (view.value) uni.navigateTo({ url: buildTransferBoardUrl(view.value.team.team_id, view.value.card.intent) }) }
function openNominate(): void {
  if (!view.value) return
  // 不能投的已登录用户在这里就提示，不再跳到榜单页打开提名弹层。
  const blocked = blockedMessage()
  if (blocked) { uni.showToast({ title: blocked, icon: 'none' }); return }
  uni.navigateTo({ url: buildTransferBoardUrl(view.value.team.team_id, view.value.card.intent, { nominate: true }) })
}
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
</script>
<style scoped src="./transfer.css"></style>
