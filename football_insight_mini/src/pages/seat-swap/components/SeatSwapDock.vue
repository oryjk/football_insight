<template>
  <view class="dock">
    <button v-if="!hasMyRequest" class="dock-cta" @tap="emit('cta')">
      <text class="dock-cta__icon">+</text>
      <text class="dock-cta__label">{{ isLoggedIn ? '发布我的换座' : '登录后发布' }}</text>
    </button>
    <view v-else class="dock-status" @tap="emit('manage')">
      <view class="dock-status__body">
        <view class="dock-status__dot"></view>
        <text class="dock-status__seat dock-status__seat--current">{{ currentSeatLabel }}</text>
        <text class="dock-status__arrow">→</text>
        <text class="dock-status__seat dock-status__seat--desired">{{ desiredSummary }}</text>
        <text class="dock-status__manage">管理 ›</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{
  hasMyRequest: boolean
  isLoggedIn: boolean
  currentSeatLabel: string
  desiredSummary: string
}>()

const emit = defineEmits<{
  (e: 'cta'): void
  (e: 'manage'): void
}>()
</script>

<style scoped>
.dock {
  position: fixed;
  left: var(--fi-space-24);
  right: var(--fi-space-24);
  bottom: calc(var(--fi-space-24) + var(--window-bottom));
  z-index: 8;
}

.dock-cta {
  width: 100%;
  padding: var(--fi-space-24) var(--fi-space-28);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: 400;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-14);
  box-shadow: 0 16rpx 36rpx rgba(21, 22, 27, 0.2);
  letter-spacing: 1rpx;
}

.dock-cta::after {
  border: 0;
}

.dock-cta__icon {
  flex-shrink: 0;
  display: inline-flex;
  width: 36rpx;
  height: 36rpx;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.18);
  align-items: center;
  justify-content: center;
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
}

.dock-cta__label {
  font-size: var(--fi-font-28);
}

.dock-status {
  padding: var(--fi-space-20) var(--fi-space-22);
  border-radius: 26rpx;
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  box-shadow: 0 16rpx 36rpx rgba(21, 22, 27, 0.32);
}

.dock-status__dot {
  flex: 0 0 auto;
  width: 14rpx;
  height: 14rpx;
  border-radius: 50%;
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.18);
}

.dock-status__manage {
  flex: 0 0 auto;
  margin-left: auto;
  font-size: var(--fi-font-22);
  font-weight: 400;
  color: rgba(255, 255, 255, 0.84);
  padding: var(--fi-space-12) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: rgba(255, 255, 255, 0.12);
}

.dock-status__body {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  flex-wrap: nowrap;
  font-size: var(--fi-font-22);
  min-width: 0;
}

.dock-status__seat {
  min-width: 0;
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: 12rpx;
  font-weight: 400;
  line-height: var(--fi-leading-none);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock-status__seat--current {
  background: #fff1f0;
  color: #b42318;
}

.dock-status__seat--desired {
  flex: 1 1 auto;
  background: #eef8f2;
  color: #167348;
}

.dock-status__arrow {
  flex: 0 0 auto;
  color: rgba(255, 255, 255, 0.4);
}
</style>
