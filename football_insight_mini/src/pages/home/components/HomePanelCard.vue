<template>
  <view class="panel">
    <view class="section-heading">
      <slot name="heading">
        <view>
          <text v-if="kicker" class="section-kicker">{{ kicker }}</text>
          <text v-if="title" class="section-title">{{ title }}</text>
        </view>
      </slot>
      <slot name="meta" />
    </view>
    <slot />
  </view>
</template>

<script setup lang="ts">
defineProps<{
  kicker?: string
  title?: string
}>()
</script>

<style scoped lang="css">
.panel {
  background: rgba(255, 255, 255, 0.94);
  border-radius: 36rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
  padding: 20rpx;
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12rpx;
}

.section-heading > view {
  display: grid;
  gap: 8rpx;
  min-width: 0;
}

.section-kicker {
  display: block;
  margin: 0;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 44rpx;
  line-height: 1.16;
  font-weight: 800;
}
</style>
