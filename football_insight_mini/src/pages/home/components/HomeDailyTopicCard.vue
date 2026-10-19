<template>
  <view class="topic-card">
    <FiTopicCrest :item="item" />
    <view class="topic-card__top">
      <text class="topic-card__category">{{ homeTopicCategory(item) }}</text>
      <text class="topic-card__number">{{ indexLabel }}</text>
    </view>
    <view class="topic-card__reading" role="button" :aria-label="`阅读${item.title}`" hover-class="topic-card__reading--pressed" @tap="emit('read', item.id)">
      <text class="topic-card__title">{{ item.title }}</text>
      <text class="topic-card__summary">{{ item.summary }}</text>
    </view>
    <!-- 整张卡片的正文区已可点开阅读，底部不再单独放「查看」；只留热度/阅读时长与收听。 -->
    <view class="topic-card__footer">
      <!-- 热度直接写在卡片里而不是嵌 FiTopicHeat：小程序组件宿主节点按基线排，和旁边文字对不齐。 -->
      <view class="topic-card__meta">
        <image class="topic-card__heat-icon" :src="fireIcon" mode="aspectFit" aria-hidden="true" />
        <text class="topic-card__heat">热度 {{ formatTopicHeat(heat) }}</text>
        <text class="topic-card__minutes">约 {{ item.readMinutes }} 分钟读完</text>
      </view>
      <view class="topic-card__audio"><FiTopicAudioEntry variant="hero" label="听" :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" @play="emit('play', item.id)" /></view>
    </view>
  </view>
</template>

<script setup lang="ts">
import FiTopicCrest from '../../../components/topics/FiTopicCrest.vue'
import fireIcon from '../../../static/home/topic-heat-flame.svg'
import FiTopicAudioEntry from '../../../components/topics/FiTopicAudioEntry.vue'
import type { DailyTopicItem } from '../../../types/dailyTopic'
import { homeTopicCategory } from '../dailyTopics'
import { formatTopicHeat } from '../../../utils/dailyTopics'
defineProps<{ item: DailyTopicItem; heat: number; indexLabel: string; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const emit = defineEmits<{ (e: 'read', id: string): void; (e: 'play', id: string): void }>()
</script>

<style scoped>
.topic-card { position: relative; box-sizing: border-box; height: var(--fi-space-topic-card-height); padding: var(--fi-space-24); border-radius: var(--fi-component-topic-hero-radius); background: var(--fi-component-topic-hero-bg); display: flex; flex-direction: column; overflow: hidden; }
.topic-card__top, .topic-card__reading, .topic-card__footer { position: relative; z-index: 1; }
.topic-card__top { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.topic-card__category { padding: var(--fi-space-8) var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-primary); color: var(--fi-color-topic-hero-text); font-size: var(--fi-font-24); line-height: var(--fi-leading-snug); font-weight: var(--fi-weight-bold); }
.topic-card__number { color: var(--fi-color-topic-hero-caption); font-size: var(--fi-font-24); }
.topic-card__reading { flex: 1; min-height: 0; overflow: hidden; margin-top: var(--fi-space-16); }
.topic-card__reading--pressed { opacity: 0.75; }
.topic-card__title { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; color: var(--fi-component-topic-hero-title); font-size: var(--fi-font-36); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-extrabold); }
.topic-card__summary { display: -webkit-box; -webkit-line-clamp: 1; -webkit-box-orient: vertical; overflow: hidden; margin-top: var(--fi-space-12); color: var(--fi-component-topic-hero-copy); font-size: var(--fi-font-26); line-height: var(--fi-leading-normal); }
.topic-card__footer { display: flex; align-items: center; gap: var(--fi-space-16); margin-top: var(--fi-space-16); }
/* 图标、热度、阅读时长同字号同行高，在一行 flex 里居中对齐。 */
.topic-card__meta { flex: 1; min-width: 0; display: flex; align-items: center; color: var(--fi-color-topic-hero-muted); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); white-space: nowrap; overflow: hidden; }
.topic-card__heat-icon { flex-shrink: 0; width: var(--fi-space-24); height: var(--fi-space-24); margin-right: var(--fi-space-6); }
.topic-card__heat { flex-shrink: 0; }
.topic-card__minutes { min-width: 0; margin-left: var(--fi-space-16); overflow: hidden; text-overflow: ellipsis; }
.topic-card__audio { flex-shrink: 0; }
</style>
