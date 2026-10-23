<template>
  <view
    v-if="rendered"
    class="fi-sheet-mask"
    :class="{ 'fi-sheet-mask--closing': closing }"
    :style="maskStyle"
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

      <view
        v-if="fitContent"
        class="fi-sheet-card__body fi-sheet-card__body--fit-content"
        @touchmove.stop
      >
        <slot></slot>
      </view>
      <scroll-view v-else scroll-y class="fi-sheet-card__body" @touchmove.stop>
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
import { computed, onBeforeUnmount, ref, useSlots, watch } from 'vue'

interface Props {
  visible: boolean
  title?: string
  eyebrow?: string
  height?: string
  closeOnMask?: boolean
  compactFooter?: boolean
  fitContent?: boolean
  zIndex?: number
  respectWindowBottom?: boolean
  /** 小程序里内容含输入框时开启：自己按键盘高度抬起面板，输入框需配 :adjust-position="false" */
  avoidKeyboard?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  eyebrow: '',
  height: '88vh',
  closeOnMask: true,
  compactFooter: false,
  fitContent: false,
  zIndex: 40,
  respectWindowBottom: false,
  avoidKeyboard: false,
})

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
}>()

const slots = useSlots()
const hasFooter = computed(() => !!slots.footer)
const keyboardHeight = ref(0)
const maskStyle = computed(() => ({
  zIndex: props.zIndex,
  bottom: keyboardHeight.value > 0
    ? `${keyboardHeight.value}px`
    : (props.respectWindowBottom ? 'var(--window-bottom, 0px)' : undefined),
}))
const cardStyle = computed(() => ({
  '--fi-sheet-height': props.height,
  height: props.fitContent ? 'auto' : props.height,
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

// 面板是 fixed 定位、正文在 scroll-view 里，小程序原生 adjust-position 推页面推不到位，
// 键盘会盖住面板底部的输入框；改为监听键盘高度，把遮罩底边抬到键盘上沿。
function handleKeyboardHeightChange(result: { height: number }) {
  keyboardHeight.value = Math.max(0, result.height || 0)
}

let keyboardListening = false

function syncKeyboardListener(active: boolean) {
  // #ifdef MP-WEIXIN
  if (active && !keyboardListening) {
    uni.onKeyboardHeightChange(handleKeyboardHeightChange)
    keyboardListening = true
  } else if (!active && keyboardListening) {
    uni.offKeyboardHeightChange(handleKeyboardHeightChange)
    keyboardListening = false
  }
  // #endif
  if (!active) keyboardHeight.value = 0
}

watch(
  () => props.visible && props.avoidKeyboard,
  syncKeyboardListener,
  { immediate: true },
)

onBeforeUnmount(() => syncKeyboardListener(false))

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
  transition: bottom 200ms ease;
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
  border-radius: var(--fi-radius-xl) var(--fi-radius-xl) 0 0;
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
  border-radius: var(--fi-radius-round);
  background: rgba(145, 150, 160, 0.32);
  margin: 18rpx auto 6rpx;
}

.fi-sheet-card__header {
  flex-shrink: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-16);
  padding: var(--fi-space-12) var(--fi-space-30) var(--fi-space-18);
}

.fi-sheet-card__heading {
  flex: 1;
  min-width: 0;
}

.fi-sheet-card__eyebrow {
  display: block;
  color: #8f9198;
  font-size: var(--fi-font-20);
  font-weight: 400;
  margin-bottom: 4rpx;
  letter-spacing: 0;
}

.fi-sheet-card__title {
  display: block;
  color: #121212;
  font-size: var(--fi-font-34);
  font-weight: 400;
  line-height: 1.2;
}

.fi-sheet-card__close {
  flex-shrink: 0;
  margin: 0 0 0 auto;
  width: 52rpx;
  height: 52rpx;
  padding: 0;
  border-radius: 50%;
  background: #f6f7fb;
  border: 1rpx solid rgba(232, 233, 238, 0.95);
  color: #6d7280;
  font-size: var(--fi-font-24);
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
  padding: var(--fi-space-6) var(--fi-space-30) var(--fi-space-20);
  box-sizing: border-box;
}

.fi-sheet-card__body--fit-content {
  flex: 0 1 auto;
  height: auto;
  max-height: var(--fi-sheet-height);
  overflow-y: auto;
}

.fi-sheet-card__footer {
  flex-shrink: 0;
  padding: 18rpx 30rpx calc(28rpx + env(safe-area-inset-bottom));
  border-top: 1rpx solid rgba(232, 233, 238, 0.95);
  background: #ffffff;
  display: flex;
  gap: var(--fi-space-16);
}

.fi-sheet-card__footer--compact {
  padding-top: var(--fi-space-14);
  padding-bottom: var(--fi-space-18);
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
