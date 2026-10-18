<template>
  <view v-if="visible" class="fi-launch-splash" :class="{ 'fi-launch-splash--leaving': leaving }">
    <!-- idle（宽限期）只铺深色底层；logo 与文案在确定要展示时才出现，动画从那一刻开始 -->
    <view v-if="bodyShown" class="fi-launch-splash__body">
      <image class="fi-launch-splash__logo" :src="logo" mode="aspectFit" />
      <text class="fi-launch-splash__name">足球洞察</text>
      <text class="fi-launch-splash__tagline">{{ brandSubtitle }}</text>
      <view class="fi-launch-splash__line" />
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import logo from '../static/app-logo-mark.png'
import { brandSubtitle } from '../utils/brandSubtitle'
import type { SplashPhase } from '../composables/useLaunchSplash'

const props = defineProps<{ phase: SplashPhase }>()
const visible = computed(() => props.phase !== 'done')
// 只要展示过一次就保持，淡出期间内容不能消失。
const bodyShown = ref(props.phase === 'visible')
watch(() => props.phase, (phase) => {
  if (phase === 'visible') bodyShown.value = true
})
const leaving = computed(() => props.phase === 'leaving')
</script>

<style scoped lang="css">
.fi-launch-splash {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-color-member-card-bg);
  opacity: 1;
  transition: opacity 0.35s ease;
}

.fi-launch-splash--leaving {
  opacity: 0;
  pointer-events: none;
}

.fi-launch-splash__body {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 32rpx;
}

.fi-launch-splash__logo {
  width: 180rpx;
  height: 180rpx;
  animation: fi-launch-breathe 1.6s ease-in-out infinite;
}

.fi-launch-splash__name {
  color: var(--fi-color-bg);
  font-size: var(--fi-font-48);
  font-weight: var(--fi-weight-black);
  letter-spacing: 0.2em;
  animation: fi-launch-rise 0.7s ease both 0.1s;
}

.fi-launch-splash__tagline {
  color: var(--fi-color-member-card-accent);
  font-size: var(--fi-font-28);
  letter-spacing: 0.12em;
  animation: fi-launch-rise 0.7s ease both 0.3s;
}

.fi-launch-splash__line {
  position: relative;
  overflow: hidden;
  width: 144rpx;
  height: 4rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-brand-nav-border);
}

.fi-launch-splash__line::after {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--fi-color-primary);
  transform: translateX(-100%);
  animation: fi-launch-slide 1.3s ease-out forwards;
}

@keyframes fi-launch-breathe {
  50% { transform: scale(1.07); }
}

@keyframes fi-launch-rise {
  from { opacity: 0; transform: translateY(8px); }
}

@keyframes fi-launch-slide {
  to { transform: translateX(0); }
}
</style>
