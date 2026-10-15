<template>
  <view class="topic-card">
    <image
      v-if="isHomeRongchengTopic(item)"
      class="topic-card__crest"
      :class="{ 'topic-card__crest--loaded': crestLoaded }"
      :src="rongchengCrest"
      mode="aspectFit"
      aria-hidden="true"
      @load="crestLoaded = true"
      @error="crestLoaded = false"
    />
    <view class="topic-card__top">
      <text class="topic-card__category">{{ homeTopicCategory(item) }}</text>
      <text class="topic-card__number">{{ indexLabel }}</text>
    </view>
    <view class="topic-card__reading" role="button" :aria-label="`阅读${item.title}`" hover-class="topic-card__reading--pressed" @tap="emit('read', item.id)">
      <text class="topic-card__title">{{ item.title }}</text>
      <text class="topic-card__summary">{{ item.summary }}</text>
    </view>
    <view class="topic-card__footer">
      <view class="topic-card__heat"><FiTopicHeat :value="heat" variant="hero" /></view>
      <view class="topic-card__read"><FiTopicReadEntry variant="hero" @read="emit('read', item.id)" /></view>
      <view class="topic-card__audio"><FiTopicAudioEntry variant="hero" label="听" :ready="audioReady" :playing="audioPlaying" :loading="audioLoading" @play="emit('play', item.id)" /></view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { CHENGDU_RONGCHENG_CREST_IMAGE_URL as rongchengCrest } from '../../../config/assets'
import FiTopicHeat from '../../../components/topics/FiTopicHeat.vue'
import FiTopicReadEntry from '../../../components/topics/FiTopicReadEntry.vue'
import FiTopicAudioEntry from '../../../components/topics/FiTopicAudioEntry.vue'
import type { DailyTopicItem } from '../../../types/dailyTopic'
import { homeTopicCategory, isHomeRongchengTopic } from '../dailyTopics'
defineProps<{ item: DailyTopicItem; heat: number; indexLabel: string; audioReady: boolean; audioPlaying: boolean; audioLoading: boolean }>()
const crestLoaded = ref(false)
const emit = defineEmits<{ (e: 'read', id: string): void; (e: 'play', id: string): void }>()
</script>

<style scoped>
.topic-card { position: relative; box-sizing: border-box; height: var(--fi-space-topic-card-height); padding: var(--fi-space-32); border-radius: var(--fi-component-topic-hero-radius); background: var(--fi-component-topic-hero-bg); display: flex; flex-direction: column; overflow: hidden; }
.topic-card__crest { position: absolute; z-index: 0; width: var(--fi-component-topic-hero-crest-size); height: var(--fi-component-topic-hero-crest-size); right: calc(-1 * var(--fi-space-32)); top: 50%; transform: translateY(-50%); opacity: 0; pointer-events: none; }
.topic-card__crest--loaded { opacity: var(--fi-component-topic-hero-crest-opacity); }
.topic-card__top, .topic-card__reading, .topic-card__footer { position: relative; z-index: 1; }
.topic-card__top { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.topic-card__category { padding: var(--fi-space-8) var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-primary); color: var(--fi-color-topic-hero-text); font-size: var(--fi-font-24); line-height: var(--fi-leading-snug); font-weight: var(--fi-weight-bold); }
.topic-card__number { color: var(--fi-color-topic-hero-caption); font-size: var(--fi-font-24); }
.topic-card__reading { flex: 1; min-height: 0; overflow: hidden; margin-top: var(--fi-space-24); }
.topic-card__reading--pressed { opacity: 0.75; }
.topic-card__title { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; color: var(--fi-component-topic-hero-title); font-size: var(--fi-font-36); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-extrabold); }
.topic-card__summary { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; margin-top: var(--fi-space-20); color: var(--fi-component-topic-hero-copy); font-size: var(--fi-font-26); line-height: var(--fi-leading-loose); }
.topic-card__footer { display: flex; align-items: center; gap: var(--fi-space-16); margin-top: var(--fi-space-20); }
.topic-card__heat { flex: 1; min-width: 0; display: flex; flex-direction: column; align-items: stretch; }
.topic-card__read, .topic-card__audio { flex-shrink: 0; }
</style>
