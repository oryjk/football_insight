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
      <FiLoading
        v-if="loading && !teams.length"
        title="战术板加载中"
        caption="正在准备球队名单和你的阵容。"
      />

      <view v-else-if="errorMessage && !squad" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
        <button class="ghost-action" @tap="reload">重试</button>
      </view>

      <view v-else class="page-content">
        <LineupTeamPicker v-if="teams.length" :teams="teams" :model-value="teamId" @update:model-value="switchTeam" />

        <view v-if="notice" class="notice"><text>{{ notice }}</text></view>

        <view class="status-row">
          <text class="status-row__title">我心中的最佳阵容</text>
          <text class="status-row__state" :class="`status-row__state--${saveState}`">{{ saveStateText }}</text>
        </view>

        <LineupFormationBar :model-value="editor.formation.value" @update:model-value="changeFormation" />

        <LineupPitch
          :players="pitchPlayers"
          :selected-slot="editor.selectedSlot.value"
          @tap="openSheet"
          @drag="editor.applyDrag"
        />

        <text class="hint">点位置选人，按住圆点可以拖动站位；前队员也能上场。已排 {{ editor.filled.value }}/11</text>

        <view class="actions">
          <button class="actions__ghost" hover-class="actions__ghost--pressed" @tap="editor.reset()">重置站位</button>
          <button
            class="actions__primary"
            hover-class="actions__primary--pressed"
            :disabled="saving"
            @tap="handleSave"
          >{{ saving ? '保存中…' : '保存' }}</button>
          <button class="actions__ghost actions__ghost--disabled" @tap="handleShareSoon">分享</button>
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
    <!-- #endif -->
  </view>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onLoad, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { getSupportProfile, listSupportTeams } from '../../api/support'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { buildLineupPageUrl, parseLineupPageQuery } from '../../utils/lineupLinks'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { buildPitchPlayers, buildSheetGroups, positionIndex } from './lineupViewModel'
import { useLineupPage } from './useLineupPage'
import LineupFormationBar from './components/LineupFormationBar.vue'
import LineupPitch from './components/LineupPitch.vue'
import LineupPlayerSheet from './components/LineupPlayerSheet.vue'
import LineupTeamPicker from './components/LineupTeamPicker.vue'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()

const { teamId, squad, loading, errorMessage, notice, saving, saveState, editor, load, save, dispose } = useLineupPage()

const teams = ref<Array<{ team_id: number; team_name: string; avatar_url: string | null }>>([])
const sheetVisible = ref(false)
let requestedTeamId = 0
let started = false

// 弹层打开时锁页面滚动（沿用换座页的 page-meta 约定）。
const pageLockStyle = computed(() => (sheetVisible.value ? 'overflow: hidden;' : ''))

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
    case 'saved':
      return '已保存'
    case 'dirty':
      return '未保存'
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
    title: '还没保存',
    content: '改动已存为草稿，下次进来会恢复。确定先离开吗？',
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

async function switchTeam(nextTeamId: number): Promise<void> {
  if (nextTeamId === teamId.value) return
  if (!editor.dirty.value) {
    await load(nextTeamId)
    return
  }
  uni.showModal({
    title: '切换球队',
    content: '当前改动已存为草稿，切换后可以回来继续。',
    confirmText: '切换',
    cancelText: '取消',
    success: (res) => {
      if (res.confirm) void load(nextTeamId)
    },
  })
}

async function handleSave(): Promise<void> {
  const result = await save()
  if (result === 'saved') {
    uni.showToast({ title: '已保存', icon: 'success' })
    return
  }
  if (result === 'need-login') {
    rememberPostLoginRedirect({ type: 'navigateTo', url: buildLineupPageUrl(teamId.value, { autosave: true }) })
    uni.showToast({ title: '登录后即可保存', icon: 'none' })
    uni.switchTab({ url: '/pages/user/index' })
    return
  }
  uni.showToast({ title: '保存失败，已存为草稿，稍后重试', icon: 'none' })
}

function handleShareSoon(): void {
  uni.showToast({ title: '分享功能即将上线', icon: 'none' })
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
    const list = await listSupportTeams()
    teams.value = list.map((t) => ({ team_id: t.team_id, team_name: t.team_name, avatar_url: t.avatar_storage_url }))
    const initial = await resolveDefaultTeamId(teams.value)
    if (initial) {
      await load(initial)
      await autosaveRestoredDraft()
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

let autosaveAfterLoad = false

onLoad((query) => {
  const parsed = parseLineupPageQuery(query as Record<string, unknown> | undefined)
  requestedTeamId = parsed.teamId
  autosaveAfterLoad = parsed.autosave
})

onShow(() => {
  // 只在首次进入时加载。登录页回跳会新建本页（switchTab 会关闭非 Tab 页），
  // 此时 URL 带一次性的 autosave 标记，草稿恢复后自动保存一次，不依赖页面实例存活。
  if (started) return
  started = true
  void start()
})

async function autosaveRestoredDraft(): Promise<void> {
  if (!autosaveAfterLoad) return
  autosaveAfterLoad = false // 一次性：之后的 onShow、回到前台都不会再自动保存
  if (!session.hasAccessToken.value || !editor.dirty.value) return
  const result = await save()
  if (result === 'saved') uni.showToast({ title: '已保存', icon: 'success' })
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
  min-height: 100vh;
  padding: calc(var(--fi-brand-nav-height) + var(--fi-space-24)) var(--fi-space-24) var(--fi-space-64);
  box-sizing: border-box;
}

.page-content {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
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
  align-items: baseline;
  justify-content: space-between;
  padding: 0 var(--fi-space-4);
}

.status-row__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
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

.status-row__state--saved {
  color: var(--fi-color-seat-desired);
}

.status-row__state--error {
  color: var(--fi-color-primary-deep);
}

.hint {
  padding: 0 var(--fi-space-4);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.actions {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.actions__ghost,
.actions__primary {
  margin: 0;
  padding: var(--fi-space-20) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.actions__ghost::after,
.actions__primary::after {
  border: 0;
}

.actions__ghost {
  flex-shrink: 0;
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
}

.actions__ghost--disabled {
  opacity: 0.5;
}

.actions__primary {
  flex: 1;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  box-shadow: var(--fi-shadow-soft);
}

.actions__primary[disabled] {
  background: var(--fi-color-text-caption);
  box-shadow: none;
}

.actions__ghost--pressed,
.actions__primary--pressed {
  transform: scale(0.97);
}

@media (prefers-reduced-motion: reduce) {
  .actions__ghost,
  .actions__primary {
    transition: none;
  }
}
</style>
