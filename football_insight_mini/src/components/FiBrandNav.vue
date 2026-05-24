<template>
  <view class="fi-brand-nav">
    <view class="fi-brand-nav__status-spacer"></view>
    <view class="fi-brand-nav__bar">
      <view class="fi-brand-nav__brand" :style="brandStyle">
        <image :src="logo" mode="aspectFill" class="fi-brand-nav__logo" />
        <view class="fi-brand-nav__copy">
          <text class="fi-brand-nav__name">足球洞察</text>
          <text class="fi-brand-nav__tagline">最懂球迷的工具</text>
        </view>
      </view>
      <button class="fi-brand-nav__ai" :style="aiButtonStyle" @tap.stop="handleAiTap">
        <text class="fi-brand-nav__ai-dot"></text>
        <text>AI 洞察</text>
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import logo from '../static/app-logo.jpg'
import { requestOpenAiChat } from '../utils/aiEntryIntent'

const props = withDefaults(defineProps<{
  openOnCurrentPage?: boolean
}>(), {
  openOnCurrentPage: false,
})

const emit = defineEmits<{
  (e: 'open-ai'): void
}>()

const aiButtonMetrics = ref({
  top: 0,
  right: 190,
  height: 58,
})

const brandMetrics = ref({
  top: 0,
  height: 64,
})

const aiButtonStyle = computed(() => {
  const top = aiButtonMetrics.value.top
  return {
    top: top ? `${top}px` : '',
    right: `${aiButtonMetrics.value.right}px`,
    height: `${aiButtonMetrics.value.height}px`,
  }
})

const brandStyle = computed(() => {
  const top = brandMetrics.value.top
  return {
    top: top ? `${top}px` : '',
    height: `${brandMetrics.value.height}px`,
  }
})

function syncAiButtonWithMenuCapsule(): void {
  try {
    const systemInfo = uni.getSystemInfoSync()
    const menuButton = uni.getMenuButtonBoundingClientRect?.()

    if (!systemInfo.windowWidth || !menuButton?.width || !menuButton?.height) {
      return
    }

    aiButtonMetrics.value = {
      top: menuButton.top,
      right: systemInfo.windowWidth - menuButton.left + 8,
      height: menuButton.height,
    }

    const brandHeight = 64
    brandMetrics.value = {
      top: menuButton.top + (menuButton.height - brandHeight) / 2,
      height: brandHeight,
    }
  } catch {
    // Keep the CSS fallback when the platform does not expose capsule metrics.
  }
}

function handleAiTap(): void {
  if (props.openOnCurrentPage) {
    emit('open-ai')
    return
  }

  requestOpenAiChat()
  uni.switchTab({
    url: '/pages/home/index',
  })
}

onMounted(() => {
  syncAiButtonWithMenuCapsule()
})
</script>

<style scoped>
.fi-brand-nav {
  --fi-brand-nav-status-height: var(--status-bar-height);
  --fi-brand-nav-bar-height: 154rpx;
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  z-index: 80;
  background: rgba(247, 248, 250, 0.82);
  border-bottom: 1rpx solid rgba(232, 233, 238, 0.72);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.fi-brand-nav__status-spacer {
  height: var(--fi-brand-nav-status-height);
}

.fi-brand-nav__bar {
  position: relative;
  min-height: var(--fi-brand-nav-bar-height);
  padding: 14rpx 196rpx 48rpx 24rpx;
}

.fi-brand-nav__brand {
  position: fixed;
  left: 24rpx;
  top: calc(var(--fi-brand-nav-status-height) + 14rpx);
  z-index: 81;
  min-width: 0;
  width: 360rpx;
  height: 64rpx;
  display: flex;
  align-items: center;
  gap: 14rpx;
}

.fi-brand-nav__logo {
  flex: 0 0 auto;
  width: 64rpx;
  height: 64rpx;
  border-radius: 18rpx;
  border: 2rpx solid rgba(255, 255, 255, 0.78);
  box-shadow: 0 10rpx 22rpx rgba(21, 22, 27, 0.1);
}

.fi-brand-nav__copy {
  min-width: 0;
  height: 64rpx;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 5rpx;
}

.fi-brand-nav__name {
  color: #15161b;
  font-size: 32rpx;
  font-weight: 900;
  line-height: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fi-brand-nav__tagline {
  color: #8f9198;
  font-size: 20rpx;
  font-weight: 500;
  line-height: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.fi-brand-nav__ai {
  position: fixed;
  top: calc(var(--fi-brand-nav-status-height) + 20rpx);
  right: 190rpx;
  z-index: 82;
  min-width: 122rpx;
  height: 58rpx;
  padding: 0 18rpx;
  border-radius: 999rpx;
  background: rgba(255, 255, 255, 0.2);
  border: 1rpx solid rgba(21, 22, 27, 0.12);
  color: #15161b;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8rpx;
  font-size: 22rpx;
  font-weight: 800;
  line-height: 1;
  box-shadow: none;
  backdrop-filter: blur(12rpx);
  -webkit-backdrop-filter: blur(12rpx);
}

.fi-brand-nav__ai::after {
  border: 0;
}

.fi-brand-nav__ai-dot {
  width: 9rpx;
  height: 9rpx;
  border-radius: 50%;
  background: #25d366;
  box-shadow: 0 0 0 6rpx rgba(37, 211, 102, 0.14);
}
</style>
