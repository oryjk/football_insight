<template>
  <view class="action-bar">
    <view class="action-bar__tools">
      <button class="tool tool--reset" hover-class="action--pressed" :disabled="busy || sharePreparing" @tap="emit('reset')">
        <image class="tool__icon" src="/static/lineup/reset.png" mode="aspectFit" aria-hidden="true" />
        <text>重置</text>
      </button>
      <button class="tool" hover-class="action--pressed" :disabled="busy || sharePreparing" @tap="emit('share')">
        <image class="tool__icon" src="/static/lineup/share.png" mode="aspectFit" aria-hidden="true" />
        <text>{{ sharePreparing ? '准备中' : '分享' }}</text>
      </button>
    </view>
    <view class="action-bar__writes">
      <button class="write write--draft" hover-class="action--pressed" :disabled="busy || sharePreparing" @tap="emit('draft')">
        <image class="write__icon" src="/static/lineup/draft.png" mode="aspectFit" aria-hidden="true" />
        <text>{{ savingDraft ? '保存中…' : '存草稿' }}</text>
      </button>
      <button class="write write--publish" hover-class="action--pressed" :disabled="busy || sharePreparing || !complete" @tap="emit('publish')">
        <image class="write__icon" src="/static/lineup/publish.png" mode="aspectFit" aria-hidden="true" />
        <text>{{ publishing ? '发布中…' : published ? '更新发布' : '发布' }}</text>
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{
  busy: boolean
  savingDraft: boolean
  publishing: boolean
  sharePreparing: boolean
  complete: boolean
  published: boolean
}>()
const emit = defineEmits<{
  (event: 'reset'): void
  (event: 'share'): void
  (event: 'draft'): void
  (event: 'publish'): void
}>()
</script>

<style scoped>
.action-bar,
.action-bar__tools,
.action-bar__writes {
  display: flex;
  align-items: stretch;
}

.action-bar {
  gap: var(--fi-space-16);
}

.action-bar__tools {
  flex-shrink: 0;
  gap: var(--fi-space-8);
}

.action-bar__writes {
  flex: 1;
  min-width: 0;
  gap: var(--fi-space-12);
}

.tool,
.write {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-surface);
  color: var(--fi-color-text-secondary);
  line-height: var(--fi-leading-snug);
}

.tool::after,
.write::after {
  border: 0;
}

.tool {
  flex-direction: column;
  min-width: calc(var(--fi-space-64) + var(--fi-space-8));
  padding: var(--fi-space-12) var(--fi-space-8);
  gap: var(--fi-space-4);
  border-radius: var(--fi-radius-md);
  font-size: var(--fi-font-20);
}

.tool--reset {
  color: var(--fi-color-primary);
}

.tool__icon {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.write {
  flex: 1;
  min-width: 0;
  padding: var(--fi-space-16) var(--fi-space-12);
  gap: var(--fi-space-8);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  white-space: nowrap;
}

.write__icon {
  flex-shrink: 0;
  width: var(--fi-space-28);
  height: var(--fi-space-28);
}

.write--draft {
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-primary);
}

.write--publish {
  border-color: var(--fi-color-primary-deep);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-soft);
}

.tool[disabled],
.write[disabled],
.action--pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
