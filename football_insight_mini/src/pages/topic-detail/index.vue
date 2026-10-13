<template>
  <view class="topic-detail">
    <view v-if="loading" class="topic-detail__state">正在加载话题…</view>
    <view v-else-if="errorMessage" class="topic-detail__state"><text>{{ errorMessage }}</text><button @tap="load">重新加载</button></view>
    <template v-else-if="item">
      <text class="topic-detail__eyebrow">成都蓉城</text>
      <text class="topic-detail__title">{{ item.title }}</text>
      <text class="topic-detail__meta">{{ item.publishedLabel }} · 约 {{ item.readMinutes }} 分钟阅读</text>
      <view class="topic-detail__audio"><FiTopicAudioEntry :ready="audio.availableIds.value.includes(item.id)" :playing="audio.status.value === 'playing'" :loading="audio.status.value === 'loading'" @play="audio.toggle(item.id)" /></view>
      <FiTopicAudioPlayer v-if="audio.activeId.value" :title="item.title" :status="audio.status.value" :current-time="audio.currentTime.value" :duration="audio.duration.value" :error-message="audio.errorMessage.value" @toggle="audio.toggle(item.id)" @seek="audio.seek" @close="audio.stop" />
      <text v-for="(paragraph, index) in item.paragraphs" :key="index" class="topic-detail__paragraph">{{ paragraph }}</text>
      <view class="topic-detail__sources">
        <text class="topic-detail__sources-heading">新闻来源</text>
        <view v-for="(source, index) in item.sources" :key="source.url" class="topic-detail__source">
          <text class="topic-detail__source-title">{{ index + 1 }}. {{ source.title }}</text>
          <text class="topic-detail__meta">{{ source.publisher }} · {{ source.dateLabel }}</text>
          <button class="topic-detail__copy" @tap="copySource(source.url)">复制原文链接</button>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, onScopeDispose, ref } from 'vue'
import { onLoad } from '@dcloudio/uni-app'
import { getTopicDetail } from '../../api/dailyTopics'
import { useDailyTopicAudio } from '../../composables/useDailyTopicAudio'
import FiTopicAudioEntry from '../../components/topics/FiTopicAudioEntry.vue'
import FiTopicAudioPlayer from '../../components/topics/FiTopicAudioPlayer.vue'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { DAILY_TOPICS_TEAM_ID, editionForTopic, isTopicSourceUrl, mapTopicDetail } from '../../utils/dailyTopics'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
const item = ref<DailyTopicItem | null>(null)
const loading = ref(false)
const errorMessage = ref('')
const edition = computed(() => editionForTopic(item.value))
const audio = useDailyTopicAudio(edition)
let topicId = ''
let revision = 0
async function load() {
  const current = ++revision
  audio.stop()
  if (!/^[a-zA-Z0-9_-]{1,128}$/.test(topicId)) { errorMessage.value = '话题不存在或已下线'; return }
  loading.value = true
  errorMessage.value = ''
  try {
    const latest = mapTopicDetail(await getTopicDetail(DAILY_TOPICS_TEAM_ID, topicId))
    if (latest.id !== topicId) throw new Error('话题加载失败，请重试')
    if (current === revision) item.value = latest
  } catch (error) {
    if (current === revision) errorMessage.value = error instanceof ApiRequestError && error.statusCode === 404
      ? '话题不存在或已下线' : extractApiErrorMessage(error, '话题加载失败，请稍后重试')
  } finally { if (current === revision) loading.value = false }
}
function copySource(url: string) {
  if (!isTopicSourceUrl(url) || !item.value?.sources.some(source => source.url === url)) return
  uni.setClipboardData({ data: url, success: () => uni.showToast({ title: '原文链接已复制', icon: 'none' }) })
}
onLoad(query => { topicId = typeof query?.id === 'string' ? query.id : ''; void load() })
onScopeDispose(() => { revision += 1 })
</script>

<style scoped>
.topic-detail { padding: var(--fi-space-28) var(--fi-space-28) var(--fi-space-40); background: var(--fi-color-page); }
.topic-detail__eyebrow { display: block; color: var(--fi-color-primary); font-size: var(--fi-font-22); font-weight: var(--fi-weight-bold); }
.topic-detail__title { display: block; margin-top: var(--fi-space-16); color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-normal); }
.topic-detail__meta { display: block; margin-top: var(--fi-space-18); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.topic-detail__audio { margin: var(--fi-space-24) 0; }
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
