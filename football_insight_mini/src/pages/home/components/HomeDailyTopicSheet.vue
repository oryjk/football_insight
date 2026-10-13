<template>
  <FiBottomSheet :visible="true" title="今日话题" eyebrow="成都蓉城 · AI 整理" @close="emit('close')">
    <view class="topic-reading">
      <text class="topic-reading__title">{{ item.title }}</text>
      <text class="topic-reading__meta">{{ item.publishedLabel }} · 约 {{ item.readMinutes }} 分钟阅读</text>
      <text v-for="(paragraph, index) in item.paragraphs" :key="index" class="topic-reading__paragraph">{{ paragraph }}</text>
      <view class="topic-reading__sources">
        <text class="topic-reading__sources-heading">新闻来源</text>
        <view v-for="(source, index) in item.sources" :key="source.url" class="topic-reading__source">
          <text class="topic-reading__source-title">{{ index + 1 }}. {{ source.title }}</text>
          <text class="topic-reading__meta">{{ source.publisher }} · {{ source.dateLabel }}</text>
          <button class="topic-reading__copy" @tap="emit('copy-source', source.url)">复制原文链接 ↗</button>
        </view>
      </view>
      <text class="topic-reading__updated">{{ updatedLabel }}</text>
    </view>
    <template #footer>
      <HomeTopicAudioEntry :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" @play="emit('play', item.id)" />
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import HomeTopicAudioEntry from './HomeTopicAudioEntry.vue'
import type { DailyTopicItem } from '../../../types/dailyTopic'
defineProps<{ item: DailyTopicItem; updatedLabel: string; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const emit = defineEmits<{ (e: 'close'): void; (e: 'play', id: string): void; (e: 'copy-source', url: string): void }>()
</script>

<style scoped>
.topic-reading { padding: var(--fi-space-24) 0 var(--fi-space-40); }
.topic-reading__title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-36); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-normal); }
.topic-reading__meta { display: block; margin-top: var(--fi-space-18); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.topic-reading__paragraph { display: block; margin-top: var(--fi-space-28); color: var(--fi-color-text-primary); font-size: var(--fi-font-30); line-height: var(--fi-leading-relaxed); }
.topic-reading__updated { display: block; margin-top: var(--fi-space-32); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.topic-reading__sources { margin-top: var(--fi-space-32); padding-top: var(--fi-space-24); border-top: var(--fi-border-divider); }
.topic-reading__sources-heading { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.topic-reading__source { margin-top: var(--fi-space-24); }
.topic-reading__source-title { display: block; color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.topic-reading__copy { display: inline-flex; margin: var(--fi-space-12) 0 0; padding: var(--fi-space-12) 0; background: transparent; color: var(--fi-color-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.topic-reading__copy::after { border: 0; }
</style>
