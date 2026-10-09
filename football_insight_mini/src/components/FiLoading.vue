<template>
  <view class="loading-card">
    <text class="loading-card__title">{{ title }}</text>
    <text class="loading-card__caption">{{ caption }}</text>
  </view>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  caption: string
}>()
</script>

<style scoped lang="css">
.loading-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-component-loading-gap);
  padding: var(--fi-component-loading-padding);
  border-radius: var(--fi-component-loading-radius);
  background: var(--fi-component-loading-bg);
  box-shadow: var(--fi-component-loading-shadow);
}

.loading-card__title {
  font-size: var(--fi-component-loading-title-size);
  font-weight: var(--fi-component-loading-title-weight);
  color: var(--fi-component-loading-title-color);
}

.loading-card__caption {
  font-size: var(--fi-component-loading-caption-size);
  line-height: var(--fi-component-loading-caption-leading);
  color: var(--fi-component-loading-caption-color);
}
</style>
