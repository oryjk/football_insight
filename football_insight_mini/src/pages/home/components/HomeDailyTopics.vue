<template>
  <view class="daily-topics">
    <view class="daily-topics__heading">
      <view class="daily-topics__heading-copy">
        <text class="daily-topics__eyebrow">CHENGDU · DAILY TOPICS</text>
        <view class="daily-topics__title-row">
          <text class="daily-topics__title">今日话题</text>
          <text class="daily-topics__date">{{ edition.dateLabel }}</text>
        </view>
      </view>
    </view>
    <text class="daily-topics__intro">成都蓉城新闻 · AI 整理 · 每篇四川话播报</text>
    <view v-if="loading && !edition.items.length" class="daily-topics__state">
      <text class="daily-topics__state-title">正在加载最新话题</text>
      <text class="daily-topics__state-copy">成都蓉城相关新闻与来源</text>
    </view>
    <view v-else-if="!edition.items.length" class="daily-topics__state">
      <text class="daily-topics__state-title">{{ errorMessage === '后端接口待接入' ? errorMessage : errorMessage ? '话题暂时未能加载' : '暂无已发布话题' }}</text>
      <text class="daily-topics__state-copy">{{ errorMessage === '后端接口待接入' ? '新闻整理与语音发布后，会在这里呈现。' : errorMessage || '有新的新闻整理发布后，会在这里呈现。' }}</text>
      <button v-if="errorMessage" class="daily-topics__retry" @tap="emit('retry')">重新加载</button>
    </view>
    <template v-else>
      <view v-if="errorMessage" class="daily-topics__refresh-error">
        <text>{{ errorMessage }} · 当前展示上次发布内容</text>
        <button class="daily-topics__retry" @tap="emit('retry')">重试</button>
      </view>
      <swiper class="daily-topics__swiper" :current="activeIndex" :circular="false" :autoplay="false" @change="onSlideChange">
        <swiper-item v-for="(item, index) in edition.items" :key="item.id">
          <view class="daily-topics__slide">
            <HomeDailyTopicCard :item="item" :index-label="`${String(index + 1).padStart(2, '0')} / ${String(edition.items.length).padStart(2, '0')}`" :audio-ready="availableAudioIds.includes(item.id)" :audio-playing="activeAudioId === item.id && audioStatus === 'playing'" :audio-loading="activeAudioId === item.id && audioStatus === 'loading'" @read="emit('read', $event)" @play="emit('play', $event)" />
          </view>
        </swiper-item>
      </swiper>
      <view v-if="edition.items.length > 1" class="daily-topics__navigation">
        <text class="daily-topics__hint">{{ activeIndex === edition.items.length - 1 ? '已到最后一篇' : '左滑查看下一篇' }}</text>
        <view class="daily-topics__dots">
          <button v-for="(item, index) in edition.items" :key="item.id" class="daily-topics__dot-button" :aria-label="`查看第 ${index + 1} 篇：${item.title}`" :aria-pressed="index === activeIndex" hover-class="daily-topics__dot-button--pressed" @tap="activeIndex = index">
            <view class="daily-topics__dot" :class="{ 'daily-topics__dot--active': index === activeIndex }" />
          </button>
        </view>
      </view>
      <text class="daily-topics__updated">{{ edition.updatedLabel }}</text>
    </template>
  </view>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import HomeDailyTopicCard from './HomeDailyTopicCard.vue'
import type { DailyTopicEdition } from '../../../types/dailyTopic'
const props = defineProps<{ edition: DailyTopicEdition; loading: boolean; errorMessage: string; availableAudioIds: string[]; activeAudioId: string | null; audioStatus: string }>()
const emit = defineEmits<{ (e: 'read', id: string): void; (e: 'play', id: string): void; (e: 'retry'): void }>()
const activeIndex = ref(0)
function onSlideChange(event: { detail: { current: number } }) { activeIndex.value = event.detail.current }
watch(() => props.edition.updatedAt, () => { activeIndex.value = 0 })
watch(() => props.edition.items.length, length => { if (activeIndex.value >= length) activeIndex.value = 0 })
</script>

<style scoped>
.daily-topics { min-width: 0; padding: var(--fi-space-24) var(--fi-space-20) var(--fi-space-20); border-radius: var(--fi-radius-xl); background: var(--fi-color-page-soft); }
.daily-topics__heading { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.daily-topics__refresh-error { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); margin-bottom: var(--fi-space-16); }
.daily-topics__heading-copy { min-width: 0; }
.daily-topics__eyebrow { display: block; color: var(--fi-color-primary); font-size: var(--fi-font-20); font-weight: var(--fi-weight-bold); letter-spacing: var(--fi-space-4); }
.daily-topics__title-row { display: flex; align-items: baseline; flex-wrap: wrap; gap: var(--fi-space-16); margin-top: var(--fi-space-10); }
.daily-topics__title { color: var(--fi-color-text-primary); font-size: var(--fi-font-44); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-snug); }
.daily-topics__date { color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.daily-topics__intro { display: block; margin-top: var(--fi-space-14); margin-bottom: var(--fi-space-24); color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.daily-topics__swiper { height: calc(var(--fi-space-topic-card-height) + var(--fi-space-12)); }
.daily-topics__slide { box-sizing: border-box; height: 100%; padding-bottom: var(--fi-space-12); display: flex; flex-direction: column; align-items: stretch; }
.daily-topics__navigation { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.daily-topics__hint { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.daily-topics__updated { display: block; margin-top: var(--fi-space-6); color: var(--fi-color-text-caption); font-size: var(--fi-font-20); line-height: var(--fi-leading-normal); }
.daily-topics__dots { display: flex; align-items: center; flex-shrink: 0; }
.daily-topics__dot-button { width: var(--fi-space-44); height: var(--fi-space-44); display: flex; align-items: center; justify-content: center; margin: 0; padding: 0; border-radius: var(--fi-radius-round); background: transparent; line-height: var(--fi-leading-normal); }
.daily-topics__dot-button::after { border: 0; }
.daily-topics__dot-button--pressed { background: var(--fi-color-topic-audio-surface); }
.daily-topics__dot { width: var(--fi-space-8); height: var(--fi-space-8); border-radius: var(--fi-radius-round); background: var(--fi-color-divider); }
.daily-topics__dot--active { width: var(--fi-space-20); background: var(--fi-color-primary); }
.daily-topics__state { padding: var(--fi-space-40) var(--fi-space-28); border-radius: var(--fi-radius-lg); background: var(--fi-color-bg); border: var(--fi-border-card); }
.daily-topics__state-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-bold); }
.daily-topics__state-copy { display: block; margin-top: var(--fi-space-16); color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-relaxed); }
.daily-topics__retry { display: inline-flex; margin: var(--fi-space-24) 0 0; padding: var(--fi-space-12) var(--fi-space-24); color: var(--fi-color-primary); background: var(--fi-color-topic-audio-surface); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); border-radius: var(--fi-radius-round); }
.daily-topics__retry::after { border: 0; }
</style>
