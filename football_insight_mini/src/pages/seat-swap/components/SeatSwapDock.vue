<template>
  <view class="dock">
    <button v-if="!hasMyRequest" class="dock-cta" hover-class="dock-cta--pressed" @tap="emit('cta')">
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
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-14);
  width: 100%;
  min-height: var(--fi-component-seat-swap-dock-height);
  box-sizing: border-box;
  padding: var(--fi-space-24) var(--fi-space-28);
  border-radius: var(--fi-radius-md);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  box-shadow: var(--fi-shadow-floating);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.dock-cta::after {
  border: 0;
}

.dock-cta--pressed {
  transform: scale(0.98);
}

.dock-cta__icon {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-space-36);
  height: var(--fi-space-36);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-white-rgb), 0.18);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
}

.dock-cta__label {
  font-size: var(--fi-font-28);
}

.dock-status {
  display: flex;
  align-items: center;
  min-height: var(--fi-component-seat-swap-dock-height);
  box-sizing: border-box;
  padding: var(--fi-space-20) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  box-shadow: var(--fi-shadow-floating);
}

.dock-status__body {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  flex-wrap: nowrap;
  min-width: 0;
  font-size: var(--fi-font-22);
}

.dock-status__dot {
  flex: 0 0 auto;
  width: var(--fi-space-14);
  height: var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-caption);
  box-shadow: 0 0 0 var(--fi-space-6) rgba(var(--fi-primitive-white-rgb), 0.12);
}

.dock-status__seat {
  min-width: 0;
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-xs);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dock-status__seat--current {
  background: var(--fi-color-seat-current-bg);
  color: var(--fi-color-seat-current);
}

.dock-status__seat--desired {
  flex: 1 1 auto;
  background: var(--fi-color-seat-desired-bg);
  color: var(--fi-color-seat-desired);
}

.dock-status__arrow {
  flex: 0 0 auto;
  color: rgba(var(--fi-primitive-white-rgb), 0.4);
}

.dock-status__manage {
  flex: 0 0 auto;
  margin-left: auto;
  padding: var(--fi-space-12) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-white-rgb), 0.12);
  color: rgba(var(--fi-primitive-white-rgb), 0.84);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}
</style>
