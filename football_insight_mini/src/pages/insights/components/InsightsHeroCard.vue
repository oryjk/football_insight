<template>
  <view class="hero-card">
    <image class="hero-dots hero-dots--top" :src="memberCardDotsImage" mode="aspectFill" />
    <image class="hero-dots hero-dots--bottom" :src="memberCardDotsImage" mode="aspectFill" />
    <view class="hero-card__top">
      <view>
        <text class="eyebrow">Insights</text>
        <text class="hero-card__title">洞察不是榜单，而是贡献结构</text>
      </view>
      <text class="meta-note">球队洞察</text>
    </view>

    <text class="hero-card__summary">
      先按球队看赛季累计贡献：这个队的进球主要打在谁身上，谁在本队内部贡献了更多进球和助攻，以及失球主要来自哪些对手。
    </text>
  </view>
</template>

<script setup lang="ts">
import memberCardDotsImage from '../../../static/user/member-card-dots.png'
</script>

<style scoped lang="css">
.hero-card {
  position: relative;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.hero-dots {
  position: absolute;
  z-index: 0;
  width: 292rpx;
  height: 180rpx;
  opacity: 0.34;
  pointer-events: none;
}

.hero-dots--top {
  top: -52rpx;
  right: -54rpx;
}

.hero-dots--bottom {
  left: -84rpx;
  bottom: -42rpx;
  opacity: 0.16;
  transform: scaleX(-1);
}

.hero-card__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
  position: relative;
  z-index: 1;
}

.eyebrow {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.hero-card__title {
  display: block;
  margin-top: var(--fi-space-10);
  color: #2a2c31;
  font-size: var(--fi-font-48);
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 1rpx;
  padding: var(--fi-space-14) 0 0;
}

.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.12);
}

.hero-card__summary {
  position: relative;
  z-index: 1;
  display: block;
  margin-top: var(--fi-space-18);
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
}
</style>
