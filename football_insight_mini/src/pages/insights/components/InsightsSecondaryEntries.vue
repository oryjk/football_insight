<template>
  <view class="secondary-entries">
    <view class="secondary-entry" role="button" aria-label="排阵容" hover-class="secondary-entry--pressed" :hover-stay-time="80" @tap="emit('open', 'tactics')">
      <text class="secondary-entry__title">排阵容</text>
      <text class="secondary-entry__caption">看大家怎么排</text>
    </view>
    <view class="secondary-entry" role="button" aria-label="转会表态" hover-class="secondary-entry--pressed" :hover-stay-time="80" @tap="emit('open', 'transfer')">
      <text class="secondary-entry__title">转会表态</text>
      <text class="secondary-entry__caption">送走谁、买回谁</text>
    </view>
  </view>
</template>

<script setup lang="ts">
const emit = defineEmits<{ (e: 'open', target: 'tactics' | 'transfer'): void }>()
</script>

<style scoped lang="css">
.secondary-entries { display: flex; flex-direction: row; gap: var(--fi-space-16); }
.secondary-entry {
  flex: 1;
  min-width: 0;
  box-sizing: border-box;
  padding: var(--fi-space-28) var(--fi-space-20);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
}
.secondary-entry--pressed { opacity: var(--fi-opacity-insights-pressed); }
.secondary-entry__title { display: block; color: var(--fi-color-text-strong); font-size: var(--fi-font-32); line-height: var(--fi-leading-snug); font-weight: var(--fi-weight-extrabold); }
.secondary-entry__caption { display: block; margin-top: var(--fi-space-8); color: var(--fi-color-text-muted); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
</style>
