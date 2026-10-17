<template>
  <view class="daily-topics">
    <view class="daily-topics__heading">
      <view class="daily-topics__heading-copy">
        <view class="daily-topics__title-row">
          <text class="daily-topics__title">今日话题</text>
          <text class="daily-topics__date">{{ edition.dateLabel }}</text>
        </view>
      </view>
      <button v-if="edition.items.length" class="daily-topics__all" @tap="emit('more')">全部话题 ›</button>
    </view>
    <view v-if="loading && !edition.items.length" class="daily-topics__state">
      <text class="daily-topics__state-title">正在加载最新话题</text>
      <text class="daily-topics__state-copy">足球相关新闻与来源</text>
    </view>
    <view v-else-if="!edition.items.length" class="daily-topics__state">
      <text class="daily-topics__state-title">{{ errorMessage ? '话题暂时未能加载' : '暂无已发布话题' }}</text>
      <text class="daily-topics__state-copy">{{ errorMessage || '有新的新闻发布后，会在这里呈现。' }}</text>
      <button v-if="errorMessage" class="daily-topics__retry" @tap="emit('retry')">重新加载</button>
    </view>
    <template v-else>
      <view v-if="errorMessage" class="daily-topics__refresh-error">
        <text>{{ errorMessage }} · 当前展示上次发布内容</text>
        <button class="daily-topics__retry" @tap="emit('retry')">重试</button>
      </view>
      <swiper class="daily-topics__swiper" :class="{ 'daily-topics__swiper--rebound': rebounding }" :style="{ transform: `translateX(${overscroll}px)` }" :current="activeIndex" :circular="false" :autoplay="false" next-margin="56rpx" @change="onSlideChange" @touchstart="onTouchStart" @touchmove="onTouchMove" @touchend="onTouchEnd" @touchcancel="onTouchEnd">
        <swiper-item v-for="(item, index) in edition.items" :key="item.id">
          <view class="daily-topics__slide">
            <HomeDailyTopicCard :item="item" :heat="topicHeatValue(item.id, item.heat)" :index-label="`${String(index + 1).padStart(2, '0')} / ${String(edition.items.length).padStart(2, '0')}`" :audio-ready="availableAudioIds.includes(item.id)" :audio-playing="activeAudioId === item.id && audioStatus === 'playing'" :audio-loading="activeAudioId === item.id && audioStatus === 'loading'" @read="emit('read', $event)" @play="emit('play', $event)" />
          </view>
        </swiper-item>
      </swiper>
    </template>
  </view>
</template>

<script setup lang="ts">
import { topicHeatValue } from '../../../stores/topicHeat'
import { computed, ref, watch } from 'vue'
import HomeDailyTopicCard from './HomeDailyTopicCard.vue'
import { lastSlideOverscroll } from '../dailyTopics'
import type { DailyTopicEdition } from '../../../types/dailyTopic'
const props = defineProps<{ edition: DailyTopicEdition; loading: boolean; errorMessage: string; availableAudioIds: string[]; activeAudioId: string | null; audioStatus: string }>()
const emit = defineEmits<{ (e: 'read', id: string): void; (e: 'play', id: string): void; (e: 'retry'): void; (e: 'more'): void }>()
const activeIndex = ref(0)
const overscroll = ref(0)
const rebounding = ref(false)
const lastIndex = computed(() => props.edition.items.length - 1)
let touchStart: { x: number; y: number } | null = null
function onTouchStart(event: TouchEvent) {
  const touch = event.touches?.[0]
  touchStart = touch ? { x: touch.clientX, y: touch.clientY } : null
  rebounding.value = false
}
function onTouchMove(event: TouchEvent) {
  const touch = event.touches?.[0]
  if (!touch || !touchStart || activeIndex.value !== lastIndex.value) return
  const dx = touch.clientX - touchStart.x
  if (Math.abs(dx) <= Math.abs(touch.clientY - touchStart.y)) return
  overscroll.value = lastSlideOverscroll(dx)
}
function onTouchEnd() {
  touchStart = null
  if (overscroll.value === 0) return
  rebounding.value = true
  overscroll.value = 0
}
function onSlideChange(event: { detail: { current: number } }) { activeIndex.value = event.detail.current }
watch(() => props.edition.updatedAt, () => { activeIndex.value = 0 })
watch(() => props.edition.items.length, length => { if (activeIndex.value >= length) activeIndex.value = 0 })
</script>

<style scoped>
.daily-topics { min-width: 0; padding: var(--fi-space-24) var(--fi-space-20) var(--fi-space-20); border-radius: var(--fi-radius-xl); background: var(--fi-color-page-soft); }
.daily-topics__heading { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); margin-bottom: var(--fi-space-16); }
.daily-topics__refresh-error { color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); margin-bottom: var(--fi-space-16); }
.daily-topics__heading-copy { min-width: 0; }
.daily-topics__title-row { display: flex; align-items: baseline; flex-wrap: wrap; gap: var(--fi-space-16); }
.daily-topics__title { color: var(--fi-color-text-primary); font-size: var(--fi-font-44); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-snug); }
.daily-topics__date { color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); }
.daily-topics__all { margin: 0; padding: var(--fi-space-8) 0 var(--fi-space-8) var(--fi-space-16); background: transparent; color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); flex-shrink: 0; }
.daily-topics__all::after { border: 0; }
.daily-topics__swiper--rebound { transition: transform 300ms cubic-bezier(0.22, 1, 0.36, 1); }
.daily-topics__swiper { height: calc(var(--fi-space-topic-card-height) + var(--fi-space-12)); }
.daily-topics__slide { box-sizing: border-box; height: 100%; padding-bottom: var(--fi-space-12); padding-right: var(--fi-space-card-gap); display: flex; flex-direction: column; align-items: stretch; }
.daily-topics__state { padding: var(--fi-space-40) var(--fi-space-28); border-radius: var(--fi-radius-lg); background: var(--fi-color-bg); border: var(--fi-border-card); }
.daily-topics__state-title { display: block; color: var(--fi-color-text-primary); font-size: var(--fi-font-30); font-weight: var(--fi-weight-bold); }
.daily-topics__state-copy { display: block; margin-top: var(--fi-space-16); color: var(--fi-color-text-secondary); font-size: var(--fi-font-26); line-height: var(--fi-leading-relaxed); }
.daily-topics__retry { display: inline-flex; margin: var(--fi-space-24) 0 0; padding: var(--fi-space-12) var(--fi-space-24); color: var(--fi-color-primary); background: var(--fi-color-topic-audio-surface); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); border-radius: var(--fi-radius-round); }
.daily-topics__retry::after { border: 0; }
</style>
