<template>
  <button class="audio-entry" :class="{ 'audio-entry--ready': ready, 'audio-entry--hero': variant === 'hero' }" :disabled="!ready" @tap.stop="emit('play')">
    <view v-if="variant === 'hero'" class="audio-entry__control" aria-hidden="true">
      <view v-if="playing" class="audio-entry__pause"><view /><view /></view>
      <view v-else class="audio-entry__play" />
    </view>
    <view v-else class="audio-entry__wave" aria-hidden="true">
      <view class="audio-entry__bar" />
      <view class="audio-entry__bar audio-entry__bar--tall" />
      <view class="audio-entry__bar audio-entry__bar--medium" />
      <view class="audio-entry__bar" />
    </view>
    <text>{{ !ready ? (variant === 'hero' ? '准备中' : '语音准备中') : loading ? '加载中' : playing ? (variant === 'hero' ? '暂停' : '暂停播报') : label }}</text>
  </button>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ ready: boolean; label?: string; playing?: boolean; loading?: boolean; variant?: 'default' | 'hero' }>(), {
  label: '播报', playing: false, loading: false, variant: 'default',
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
.audio-entry.audio-entry--hero { box-sizing: border-box; height: var(--fi-component-topic-hero-action-height); padding: 0 var(--fi-space-20); gap: var(--fi-space-10); background: var(--fi-color-topic-hero-control); color: var(--fi-color-topic-hero-text); font-size: var(--fi-font-26); }
.audio-entry.audio-entry--hero[disabled] { background: var(--fi-color-topic-hero-disabled); color: var(--fi-color-topic-hero-caption); }
.audio-entry__control { display: flex; align-items: center; justify-content: center; width: var(--fi-space-20); height: var(--fi-space-24); flex-shrink: 0; }
.audio-entry__play { width: 0; height: 0; border-top: var(--fi-space-10) solid transparent; border-bottom: var(--fi-space-10) solid transparent; border-left: var(--fi-space-16) solid currentColor; }
.audio-entry__pause { display: flex; justify-content: space-between; width: var(--fi-space-16); height: var(--fi-space-24); }
.audio-entry__pause view { width: var(--fi-space-6); height: 100%; border-radius: var(--fi-space-1); background: currentColor; }
</style>
