<template>
  <view class="topic-card">
    <view class="topic-card__top">
      <text class="topic-card__category">蓉城动态</text>
      <text class="topic-card__number">{{ indexLabel }}</text>
    </view>
    <view class="topic-card__reading" role="button" :aria-label="`阅读${item.title}`" hover-class="topic-card__reading--pressed" @tap="emit('read', item.id)">
      <text class="topic-card__title">{{ item.title }}</text>
      <text class="topic-card__summary">{{ item.summary }}</text>
    </view>
    <text class="topic-card__source">{{ item.sourceLabel }}</text>
    <view class="topic-card__footer">
      <button class="topic-card__read" @tap="emit('read', item.id)">
        <view class="topic-card__read-icon" aria-hidden="true">
          <view class="topic-card__read-line" />
          <view class="topic-card__read-line" />
          <view class="topic-card__read-line topic-card__read-line--short" />
        </view>
        <text>阅读全文</text>
      </button>
      <FiTopicAudioEntry :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" @play="emit('play', item.id)" />
    </view>
  </view>
</template>

<script setup lang="ts">
import FiTopicAudioEntry from '../../../components/topics/FiTopicAudioEntry.vue'
import type { DailyTopicItem } from '../../../types/dailyTopic'
defineProps<{ item: DailyTopicItem; indexLabel: string; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const emit = defineEmits<{ (e: 'read', id: string): void; (e: 'play', id: string): void }>()
</script>

<style scoped>
.topic-card { box-sizing: border-box; height: var(--fi-space-topic-card-height); padding: var(--fi-space-28); border-radius: var(--fi-radius-lg); background: var(--fi-color-bg); border: var(--fi-border-card); box-shadow: var(--fi-shadow-soft); display: flex; flex-direction: column; }
.topic-card__top { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.topic-card__category { padding: var(--fi-space-4) var(--fi-space-12); border-radius: var(--fi-radius-round); background: var(--fi-color-topic-audio-surface); color: var(--fi-color-primary); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); }
.topic-card__number { color: var(--fi-color-text-caption); font-size: var(--fi-font-22); }
.topic-card__reading { flex: 1; min-height: 0; overflow: hidden; margin-top: var(--fi-space-18); }
.topic-card__reading--pressed { opacity: 0.75; }
.topic-card__title { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; color: var(--fi-color-text-primary); font-size: var(--fi-font-32); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-bold); }
.topic-card__summary { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-top: var(--fi-space-20); color: var(--fi-color-text-secondary); font-size: var(--fi-font-28); line-height: var(--fi-leading-relaxed); }
.topic-card__source { display: block; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; margin-top: var(--fi-space-16); color: var(--fi-color-text-caption); font-size: var(--fi-font-20); line-height: var(--fi-leading-normal); }
.topic-card__footer { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); margin-top: var(--fi-space-18); padding-top: var(--fi-space-18); border-top: var(--fi-border-divider); }
.topic-card__read { display: inline-flex; align-items: center; gap: var(--fi-space-8); margin: 0; padding: var(--fi-space-12) 0; background: transparent; color: var(--fi-color-text-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-bold); white-space: nowrap; }
.topic-card__read::after { border: 0; }
.topic-card__read-icon { box-sizing: border-box; display: flex; flex-direction: column; justify-content: center; flex-shrink: 0; gap: var(--fi-space-4); width: var(--fi-space-24); height: var(--fi-space-28); padding: var(--fi-space-4); border: var(--fi-border-width) solid currentColor; border-radius: var(--fi-space-4); color: var(--fi-color-primary); }
.topic-card__read-line { flex-shrink: 0; height: var(--fi-border-width); background: currentColor; border-radius: var(--fi-radius-round); }
.topic-card__read-line--short { width: 75%; }
</style>
