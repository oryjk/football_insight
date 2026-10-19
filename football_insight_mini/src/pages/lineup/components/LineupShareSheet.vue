<template>
  <FiBottomSheet :visible="visible" title="分享战术板" height="72vh" :z-index="100" @update:visible="emit('update:visible', $event)">
    <view class="share-content">
      <text class="share-copy">分享给好友，一起聊聊你的最佳阵容。</text>
      <!-- #ifdef MP-WEIXIN -->
      <button class="share-action share-action--primary" open-type="share" :disabled="busy || !ready">分享给好友</button>
      <!-- #endif -->
      <button class="share-action" :disabled="busy" @tap="emit('generate')">{{ busy ? '正在准备…' : imagePath ? '重新生成分享图' : '生成分享图' }}</button>
      <text v-if="error" class="share-error">{{ error }}</text>
      <template v-if="imagePath">
        <image :src="imagePath" class="share-preview" mode="widthFix" @tap="emit('preview')" />
        <view class="share-row">
          <button class="share-action" @tap="emit('preview')">预览大图</button>
          <button class="share-action share-action--primary" @tap="emit('save')">保存到相册</button>
        </view>
      </template>
      <text class="share-copy">分享图中的小程序码打开正式版；体验版可先验证好友卡片。</text>
    </view>
  </FiBottomSheet>
</template>
<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
defineProps<{ visible: boolean; busy: boolean; ready: boolean; imagePath: string; error: string }>()
const emit = defineEmits<{ (e: 'update:visible', value: boolean): void; (e: 'generate'): void; (e: 'preview'): void; (e: 'save'): void }>()
</script>
<style scoped>
.share-content { display: flex; flex-direction: column; gap: var(--fi-space-20); }
.share-copy { color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.share-error { color: var(--fi-color-primary-deep); font-size: var(--fi-font-24); }
.share-row { display: flex; gap: var(--fi-space-16); }
.share-action { margin: 0; padding: var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-primary); font-size: var(--fi-font-28); line-height: var(--fi-leading-normal); }
.share-row > .share-action { flex: 1; }
.share-action::after { border: 0; }
.share-action--primary { background: var(--fi-color-primary); color: var(--fi-color-surface); }
.share-action[disabled] { opacity: var(--fi-opacity-pressed); }
.share-preview { width: 100%; border-radius: var(--fi-radius-md); }
</style>
