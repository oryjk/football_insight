<template>
  <button class="topic-read" @tap.stop="emit('read')">
    <view class="topic-read-icon" aria-hidden="true">
      <view class="topic-read-line" />
      <view class="topic-read-line" />
      <view class="topic-read-line topic-read-line--short" />
    </view>
    <text>阅读全文</text>
  </button>
</template>
<script setup lang="ts">
const emit = defineEmits<{ (e: 'read'): void }>()
</script>
<style scoped>
.topic-read { display: inline-flex; align-items: center; gap: var(--fi-space-8); margin: 0; padding: var(--fi-space-12) 0; background: transparent; color: var(--fi-color-text-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-bold); white-space: nowrap; }
.topic-read::after { border: 0; }
.topic-read-icon { box-sizing: border-box; display: flex; flex-direction: column; justify-content: center; flex-shrink: 0; gap: var(--fi-space-4); width: var(--fi-space-24); height: var(--fi-space-28); padding: var(--fi-space-4); border: var(--fi-border-width) solid currentColor; border-radius: var(--fi-space-4); color: var(--fi-color-primary); }
.topic-read-line { flex-shrink: 0; height: var(--fi-border-width); background: currentColor; border-radius: var(--fi-radius-round); }
.topic-read-line--short { width: 75%; }
</style>
