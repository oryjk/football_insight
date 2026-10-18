<template>
  <view class="panel share-poster-panel">
    <button class="poster-button" :loading="generating" :disabled="loading || generating" @tap="emit('generate')">
      {{ generating ? '正在生成分享图' : '生成分享图' }}
    </button>
    <text v-if="error" class="poster-error">{{ error }}</text>
    <template v-if="imagePath">
      <image class="poster-preview" :src="imagePath" mode="widthFix" @tap="emit('preview')" />
      <text class="poster-hint">点击预览，可长按图片发送给朋友</text>
      <button class="poster-button" :disabled="loading || generating" @tap="emit('save')">保存到相册</button>
    </template>
  </view>
</template>
<script setup lang="ts">
defineProps<{ imagePath: string; generating: boolean; loading: boolean; error: string }>()
const emit = defineEmits<{ (event: 'generate'): void; (event: 'preview'): void; (event: 'save'): void }>()
</script>
<style scoped>
@import '../ticket-watch-shared.css';
.share-poster-panel { display: flex; flex-direction: column; gap: var(--fi-space-16); }
.poster-button { width: 100%; margin: 0; border-radius: var(--fi-radius-md); font-size: var(--fi-font-26); background: var(--fi-color-page-soft); color: var(--fi-color-text-primary); }
.poster-preview { width: 100%; border-radius: var(--fi-radius-md); }
.poster-hint { font-size: var(--fi-font-22); color: var(--fi-color-text-secondary); }
.poster-error { font-size: var(--fi-font-24); color: var(--fi-color-error); }
</style>
