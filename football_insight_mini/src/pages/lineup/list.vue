<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav :shareable="false" @back="goBack" />
  <view class="community-page">
    <text class="community-title">大家排的阵容</text>
    <!-- #ifdef H5 -->
    <text class="community-caption">请在微信小程序中查看球队战术板。</text>
    <!-- #endif -->
    <!-- #ifndef H5 -->
    <LineupTeamPicker :teams="teams" :model-value="teamId" @update:model-value="switchTeam" />
    <view class="community-row">
      <button class="community-action" :class="{ 'community-action--active': sort === 'latest' }" @tap="changeSort('latest')">最新发布</button>
      <button class="community-action" :class="{ 'community-action--active': sort === 'likes' }" @tap="changeSort('likes')">点赞榜</button>
    </view>
    <text class="community-caption">{{ total }} 套完整阵容 · {{ sort === 'likes' ? '按点赞数排序，同赞按发布时间排序' : '按首次发布时间排序' }}</text>
    <text v-if="error" class="community-error">{{ error }}</text>
    <FiLoading v-if="loading && !items.length" title="战术板加载中" caption="正在读取球队的球迷阵容。" />
    <FiLineupCommunityCard v-for="(item, index) in items" :key="item.share_code" :item="item" :rank="sort === 'likes' ? index + 1 : undefined" :pending="pendingLikes.includes(item.share_code)" @open="open(item.share_code)" @like="like(item, returnUrl)" @dislike="dislike(item, returnUrl)" />
    <text v-if="!loading && !error && !items.length" class="community-caption">还没有完整阵容，来排出第一套吧。</text>
    <button v-if="error || items.length < total" class="community-action" :disabled="loading" @tap="retry">{{ loading ? '加载中…' : error ? '重试加载' : '加载更多' }}</button>
    <button class="community-action community-action--active" @tap="edit">我也排一套</button>
    <!-- #endif -->
  </view>
</template>
<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { onLoad, onReachBottom, onShow } from '@dcloudio/uni-app'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import FiLoading from '../../components/FiLoading.vue'
import LineupTeamPicker from './components/LineupTeamPicker.vue'
import FiLineupCommunityCard from '../../components/FiLineupCommunityCard.vue'
import { listSupportTeams } from '../../api/support'
import type { LineupSort, SquadResponse } from '../../types/lineup'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { buildLineupListUrl, buildLineupPageUrl, buildLineupViewUrl, parseLineupListQuery } from '../../utils/lineupLinks'
import { createLineupList } from '../../composables/useLineupCommunity'
const { navigationPageStyle } = useNavigationLayout()
const state = createLineupList()
const { teamId, sort, items, total, loading, error, pendingLikes, more, like, dislike } = state
const teams = ref<Array<SquadResponse['team']>>([])
const returnUrl = computed(() => buildLineupListUrl(teamId.value, sort.value))
let requested = 0, requestedSort: LineupSort = 'latest', started = false, disposed = false
function goBack(): void { uni.navigateBack({ fail: () => uni.switchTab({ url: '/pages/insights/index' }) }) }
function open(code: string): void { uni.navigateTo({ url: buildLineupViewUrl(code) }) }
function edit(): void { uni.navigateTo({ url: buildLineupPageUrl(teamId.value) }) }
function switchTeam(id: number): void { if (id !== teamId.value) void state.load(id) }
function changeSort(value: LineupSort): void { if (value !== sort.value) void state.load(teamId.value, value) }
function retry(): void {
  if (!teamId.value || !teams.value.length) void start()
  else void more()
}
async function start(): Promise<void> {
  if (disposed) return
  error.value = ''
  loading.value = true
  try {
    const list = await listSupportTeams()
    if (disposed) return
    teams.value = list.map(team => ({ team_id: team.team_id, team_name: team.team_name, avatar_url: team.avatar_storage_url }))
    const id = teams.value.some(team => team.team_id === requested) ? requested : teams.value[0]?.team_id
    loading.value = false
    if (id) await state.load(id, requestedSort)
  } catch { if (!disposed) { error.value = '球队加载失败，请重试'; loading.value = false } }
}
onLoad(query => {
  const parsed = parseLineupListQuery(query as Record<string, unknown>)
  requested = parsed.teamId; requestedSort = parsed.sort
})
onShow(() => {
  // Re-read after detail returns so ranks and viewer-specific likes are authoritative.
  if (!started) { started = true; void start() }
  else if (teamId.value) void state.load(teamId.value)
})
onReachBottom(() => void more())
onBeforeUnmount(() => { disposed = true; state.dispose() })
</script>
<style scoped src="./community.css"></style>
