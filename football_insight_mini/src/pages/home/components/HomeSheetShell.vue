<template>
  <view class="sheet-mask" :class="{ 'sheet-mask--animated': animated }" @tap="emit('close')">
    <view class="sheet-card" :class="{ 'sheet-card--animated': animated }" @tap.stop>
      <view class="section-heading">
        <view>
          <text v-if="kicker" class="section-kicker">{{ kicker }}</text>
          <slot name="title">
            <text v-if="title" class="section-title">{{ title }}</text>
          </slot>
        </view>
        <button v-if="closable" class="sheet-shell__close" @click="emit('close')">关闭</button>
        <slot v-else name="meta" />
      </view>
      <slot />
    </view>
  </view>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    kicker?: string
    title?: string
    closable?: boolean
    animated?: boolean
  }>(),
  { kicker: '', title: '', closable: true, animated: false },
)

const emit = defineEmits<{
  (e: 'close'): void
}>()
</script>

<style scoped lang="css">
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(var(--fi-primitive-ink-rgb), 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
}

.sheet-mask--animated {
  animation: sheet-shell-mask-fade 220ms ease-out both;
}

.sheet-card {
  width: 100%;
  max-height: 78vh;
  border-radius: 36rpx 36rpx 0 0;
  background: rgba(255, 255, 255, 0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12, 14, 20, 0.12);
  overflow-y: auto;
}

.sheet-card--animated {
  animation: sheet-shell-card-enter 280ms cubic-bezier(0.2, 0.9, 0.22, 1) both;
}

.section-heading {
  display: flex;
  align-items: center;
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

.sheet-shell__close {
  display: inline-flex;
  margin: 0 0 0 auto;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 18rpx;
  border-radius: 999rpx;
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: 24rpx;
}

@keyframes sheet-shell-mask-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes sheet-shell-card-enter {
  from {
    opacity: 0;
    transform: translateY(32rpx) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
