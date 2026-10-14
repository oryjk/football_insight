<template>
  <page-meta :page-style="navigationPageStyle" />
  <FiSecondaryNav :share-disabled="!shareArticle" @back="goBack" @share="copyArticleShareLink" />
  <view class="topic-page-offset">
  <view class="topic-detail">
    <view v-if="loading" class="topic-detail__state">正在加载话题…</view>
    <view v-else-if="errorMessage" class="topic-detail__state"><text>{{ errorMessage }}</text><button @tap="load">重新加载</button></view>
    <template v-else-if="item">
      <FiTopicTeamTags v-if="timeline.tags.value.length" :tags="timeline.tags.value" :selected-team-id="timeline.selectedTeamId.value" @select="selectTimelineTeam" />
      <text class="topic-detail__title">{{ item.title }}</text>
      <text class="topic-detail__meta">{{ item.publishedLabel }} · 约 {{ item.readMinutes }} 分钟阅读</text>
      <view class="topic-detail__heat"><FiTopicHeat :value="topicHeatValue(item.id, item.heat)" /></view>
      <view class="topic-detail__audio"><FiTopicAudioEntry :ready="audio.availableIds.value.includes(item.id)" :playing="audio.status.value === 'playing'" :loading="audio.status.value === 'loading'" @play="audio.toggle(item.id)" /></view>
      <FiTopicAudioPlayer v-if="audio.activeId.value" :title="item.title" :status="audio.status.value" :current-time="audio.currentTime.value" :duration="audio.duration.value" :error-message="audio.errorMessage.value" @toggle="audio.toggle(item.id)" @seek="audio.seek" @close="audio.stop" />
      <view v-if="item.images?.length" class="topic-detail__gallery">
        <TopicImageGallery :key="item.id" :images="item.images" @preview="previewPhoto" />
      </view>
      <text v-for="(paragraph, index) in item.paragraphs" :key="index" class="topic-detail__paragraph">{{ paragraph }}</text>
      <view class="topic-detail__sources">
        <text class="topic-detail__sources-heading">来源</text>
        <view v-for="(source, index) in item.sources" :key="source.url" class="topic-detail__source">
          <text class="topic-detail__source-title">{{ index + 1 }}. {{ source.title }}</text>
          <text class="topic-detail__meta">{{ source.publisher }} · {{ source.dateLabel }}</text>
          <button class="topic-detail__copy" @tap="copySource(source.url)">复制原文链接</button>
        </view>
      </view>
      <view v-if="timeline.tags.value.length" id="topic-timeline" class="topic-detail__timeline">
        <TopicTimeline :anchor="item" :tags="timeline.tags.value" :selected-team-id="timeline.selectedTeamId.value" :items="timeline.items.value"
          :loading="timeline.loading.value" :error-message="timeline.errorMessage.value" :has-more="!!timeline.nextCursor.value"
          @select="timeline.selectTeam" @read="openTimelineTopic" @more="timeline.loadMore" @retry="timeline.retry" />
      </view>
    </template>
  </view>
  </view>
  <canvas :canvas-id="TOPIC_SHARE_CANVAS_ID" class="topic-share-canvas" :style="shareCanvasStyle" />
</template>

<script setup lang="ts">
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { computed, onScopeDispose, ref } from 'vue'
import { onLoad, onShareAppMessage } from '@dcloudio/uni-app'
import { topicVisitorId } from '../../utils/topicVisitor'
import { topicHeatValue, setTopicHeat, syncTopicHeat } from '../../stores/topicHeat'
import FiTopicHeat from '../../components/topics/FiTopicHeat.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { getTopicDetail, recordTopicVisit } from '../../api/dailyTopics'
import { useDailyTopicAudio } from '../../composables/useDailyTopicAudio'
import FiTopicAudioEntry from '../../components/topics/FiTopicAudioEntry.vue'
import FiTopicAudioPlayer from '../../components/topics/FiTopicAudioPlayer.vue'
import TopicImageGallery from './components/TopicImageGallery.vue'
import TopicTimeline from './components/TopicTimeline.vue'
import FiTopicTeamTags from '../../components/topics/FiTopicTeamTags.vue'
import { useTopicTimeline } from './useTopicTimeline'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { DAILY_TOPICS_TEAM_ID, editionForTopic, isTopicSourceUrl, mapTopicDetail, topicDetailUrl } from '../../utils/dailyTopics'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { buildTopicSharePayload, TOPIC_SHARE_CARD } from './share'
import { TOPIC_SHARE_CANVAS_ID, useTopicShareCard } from './useTopicShareCard'

const { navigationPageStyle } = useNavigationLayout()
const item = ref<DailyTopicItem | null>(null)
const timeline = useTopicTimeline(item)
const loading = ref(false)
const errorMessage = ref('')
const edition = computed(() => editionForTopic(item.value))
const audio = useDailyTopicAudio(edition)
const shareCard = useTopicShareCard()
const shareArticle = computed(() => !loading.value && !errorMessage.value ? item.value : null)
// 分享图按实际像素绘制，与页面布局的 rpx 无关。
const shareCanvasStyle = { width: `${TOPIC_SHARE_CARD.width}px`, height: `${TOPIC_SHARE_CARD.height}px` }
let topicId = ''
let revision = 0
async function load() {
  const current = ++revision
  audio.stop()
  shareCard.reset()
  if (!/^[a-zA-Z0-9_-]{1,128}$/.test(topicId)) { errorMessage.value = '话题不存在或已下线'; return }
  loading.value = true
  errorMessage.value = ''
  try {
    const latest = mapTopicDetail(await getTopicDetail(DAILY_TOPICS_TEAM_ID, topicId))
    if (latest.id !== topicId) throw new Error('话题加载失败，请重试')
    if (current === revision) {
      syncTopicHeat([latest])
      item.value = latest
      void shareCard.prepare(latest)
      const visitorId = topicVisitorId()
      if (visitorId) void recordTopicVisit(DAILY_TOPICS_TEAM_ID, latest.id, visitorId).then(result => {
        setTopicHeat(latest.id, result.heat)
      }).catch(() => { /* Statistics never block reading or playback. */ })
    }
  } catch (error) {
    if (current === revision) errorMessage.value = error instanceof ApiRequestError && error.statusCode === 404
      ? '话题不存在或已下线' : extractApiErrorMessage(error, '话题加载失败，请稍后重试')
  } finally { if (current === revision) loading.value = false }
}
function copySource(url: string) {
  if (!isTopicSourceUrl(url) || !item.value?.sources.some(source => source.url === url)) return
  uni.setClipboardData({ data: url, success: () => uni.showToast({ title: '原文链接已复制', icon: 'none' }) })
}
function selectTimelineTeam(teamId: number) {
  timeline.selectTeam(teamId)
  uni.pageScrollTo({ selector: '#topic-timeline', duration: 200 })
}
function openTimelineTopic(id: string) { audio.stop(); uni.navigateTo({ url: topicDetailUrl(id) }) }
function previewPhoto(url: string, urls: string[]) {
  const allowedUrls = (item.value?.images ?? []).map(image => image.url)
  if (urls.includes(url) && urls.every(imageUrl => allowedUrls.includes(imageUrl))) {
    uni.previewImage({ current: url, urls })
  }
}
onLoad(query => { topicId = typeof query?.id === 'string' ? query.id : ''; void load() })
function goBack() {
  goBackFromSecondaryPage()
}
function copyArticleShareLink() { copyPageShareLink(buildTopicSharePayload(shareArticle.value).path) }
onShareAppMessage(() => {
  const article = shareArticle.value
  const fallback = buildTopicSharePayload(article, shareCard.imageUrl.value)
  if (!article || shareCard.imageUrl.value) return fallback
  return { ...fallback, promise: shareCard.prepare(article).then(imageUrl => buildTopicSharePayload(article, imageUrl)) }
})
onScopeDispose(() => { revision += 1 })
</script>

<style scoped>
.topic-page-offset { padding-top: var(--fi-brand-nav-height); }
.topic-share-canvas { position: fixed; left: -9999rpx; top: 0; pointer-events: none; }
.topic-detail { padding: var(--fi-space-28) var(--fi-space-28) var(--fi-space-40); background: var(--fi-color-page); }
.topic-detail__timeline { margin-top: var(--fi-space-32); }
.topic-detail__title { display: block; margin-top: var(--fi-space-16); color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-normal); }
.topic-detail__meta { display: block; margin-top: var(--fi-space-18); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.topic-detail__heat { margin-top: var(--fi-space-16); }
.topic-detail__audio { margin: var(--fi-space-24) 0; }
.topic-detail__gallery { margin-top: var(--fi-space-24); }
.topic-detail__paragraph { display: block; margin-top: var(--fi-space-28); color: var(--fi-color-text-primary); font-size: var(--fi-font-30); line-height: var(--fi-leading-relaxed); white-space: pre-wrap; }
.topic-detail__sources { margin-top: var(--fi-space-32); padding-top: var(--fi-space-24); border-top: var(--fi-border-divider); }
.topic-detail__sources-heading { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.topic-detail__source { margin-top: var(--fi-space-24); }
.topic-detail__source-title { display: block; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.topic-detail__copy { display: inline-flex; margin: var(--fi-space-12) 0 0; padding: var(--fi-space-12) 0; background: transparent; color: var(--fi-color-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.topic-detail__copy::after, .topic-detail__state button::after { border: 0; }
.topic-detail__state { padding: var(--fi-space-40) 0; text-align: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); }
.topic-detail__state button { margin-top: var(--fi-space-24); font-size: var(--fi-font-26); color: var(--fi-color-primary); background: var(--fi-color-bg); }
</style>
