<template>
  <view v-if="availableImages.length" class="topic-gallery">
    <view class="topic-gallery__frame">
      <swiper class="topic-gallery__swiper" :current="activeIndex" :circular="availableImages.length > 1" :autoplay="false" @change="onSlideChange">
        <swiper-item v-for="(image, index) in availableImages" :key="image.url">
          <image class="topic-gallery__image" :src="image.url" mode="aspectFit" lazy-load :aria-label="`查看第 ${index + 1} 张配图`" @error="onImageError(image.url)" @tap="preview(image.url)" />
        </swiper-item>
      </swiper>
      <text v-if="availableImages.length > 1" class="topic-gallery__counter">{{ activeIndex + 1 }} / {{ availableImages.length }}</text>
    </view>
    <view v-if="availableImages.length > 1" class="topic-gallery__dots">
      <button v-for="(image, index) in availableImages" :key="image.url" class="topic-gallery__dot-button" :aria-label="`切换到第 ${index + 1} 张配图`" :aria-pressed="activeIndex === index" @tap="activeIndex = index">
        <view class="topic-gallery__dot" :class="{ 'topic-gallery__dot--active': activeIndex === index }" />
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { DailyTopicImage } from '../../../types/dailyTopic'
const props = defineProps<{ images: DailyTopicImage[] }>()
const emit = defineEmits<{ (event: 'preview', url: string, urls: string[]): void }>()
const activeIndex = ref(0)
const failedUrls = ref<string[]>([])
const availableImages = computed(() => props.images.filter(image => !failedUrls.value.includes(image.url)))
watch(() => props.images.map(image => image.url).join('\n'), () => {
  activeIndex.value = 0
  failedUrls.value = []
})
function onSlideChange(event: { detail: { current: number } }) {
  activeIndex.value = Math.max(0, Math.min(event.detail.current, availableImages.value.length - 1))
}
function onImageError(url: string) {
  if (!failedUrls.value.includes(url)) failedUrls.value = [...failedUrls.value, url]
  activeIndex.value = 0
}
function preview(url: string) {
  emit('preview', url, availableImages.value.map(image => image.url))
}
</script>

<style scoped>
.topic-gallery { min-width: 0; }
.topic-gallery__frame { position: relative; border-radius: var(--fi-radius-md); background: var(--fi-color-page-soft); overflow: hidden; }
.topic-gallery__swiper { height: var(--fi-component-topic-gallery-height); }
.topic-gallery__image { display: block; width: 100%; height: 100%; }
.topic-gallery__counter { position: absolute; right: var(--fi-space-12); bottom: var(--fi-space-12); padding: var(--fi-space-4) var(--fi-space-12); border-radius: var(--fi-radius-round); background: var(--fi-color-surface-frosted); color: var(--fi-color-text-primary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); pointer-events: none; }
.topic-gallery__dots { display: flex; justify-content: center; }
.topic-gallery__dot-button { display: flex; align-items: center; justify-content: center; width: var(--fi-space-40); height: var(--fi-space-40); margin: 0; padding: 0; background: transparent; line-height: var(--fi-leading-normal); }
.topic-gallery__dot-button::after { border: 0; }
.topic-gallery__dot { width: var(--fi-space-8); height: var(--fi-space-8); border-radius: var(--fi-radius-round); background: var(--fi-color-divider); }
.topic-gallery__dot--active { width: var(--fi-space-20); background: var(--fi-color-primary); }
</style>
