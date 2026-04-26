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
  gap: 12rpx;
  padding: 40rpx 32rpx;
  border-radius: 28rpx;
  background: rgba(255, 255, 255, 0.92);
  box-shadow: 0 24rpx 60rpx rgba(26, 28, 36, 0.08);
}

.loading-card__title {
  font-size: 34rpx;
  font-weight: 700;
  color: #15161b;
}

.loading-card__caption {
  font-size: 26rpx;
  line-height: 1.6;
  color: #7a7f8b;
}
</style>
