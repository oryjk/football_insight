<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav @back="goBack" @share="copyPageShareLink('/pages/topics/index')" />
  <view class="topic-page-offset">
  <view class="topics-page">
    <view class="topics-heading">
      <text class="topics-title">全部话题</text>
      <text class="topics-caption">成都蓉城新闻 · 最新发布在前</text>
    </view>
    <view v-if="items.length" class="topics-list">
      <TopicHistoryCard v-for="item in items" :key="item.id" :item="item" :heat="topicHeatValue(item.id, item.heat)" @read="openTopic" />
    </view>
    <view class="topics-state">
      <text v-if="loading">正在加载话题…</text>
      <template v-else-if="errorMessage"><text>{{ errorMessage }}</text><button @tap="retry">重新加载</button></template>
      <text v-else-if="!items.length">暂无已发布话题</text>
      <button v-else-if="nextCursor" @tap="load(false)">加载更多话题</button>
      <text v-else>已显示全部话题</text>
    </view>
  </view>
  </view>
</template>

<script setup lang="ts">
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { onScopeDispose, ref } from 'vue'
import { onLoad, onPullDownRefresh, onReachBottom, onShareAppMessage } from '@dcloudio/uni-app'
import TopicHistoryCard from './components/TopicHistoryCard.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { topicHeatValue, syncTopicHeat } from '../../stores/topicHeat'
import { getTopicHistory } from '../../api/dailyTopics'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { DAILY_TOPICS_TEAM_ID, mapTopicHistory, mergeTopicHistory, topicDetailUrl } from '../../utils/dailyTopics'
import { extractApiErrorMessage } from '../../utils/apiError'

const { navigationPageStyle } = useNavigationLayout()
const items = ref<DailyTopicItem[]>([])
const nextCursor = ref<string | null>(null)
const loading = ref(false)
const errorMessage = ref('')
let revision = 0
let failedReset = true
async function load(reset: boolean) {
  if (!reset && (loading.value || !nextCursor.value)) return
  const current = ++revision
  const cursor = reset ? null : nextCursor.value
  loading.value = true
  errorMessage.value = ''
  failedReset = reset
  try {
    const page = mapTopicHistory(await getTopicHistory(DAILY_TOPICS_TEAM_ID, cursor))
    if (current !== revision) return
    if (page.nextCursor && page.nextCursor === cursor) throw new Error('话题加载失败，请重试')
    syncTopicHeat(page.items)
    items.value = mergeTopicHistory(reset ? [] : items.value, page.items)
    nextCursor.value = page.nextCursor
  } catch (error) {
    if (current === revision) errorMessage.value = extractApiErrorMessage(error, '话题加载失败，请稍后重试')
  } finally {
    if (current === revision) { loading.value = false; uni.stopPullDownRefresh() }
  }
}
function retry() { void load(failedReset) }
function openTopic(id: string) { uni.navigateTo({ url: topicDetailUrl(id) }) }
onLoad(() => { void load(true) })
onReachBottom(() => { if (!errorMessage.value) void load(false) })
onPullDownRefresh(() => { void load(true) })
function goBack() {
  goBackFromSecondaryPage()
}
onShareAppMessage(() => ({ title: '成都蓉城新闻，看看球迷正在关注的话题', path: '/pages/topics/index' }))
onScopeDispose(() => { revision += 1 })
</script>

<style scoped>
.topics-list { display: flex; flex-direction: column; align-items: stretch; gap: var(--fi-space-card-gap); }
.topic-page-offset { padding-top: var(--fi-brand-nav-height); }
.topics-page { padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-40); background: var(--fi-color-page); min-height: 100%; }
.topics-heading { margin-bottom: var(--fi-space-28); }
.topics-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); }
.topics-caption { display: block; margin-top: var(--fi-space-12); color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.topics-state { padding: var(--fi-space-28) 0; text-align: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.topics-state button { margin-top: var(--fi-space-20); padding: var(--fi-space-16); color: var(--fi-color-primary); background: var(--fi-color-bg); border-radius: var(--fi-radius-lg); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.topics-state button::after { border: 0; }
</style>
