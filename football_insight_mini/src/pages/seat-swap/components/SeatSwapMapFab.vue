<template>
  <view
    class="map-fab"
    :class="{ 'map-fab--visible': visible }"
    role="button"
    :aria-label="clearable ? `当前筛选 ${label}，打开球场图` : '打开球场图'"
    :aria-hidden="!visible"
  >
    <view class="map-fab__main" hover-class="map-fab__main--pressed" @tap="emit('open')">
      <image class="map-fab__icon" src="/static/seat-swap/map-ink.svg" mode="aspectFit" />
      <view class="map-fab__labels">
        <text :key="`in-${label}`" class="map-fab__label map-fab__label--in">{{ label }}</text>
        <text v-if="leavingLabel" :key="`out-${leavingLabel}`" class="map-fab__label map-fab__label--out">{{ leavingLabel }}</text>
      </view>
    </view>
    <view
      v-if="clearable"
      class="map-fab__clear"
      hover-class="map-fab__clear--pressed"
      aria-label="清除筛选"
      @tap.stop="emit('clear')"
    >
      <text>✕</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{ visible: boolean; label: string; clearable: boolean }>()
const emit = defineEmits<{ (e: 'open'): void; (e: 'clear'): void }>()

// 文案交叉过渡：旧文案保留一个动画时长后移除。
const LABEL_SWAP_MS = 220
const leavingLabel = ref('')
let leaveTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.label, (_next, previous) => {
  if (!previous) return
  leavingLabel.value = previous
  if (leaveTimer) clearTimeout(leaveTimer)
  leaveTimer = setTimeout(() => { leavingLabel.value = '' }, LABEL_SWAP_MS)
})

onBeforeUnmount(() => {
  if (leaveTimer) clearTimeout(leaveTimer)
})
</script>

<style scoped>
/* 常驻节点，用 class 切换进出场，保证退场动画播完；隐藏时不拦截点击。 */
.map-fab {
  position: fixed;
  right: var(--fi-space-24);
  bottom: calc(var(--fi-space-12) + var(--window-bottom) + var(--fi-component-seat-swap-dock-height) + var(--fi-space-16));
  z-index: 9;
  display: flex;
  align-items: center;
  border-radius: var(--fi-radius-round);
  /* 辅助工具：白底墨字，与红色发布按钮、卡片描边按钮区分开。 */
  border: var(--fi-border-card);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-floating);
  opacity: 0;
  transform: translateY(var(--fi-space-16)) scale(0.95);
  pointer-events: none;
  transition:
    opacity 200ms var(--fi-motion-ease-in),
    transform 200ms var(--fi-motion-ease-in);
}

.map-fab--visible {
  opacity: 1;
  transform: translateY(0) scale(1);
  pointer-events: auto;
  transition:
    opacity var(--fi-motion-duration-slow) var(--fi-motion-ease-out),
    transform var(--fi-motion-duration-slow) var(--fi-motion-ease-out);
}

.map-fab__main {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  padding: var(--fi-space-18) var(--fi-space-24);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.map-fab__main--pressed {
  transform: scale(0.94);
}

.map-fab__icon {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.map-fab__labels {
  position: relative;
  overflow: hidden;
}

.map-fab__label {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  white-space: nowrap;
}

.map-fab__label--in {
  animation: map-fab-label-in var(--fi-motion-duration-base) var(--fi-motion-ease-out) both;
}

.map-fab__label--out {
  position: absolute;
  top: 0;
  left: 0;
  animation: map-fab-label-out var(--fi-motion-duration-base) var(--fi-motion-ease-out) both;
}

.map-fab__clear {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: var(--fi-space-64);
  min-height: var(--fi-space-64);
  margin-right: var(--fi-space-6);
  border-radius: var(--fi-radius-round);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  transition: transform var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.map-fab__clear--pressed {
  transform: scale(0.9);
}

@keyframes map-fab-label-in {
  from { opacity: 0; transform: translateY(var(--fi-space-8)); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes map-fab-label-out {
  from { opacity: 1; transform: translateY(0); }
  to { opacity: 0; transform: translateY(calc(-1 * var(--fi-space-8))); }
}

@media (prefers-reduced-motion: reduce) {
  .map-fab,
  .map-fab--visible {
    transform: none;
    transition: opacity 150ms linear;
  }

  .map-fab__label--in {
    animation-name: fi-overlay-fade-in;
    animation-duration: 150ms;
  }

  .map-fab__label--out {
    display: none;
  }
}
</style>
