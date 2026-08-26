<template>
  <view class="guest-chant-wall">
    <view class="guest-chant-wall__meta">
      <text class="guest-chant-wall__kicker">MATCHDAY AT PHOENIX HILL</text>
      <text class="guest-chant-wall__title">成都主场的歌声</text>
    </view>

    <view class="guest-chant-wall__lyrics">
      <view class="guest-chant-wall__fade guest-chant-wall__fade--top"></view>
      <view class="guest-chant-wall__fade guest-chant-wall__fade--bottom"></view>
      <view class="guest-chant-wall__track">
        <text
          v-for="(line, index) in loopLines"
          :key="`guest-chant-${index}`"
          class="guest-chant-wall__line"
          :class="{ 'guest-chant-wall__line--focus': index % lines.length === 2 }"
        >
          {{ line }}
        </text>
      </view>
    </view>

    <view class="guest-chant-wall__meter">
      <view
        v-for="(height, index) in meterBars"
        :key="`guest-chant-meter-${index}`"
        class="guest-chant-wall__bar"
        :style="{ height: `${height}rpx`, animationDelay: `${index * -0.13}s` }"
      ></view>
    </view>
  </view>
</template>

<script setup lang="ts">
const lines = [
  '凤凰山的灯光亮起',
  '红色看台一起呼吸',
  '为成都喊到终场',
  '每一次进攻都有人相信',
  '主场的风吹过球衣',
  '今晚继续并肩向前',
]
const loopLines = [...lines, ...lines]
const meterBars = [18, 34, 24, 48, 30, 62, 38, 74, 44, 58, 32, 46, 26, 36]
</script>

<style scoped lang="css">
@keyframes guest-chant-scroll {
  from { transform: translateY(0); }
  to { transform: translateY(-50%); }
}

@keyframes guest-chant-pulse {
  0%, 100% {
    opacity: 0.38;
    transform: scaleY(0.56);
  }
  45% {
    opacity: 0.92;
    transform: scaleY(1);
  }
  72% {
    opacity: 0.56;
    transform: scaleY(0.74);
  }
}

.guest-chant-wall {
  position: fixed;
  left: 44rpx;
  right: 44rpx;
  bottom: calc(178rpx + env(safe-area-inset-bottom));
  height: 520rpx;
  z-index: 30;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  pointer-events: none;
}

.guest-chant-wall__meta {
  margin-bottom: 28rpx;
  text-align: center;
  text-shadow: 0 10rpx 26rpx rgba(0, 0, 0, 0.45);
}

.guest-chant-wall__kicker {
  display: block;
  color: rgba(255, 255, 255, 0.54);
  font-size: var(--fi-font-20);
  font-weight: 800;
  letter-spacing: 4rpx;
}

.guest-chant-wall__title {
  display: block;
  margin-top: 10rpx;
  color: rgba(255, 255, 255, 0.94);
  font-size: 42rpx;
  font-weight: 900;
  line-height: 1.15;
}

.guest-chant-wall__lyrics {
  position: relative;
  height: 338rpx;
  overflow: hidden;
}

.guest-chant-wall__track {
  display: flex;
  flex-direction: column;
  align-items: center;
  animation: guest-chant-scroll 20s linear infinite;
}

.guest-chant-wall__line {
  display: block;
  width: 100%;
  height: 68rpx;
  color: rgba(255, 255, 255, 0.42);
  font-size: 31rpx;
  font-weight: 700;
  line-height: 68rpx;
  text-align: center;
  text-shadow: 0 8rpx 24rpx rgba(0, 0, 0, 0.42);
}

.guest-chant-wall__line--focus {
  color: rgba(255, 255, 255, 0.94);
  font-size: 39rpx;
  font-weight: 900;
}

.guest-chant-wall__fade {
  position: absolute;
  left: 0;
  right: 0;
  height: 92rpx;
  z-index: 1;
  pointer-events: none;
}

.guest-chant-wall__fade--top {
  top: 0;
  background: linear-gradient(180deg, rgba(17, 23, 15, 0.84), rgba(17, 23, 15, 0));
}

.guest-chant-wall__fade--bottom {
  bottom: 0;
  background: linear-gradient(0deg, rgba(17, 23, 15, 0.72), rgba(17, 23, 15, 0));
}

.guest-chant-wall__meter {
  height: 88rpx;
  margin: 18rpx auto 0;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 10rpx;
  opacity: 0.52;
}

.guest-chant-wall__bar {
  width: 8rpx;
  border-radius: var(--fi-radius-round);
  background: rgba(255, 255, 255, 0.64);
  box-shadow: 0 0 18rpx rgba(255, 255, 255, 0.18);
  transform-origin: bottom center;
  animation: guest-chant-pulse 1.45s ease-in-out infinite;
}
</style>
