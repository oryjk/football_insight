<template>
  <button class="audio-entry" :class="{ 'audio-entry--ready': ready }" :disabled="!ready" @tap.stop="emit('play')">
    <view class="audio-entry__wave" aria-hidden="true">
      <view class="audio-entry__bar" />
      <view class="audio-entry__bar audio-entry__bar--tall" />
      <view class="audio-entry__bar audio-entry__bar--medium" />
      <view class="audio-entry__bar" />
    </view>
    <text>{{ !ready ? '四川话准备中' : loading ? '加载中' : playing ? '暂停播报' : label }}</text>
  </button>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ ready: boolean; label?: string; playing?: boolean; loading?: boolean }>(), {
  label: '四川话播报', playing: false, loading: false,
})
const emit = defineEmits<{ (e: 'play'): void }>()
</script>

<style scoped>
.audio-entry { display: inline-flex; align-items: center; justify-content: center; gap: var(--fi-space-10); margin: 0; padding: var(--fi-space-16) var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-page); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-snug); font-weight: var(--fi-weight-bold); white-space: nowrap; }
.audio-entry::after { border: 0; }
.audio-entry--ready { background: var(--fi-color-topic-audio-surface); color: var(--fi-color-primary); }
.audio-entry__wave { display: flex; align-items: center; gap: var(--fi-space-4); height: var(--fi-space-24); }
.audio-entry__bar { width: var(--fi-space-4); height: 50%; border-radius: var(--fi-radius-round); background: currentColor; }
.audio-entry__bar--tall { height: 100%; }
.audio-entry__bar--medium { height: 75%; }
</style>
