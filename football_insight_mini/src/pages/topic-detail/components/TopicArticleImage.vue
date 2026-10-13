<template>
  <view v-if="!failed" class="article-photo">
    <image class="article-photo__image" :src="image.url" mode="widthFix" lazy-load :style="{ aspectRatio: `${image.width} / ${image.height}` }" @error="failed = true" @tap="$emit('preview', image.url)" />
    <text class="article-photo__source">图片来源：{{ image.publisher }}</text>
  </view>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { DailyTopicImage } from '../../../types/dailyTopic'
const props = defineProps<{ image: DailyTopicImage }>()
defineEmits<{ (event: 'preview', url: string): void }>()
const failed = ref(false)
watch(() => props.image.url, () => { failed.value = false })
</script>

<style scoped>
.article-photo { margin-top: var(--fi-space-28); }
.article-photo__image { display: block; width: 100%; height: auto; border-radius: var(--fi-radius-md); background: var(--fi-color-bg); }
.article-photo__source { display: block; margin-top: var(--fi-space-12); color: var(--fi-color-text-caption); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); }
</style>
