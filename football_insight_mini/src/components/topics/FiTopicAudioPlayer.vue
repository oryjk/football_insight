<template>
  <view class="topic-player">
    <view class="topic-player__heading">
      <text class="topic-player__title">{{ title }}</text>
      <button class="topic-player__close" @tap="emit('close')">关闭</button>
    </view>
    <text v-if="errorMessage" class="topic-player__error">{{ errorMessage }}</text>
    <view class="topic-player__controls">
      <button class="topic-player__toggle" @tap="emit('toggle')">{{ status === 'loading' ? '加载中' : status === 'playing' ? '暂停' : status === 'error' ? '重试' : '播放' }}</button>
      <text class="topic-player__time">{{ formatTopicAudioTime(displayTime) }}</text>
      <slider class="topic-player__slider" :value="Math.floor(displayTime)" :max="Math.max(1, Math.floor(duration))" :disabled="duration <= 0 || status === 'error'" :block-size="12" @changing="onDragging" @change="onSeek" />
      <text class="topic-player__time">{{ formatTopicAudioTime(duration) }}</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { formatTopicAudioTime } from '../../utils/dailyTopics'
const props = defineProps<{ title: string; status: string; currentTime: number; duration: number; errorMessage: string }>()
const emit = defineEmits<{ (e: 'toggle'): void; (e: 'seek', seconds: number): void; (e: 'close'): void }>()
const dragTime = ref<number | null>(null)
const displayTime = computed(() => dragTime.value ?? props.currentTime)
function onDragging(event: { detail: { value: number } }) { dragTime.value = event.detail.value }
function onSeek(event: { detail: { value: number } }) { dragTime.value = null; emit('seek', event.detail.value) }
watch(() => props.title, () => { dragTime.value = null })
</script>

<style scoped>
.topic-player { padding: var(--fi-space-20) var(--fi-space-24); background: var(--fi-color-bg); border: var(--fi-border-card); border-radius: var(--fi-radius-lg); }
.topic-player__heading { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-12); }
.topic-player__title { min-width: 0; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; color: var(--fi-color-text-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); }
.topic-player__close { flex-shrink: 0; margin: 0; padding: var(--fi-space-12); background: transparent; color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
.topic-player__close::after, .topic-player__toggle::after { border: 0; }
.topic-player__controls { display: flex; align-items: center; gap: var(--fi-space-10); margin-top: var(--fi-space-12); }
.topic-player__toggle { flex-shrink: 0; margin: 0; padding: var(--fi-space-12) var(--fi-space-20); border-radius: var(--fi-radius-round); color: var(--fi-color-primary); background: var(--fi-color-topic-audio-surface); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.topic-player__time { color: var(--fi-color-text-secondary); font-size: var(--fi-font-20); }
.topic-player__slider { flex: 1; min-width: 0; margin: 0 var(--fi-space-8); }
.topic-player__error { display: block; color: var(--fi-color-error); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); margin-top: var(--fi-space-12); }
</style>
