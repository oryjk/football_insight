<template>
  <page-meta :page-style="navigationPageStyle + pageLockStyle" />
  <FiSecondaryNav :shareable="false" @back="goBack" />
  <view class="page-root">
    <view class="page-bg"></view>

    <!-- #ifdef H5 -->
    <view class="page page--notice">
      <view class="state-card"><text>请在微信小程序中使用战术板。</text></view>
    </view>
    <!-- #endif -->

    <!-- #ifndef H5 -->
    <view class="page">
      <view v-if="teams.length" class="team-picker">
        <LineupTeamPicker :teams="teams" :model-value="teamId" @update:model-value="switchTeam" />
      </view>
      <FiLoading
        v-if="loading"
        title="战术板加载中"
        caption="正在准备球队名单和你的阵容。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
        <button class="ghost-action" @tap="reload">重试</button>
      </view>

      <view v-else-if="squad" class="page-content">
        <view v-if="notice" class="notice"><text>{{ notice }}</text></view>

        <view class="status-row">
          <view class="status-row__heading">
            <text class="status-row__title">我的最佳阵容</text>
            <text class="status-row__state" :class="`status-row__state--${saveState}`">{{ saveStateText }}</text>
          </view>
          <button class="community-link" @tap="openCommunity">看看大家怎么排 ›</button>
        </view>

        <LineupFormationBar :model-value="editor.formation.value" @update:model-value="changeFormation" />

        <view class="pitch-slot">
          <LineupPitch
            :players="pitchPlayers"
            :selected-slot="editor.selectedSlot.value"
            fit-container
            :code-src="pitchCodeSrc"
            @tap="openSheet"
            @drag="editor.applyDrag"
          />
        </view>

        <view class="hint-row">
          <text class="hint">点选球员 · 按住拖动 · 已排 {{ editor.filled.value }}/11</text>
          <text v-if="editor.filled.value < 11" class="hint-link" @tap="fillStartingEleven">一键填入首发</text>
        </view>

        <text class="hint">草稿仅自己可见，满 11 人可发布并参与排名</text>

        <view class="action-slot">
          <LineupActionBar
            :busy="saving"
            :saving-draft="savingDraft"
            :publishing="publishing"
            :share-preparing="sharePreparing"
            :complete="editor.filled.value === 11"
            :published="Boolean(savedLineup)"
            @reset="requestConfirmation('reset')"
            @share="handleShare"
            @draft="handleDraft"
            @publish="requestConfirmation('publish')"
          />
        </view>
      </view>
    </view>

    <LineupPlayerSheet
      v-model:visible="sheetVisible"
      :slot-label="sheetSlotLabel"
      :occupied="sheetOccupied"
      :groups="sheetGroups"
      @pick="handlePick"
      @remove="handleRemove"
    />

    <LineupConfirmDialog
      :visible="confirmAction !== null"
      :title="confirmAction === 'reset' ? '确认重置？' : savedLineup ? '确认更新发布？' : '确认发布？'"
      :message="confirmationMessage"
      @cancel="confirmAction = null"
      @confirm="confirmPendingAction"
    />
    <LineupShareSheet v-model:visible="shareVisible" :busy="shareBusy" :ready="Boolean(sharePayload)" :image-path="imagePath" :error="shareError" @generate="generate" @preview="preview" @save="saveImage" />
    <canvas canvas-id="lineup-share-canvas" class="share-canvas" :style="{ width: '750px', height: '1200px' }" />
    <canvas canvas-id="lineup-card-canvas" class="share-canvas" :style="{ width: '750px', height: '600px' }" />
    <!-- #endif -->
  </view>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { onLoad, onShow, onShareAppMessage } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import LineupConfirmDialog from './components/LineupConfirmDialog.vue'
import { getSharedLineup } from '../../api/lineup'
import { extractApiErrorMessage } from '../../utils/apiError'
import { prepareEditorShare } from './lineupEditorShare'
import { useLineupShare } from './useLineupShare'
import LineupShareSheet from './components/LineupShareSheet.vue'
import { getSupportProfile, listSupportTeams } from '../../api/support'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { buildLineupListUrl, buildLineupPageUrl, parseLineupPageQuery } from '../../utils/lineupLinks'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { buildPitchPlayers, buildSheetGroups, positionIndex } from './lineupViewModel'
import { useLineupPage } from './useLineupPage'
import { useLineupPitchCode } from './useLineupPitchCode'
import LineupFormationBar from './components/LineupFormationBar.vue'
import LineupPitch from './components/LineupPitch.vue'
import LineupPlayerSheet from './components/LineupPlayerSheet.vue'
import LineupTeamPicker from './components/LineupTeamPicker.vue'
import LineupActionBar from './components/LineupActionBar.vue'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()

const { teamId, squad, loading, errorMessage, notice, saving, savingDraft, publishing, saveState, savedLineup, hasUnpublishedChanges, editor, load, saveDraft, publish, fillStartingEleven, dispose } = useLineupPage()
const { codeSrc: pitchCodeSrc } = useLineupPitchCode(() => savedLineup.value?.share_code)

const teams = ref<Array<{ team_id: number; team_name: string; avatar_url: string | null }>>([])
const sheetVisible = ref(false)
const confirmAction = ref<'publish' | 'reset' | null>(null)
const sharePreparing = ref(false)
const shareContext = () => JSON.stringify([teamId.value, editor.revision.value, session.revision.value, loading.value])
const share = useLineupShare(shareContext, () => buildLineupPageUrl(teamId.value))
const { visible: shareVisible, busy: shareBusy, error: shareError, imagePath, payload: sharePayload, generate, preview, save: saveImage } = share
watch(sharePayload, payload => {
  // #ifdef MP-WEIXIN
  if (payload) uni.showShareMenu({ menus: ['shareAppMessage'] }); else uni.hideShareMenu({ hideShareItems: ['shareAppMessage', 'shareTimeline'] })
  // #endif
}, { immediate: true })
onShareAppMessage(() => sharePayload.value ?? {})
let requestedTeamId = 0
let started = false

// 弹层打开时锁页面滚动（沿用换座页的 page-meta 约定）。
const pageLockStyle = computed(() => (sheetVisible.value || confirmAction.value !== null || shareVisible.value ? 'overflow: hidden;' : ''))
const confirmationMessage = computed(() => {
  if (confirmAction.value === 'reset') return '所有球员将恢复为当前阵型的默认站位，已选球员保持不变。'
  return savedLineup.value
    ? '用当前阵容更新已发布的战术？分享链接和已有点赞会保留。'
    : '发布当前战术阵容？发布后将在球队榜单公开展示，球迷可查看和点赞。'
})
watch([teamId, loading, saving, session.revision], () => { confirmAction.value = null })
watch(
  () => [session.currentUser.value?.id ?? null, session.hasAccessToken.value] as const,
  ([userId, loggedIn], previous) => {
    if (!started || !teamId.value || (previous?.[0] === userId && previous?.[1] === loggedIn)) return
    sheetVisible.value = false
    shareVisible.value = false
    // 后台登录、退出或换账号后重新装载该身份的云草稿；普通身份变化不移交游客稿。
    // load 会先把旧编辑落回旧 owner，旧请求由 loadSeq 与 session revision 作废。
    void load(teamId.value)
  },
)

const pitchPlayers = computed(() => buildPitchPlayers(editor.slots.value, editor.formation.value, squad.value))

const selectedEntry = computed(() => pitchPlayers.value.find((p) => p.slot === editor.selectedSlot.value) ?? null)
const sheetSlotLabel = computed(() => selectedEntry.value?.label ?? '选择球员')
const sheetOccupied = computed(() => Boolean(selectedEntry.value?.player))
const sheetGroups = computed(() => {
  if (!squad.value || !selectedEntry.value) return []
  return buildSheetGroups(squad.value, editor.slots.value, selectedEntry.value.position)
})

const saveStateText = computed(() => {
  switch (saveState.value) {
    case 'published':
      return '已发布'
    case 'draft':
      return '草稿已保存'
    case 'dirty':
      return savedLineup.value ? '有未发布改动' : '未保存草稿'
    case 'error':
      return '保存失败'
    default:
      return ''
  }
})

function goBack(): void {
  if (!editor.dirty.value) {
    navigateBack()
    return
  }
  uni.showModal({
    title: '草稿尚未存云端',
    content: '当前改动只保留在本机，下次进来可继续编辑。确定先离开吗？',
    confirmText: '离开',
    cancelText: '继续排',
    success: (res) => {
      if (res.confirm) navigateBack()
    },
  })
}

function navigateBack(): void {
  uni.navigateBack({
    fail: () => {
      uni.switchTab({ url: '/pages/insights/index' })
    },
  })
}

function openSheet(slot: number): void {
  editor.selectedSlot.value = slot
  sheetVisible.value = true
}

function handlePick(playerId: number): void {
  if (editor.selectedSlot.value === null) return
  editor.pick(editor.selectedSlot.value, playerId)
  sheetVisible.value = false
}

function handleRemove(): void {
  if (editor.selectedSlot.value === null) return
  editor.pick(editor.selectedSlot.value, null)
  sheetVisible.value = false
}

function changeFormation(code: string): void {
  editor.setFormation(code, positionIndex(squad.value))
}

function requestConfirmation(action: 'publish' | 'reset'): void {
  if (loading.value || !squad.value || saving.value || sharePreparing.value) return
  if (action === 'publish' && editor.filled.value < 11) return
  uni.hideToast()
  confirmAction.value = action
}

function confirmPendingAction(): void {
  const action = confirmAction.value
  confirmAction.value = null
  if (!action || loading.value || !squad.value || saving.value || sharePreparing.value) return
  if (action === 'reset') editor.reset()
  else void handlePublish()
}

async function switchTeam(nextTeamId: number): Promise<void> {
  if (nextTeamId === teamId.value) return
  if (!editor.dirty.value) {
    await load(nextTeamId)
    return
  }
  uni.showModal({
    title: '切换球队',
    content: '当前改动只保留在本机，切换后可以回来继续。',
    confirmText: '切换',
    cancelText: '取消',
    success: (res) => {
      if (res.confirm) void load(nextTeamId)
    },
  })
}

function loginForAction(action: 'draft' | 'publish'): void {
  rememberPostLoginRedirect({ type: 'navigateTo', url: buildLineupPageUrl(teamId.value, { resumeAction: action }) })
  uni.showToast({ title: action === 'draft' ? '登录后可将草稿存云端' : '登录后即可发布', icon: 'none' })
  uni.switchTab({ url: '/pages/user/index' })
}

async function handleDraft(): Promise<void> {
  const result = await saveDraft()
  if (result === 'cancelled') return
  if (result === 'saved') {
    uni.showToast({ title: '草稿已保存', icon: 'success' })
    return
  }
  if (result === 'need-login') {
    loginForAction('draft')
    return
  }
  uni.showToast({ title: '云端保存失败，已保留本机草稿', icon: 'none' })
}

async function handlePublish(): Promise<void> {
  const result = await publish()
  if (result === 'cancelled') return
  if (result === 'saved') uni.showToast({ title: '已发布', icon: 'success' })
  else if (result === 'need-login') loginForAction('publish')
  else if (result === 'incomplete') uni.showToast({ title: '排满 11 人后再发布', icon: 'none' })
  else uni.showToast({ title: '发布失败，已保留本机草稿', icon: 'none' })
}

function openCommunity(): void { uni.navigateTo({ url: buildLineupListUrl(teamId.value) }) }
async function handleShare(): Promise<void> {
  if (sharePreparing.value || loading.value || saving.value) return
  sharePreparing.value = true
  try {
    const prepared = await prepareEditorShare({
      context: shareContext, saved: () => savedLineup.value,
      dirty: () => hasUnpublishedChanges.value, detail: getSharedLineup,
    })
    if (prepared.lineup) { await share.open(prepared.lineup); return }
    if (prepared.result === 'needs-publish') uni.showToast({ title: '请先发布当前战术再分享', icon: 'none' })
    else if (prepared.result === 'incomplete') uni.showToast({ title: '排满 11 人并发布后再分享', icon: 'none' })
  } catch (error) { uni.showToast({ title: extractApiErrorMessage(error, '分享准备失败，请重试'), icon: 'none' }) }
  finally { sharePreparing.value = false }
}

async function resolveDefaultTeamId(list: Array<{ team_id: number }>): Promise<number> {
  if (requestedTeamId && list.some((t) => t.team_id === requestedTeamId)) return requestedTeamId
  if (session.hasAccessToken.value) {
    try {
      const profile = await getSupportProfile()
      const favorite = profile.favorite_team?.team_id
      if (favorite && list.some((t) => t.team_id === favorite)) return favorite
    } catch {
      // 取不到主队就用列表第一支队，不阻断页面
    }
  }
  return list[0]?.team_id ?? 0
}

async function start(): Promise<void> {
  loading.value = true
  try {
    if (session.hasAccessToken.value) await session.ensureSessionReady()
    const list = await listSupportTeams()
    teams.value = list.map((t) => ({ team_id: t.team_id, team_name: t.team_name, avatar_url: t.avatar_storage_url }))
    const initial = await resolveDefaultTeamId(teams.value)
    if (initial) {
      await load(initial, { restoreGuestDraft: resumeAfterLogin !== null })
      await resumeRequestedAction()
    }
  } catch (error) {
    errorMessage.value = error instanceof Error && error.message ? error.message : '战术板加载失败'
    loading.value = false
  }
}

async function reload(): Promise<void> {
  errorMessage.value = ''
  if (teamId.value) await load(teamId.value)
  else await start()
}

let resumeAfterLogin: 'draft' | 'publish' | null = null

onLoad((query) => {
  const parsed = parseLineupPageQuery(query as Record<string, unknown> | undefined)
  requestedTeamId = parsed.teamId
  resumeAfterLogin = parsed.resumeAction
})

onShow(() => {
  // 只在首次进入时加载。登录页回跳会新建本页（switchTab 会关闭非 Tab 页），
  // 登录回跳的一次性意图只恢复私密草稿；发布仍须再次确认。
  if (started) return
  started = true
  void start()
})

async function resumeRequestedAction(): Promise<void> {
  const action = resumeAfterLogin
  resumeAfterLogin = null
  if (!action || !session.hasAccessToken.value || loading.value || !squad.value) return
  if (action === 'draft') await handleDraft()
  else requestConfirmation('publish')
}

onBeforeUnmount(dispose)
</script>

<style scoped>
.page-root {
  position: relative;
  min-height: 100vh;
  background: var(--fi-color-page-soft);
}

.page-bg {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100vh;
  background: var(--fi-color-page-soft);
  pointer-events: none;
}

.page {
  position: relative;
  z-index: 1;
  height: 100vh;
  padding: calc(var(--fi-brand-nav-height) + var(--fi-space-12)) var(--fi-space-24) calc(env(safe-area-inset-bottom) + var(--fi-space-12));
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
}

.page-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  gap: var(--fi-space-12);
}

.page-content > view:not(.pitch-slot),
.page-content > text {
  flex-shrink: 0;
}

.pitch-slot {
  position: relative;
  flex: 1;
  min-height: calc(var(--fi-space-64) * 6);
}

.team-picker {
  flex-shrink: 0;
  margin-bottom: var(--fi-space-12);
}

.state-card {
  padding: var(--fi-space-28);
  border: var(--fi-border-card);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-surface-frosted);
  box-shadow: var(--fi-shadow-soft);
  text-align: center;
}

.state-card--error {
  color: var(--fi-color-primary-deep);
}

.ghost-action {
  margin-top: var(--fi-space-16);
  padding: var(--fi-space-12) var(--fi-space-22);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.ghost-action::after {
  border: 0;
}

.notice {
  padding: var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: rgba(var(--fi-primitive-red-rgb), 0.06);
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.status-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--fi-space-4);
}

.status-row__heading {
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-12);
  min-width: 0;
}

.community-link {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-8) 0 var(--fi-space-8) var(--fi-space-8);
  background: transparent;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.community-link::after {
  border: 0;
}

.status-row__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.status-row__state {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}

.status-row__state--dirty {
  color: var(--fi-color-primary);
}

.status-row__state--published,
.status-row__state--draft {
  color: var(--fi-color-seat-desired);
}

.status-row__state--error {
  color: var(--fi-color-primary-deep);
}

.hint-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: 0 var(--fi-space-4);
}

.hint {
  min-width: 0;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.hint-row .hint {
  flex: 1;
}

.hint-link {
  flex-shrink: 0;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

</style>

<style scoped>
.share-canvas { position: fixed; left: -3000rpx; top: 0; pointer-events: none; }
</style>
