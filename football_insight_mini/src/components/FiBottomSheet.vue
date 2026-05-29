<template>
  <view
    v-if="rendered"
    class="fi-sheet-mask"
    :class="{ 'fi-sheet-mask--closing': closing }"
    @touchmove.stop.prevent
    @tap="handleMaskTap"
  >
    <view
      class="fi-sheet-card"
      :class="{ 'fi-sheet-card--closing': closing }"
      :style="cardStyle"
      @touchmove.stop
      @tap.stop
    >
      <view class="fi-sheet-card__handle"></view>

      <view v-if="title || eyebrow" class="fi-sheet-card__header">
        <view class="fi-sheet-card__heading">
          <text v-if="eyebrow" class="fi-sheet-card__eyebrow">{{ eyebrow }}</text>
          <text v-if="title" class="fi-sheet-card__title">{{ title }}</text>
        </view>
        <button class="fi-sheet-card__close" @tap="handleClose">✕</button>
      </view>

      <scroll-view scroll-y class="fi-sheet-card__body" @touchmove.stop>
        <slot></slot>
      </scroll-view>

      <view
        v-if="hasFooter"
        class="fi-sheet-card__footer"
        :class="{ 'fi-sheet-card__footer--compact': compactFooter }"
      >
        <slot name="footer"></slot>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, useSlots, watch } from 'vue'

interface Props {
  visible: boolean
  title?: string
  eyebrow?: string
  height?: string
  closeOnMask?: boolean
  compactFooter?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  eyebrow: '',
  height: '88vh',
  closeOnMask: true,
  compactFooter: false,
})

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
}>()

const slots = useSlots()
const hasFooter = computed(() => !!slots.footer)
const cardStyle = computed(() => ({
  '--fi-sheet-height': props.height,
  height: props.height,
  maxHeight: props.height,
}))
const rendered = ref(props.visible)
const closing = ref(false)
let closeTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.visible,
  (visible) => {
    if (closeTimer) {
      clearTimeout(closeTimer)
      closeTimer = null
    }

    if (visible) {
      rendered.value = true
      closing.value = false
      return
    }

    if (!rendered.value) return
    closing.value = true
    closeTimer = setTimeout(() => {
      rendered.value = false
      closing.value = false
      closeTimer = null
    }, 220)
  },
)

function handleClose() {
  emit('update:visible', false)
  emit('close')
}

function handleMaskTap() {
  if (props.closeOnMask) {
    handleClose()
  }
}
</script>

<style scoped>
.fi-sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
  animation: fi-overlay-fade-in 180ms ease both;
}

.fi-sheet-mask--closing {
  animation: fi-overlay-fade-out 220ms ease both;
}

.fi-sheet-card {
  position: relative;
  width: 100%;
  height: var(--fi-sheet-height);
  max-height: var(--fi-sheet-height);
  display: flex;
  flex-direction: column;
  border-radius: 36rpx 36rpx 0 0;
  background: #ffffff;
  border-top: 1rpx solid rgba(232, 233, 238, 0.95);
  box-shadow: 0 -24rpx 56rpx rgba(12, 14, 20, 0.12);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
  overflow: hidden;
}

.fi-sheet-card--closing {
  animation: fi-sheet-down 220ms cubic-bezier(0.4, 0, 1, 1) both;
}

.fi-sheet-card__handle {
  flex-shrink: 0;
  width: 56rpx;
  height: 6rpx;
  border-radius: 999rpx;
  background: rgba(145, 150, 160, 0.32);
  margin: 18rpx auto 6rpx;
}

.fi-sheet-card__header {
  flex-shrink: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16rpx;
  padding: 12rpx 30rpx 18rpx;
}

.fi-sheet-card__heading {
  flex: 1;
  min-width: 0;
}

.fi-sheet-card__eyebrow {
  display: block;
  color: #8f9198;
  font-size: 20rpx;
  font-weight: 400;
  margin-bottom: 4rpx;
  letter-spacing: 0;
}

.fi-sheet-card__title {
  display: block;
  color: #121212;
  font-size: 34rpx;
  font-weight: 400;
  line-height: 1.2;
}

.fi-sheet-card__close {
  flex-shrink: 0;
  width: 52rpx;
  height: 52rpx;
  padding: 0;
  border-radius: 50%;
  background: #f6f7fb;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: 24rpx;
  font-weight: 400;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.fi-sheet-card__close::after {
  border: none;
}

.fi-sheet-card__body {
  flex: 1;
  min-height: 0;
  height: 100%;
  padding: 6rpx 30rpx 20rpx;
  box-sizing: border-box;
}

.fi-sheet-card__footer {
  flex-shrink: 0;
  padding: 18rpx 30rpx calc(28rpx + env(safe-area-inset-bottom));
  border-top: 1rpx solid rgba(232, 233, 238, 0.95);
  background: #ffffff;
  display: flex;
  gap: 16rpx;
}

.fi-sheet-card__footer--compact {
  padding-top: 14rpx;
  padding-bottom: 18rpx;
}

@keyframes fi-overlay-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fi-overlay-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}

@keyframes fi-sheet-up {
  from {
    opacity: 0;
    transform: translateY(100%);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fi-sheet-down {
  from {
    opacity: 1;
    transform: translateY(0);
  }
  to {
    opacity: 0;
    transform: translateY(100%);
  }
}
</style>
