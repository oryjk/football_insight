<template>
  <button class="topic-read" :class="{ 'topic-read--hero': variant === 'hero' }" @tap.stop="emit('read')">
    <view v-if="variant !== 'hero'" class="topic-read-icon" aria-hidden="true">
      <view class="topic-read-line" />
      <view class="topic-read-line" />
      <view class="topic-read-line topic-read-line--short" />
    </view>
    <text>{{ variant === 'hero' ? '阅读' : '阅读全文' }}</text>
  </button>
</template>
<script setup lang="ts">
withDefaults(defineProps<{ variant?: 'default' | 'hero' }>(), { variant: 'default' })
const emit = defineEmits<{ (e: 'read'): void }>()
</script>
<style scoped>
.topic-read { display: inline-flex; align-items: center; gap: var(--fi-space-8); margin: 0; padding: var(--fi-space-12) 0; background: transparent; color: var(--fi-color-text-primary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); font-weight: var(--fi-weight-bold); white-space: nowrap; }
.topic-read::after { border: 0; }
.topic-read-icon { box-sizing: border-box; display: flex; flex-direction: column; justify-content: center; flex-shrink: 0; gap: var(--fi-space-4); width: var(--fi-space-24); height: var(--fi-space-28); padding: var(--fi-space-4); border: var(--fi-border-width) solid currentColor; border-radius: var(--fi-space-4); color: var(--fi-color-primary); }
.topic-read-line { flex-shrink: 0; height: var(--fi-border-width); background: currentColor; border-radius: var(--fi-radius-round); }
.topic-read-line--short { width: 75%; }
.topic-read--hero { box-sizing: border-box; justify-content: center; height: var(--fi-component-topic-hero-action-height); padding: 0 var(--fi-space-22); border-radius: var(--fi-radius-round); background: var(--fi-color-surface); color: var(--fi-color-text-primary); font-size: var(--fi-font-24); }
</style>
