<template>
  <view class="panel recent-reflux-panel">
    <view class="section-heading section-heading--compact">
      <view class="recent-reflux-panel__heading">
        <view class="recent-reflux-panel__icon">
          <image
            class="recent-reflux-panel__icon-image"
            :src="radarIcon"
            mode="aspectFit"
          />
        </view>
        <text class="section-title">最近回流速览</text>
      </view>
      <text class="meta-pill">{{
        mode === 'unlocked' ? '分档可见' : 'V6 及以上'
      }}</text>
    </view>

    <view v-if="mode === 'unlocked'" class="recent-reflux-buckets">
      <view
        v-for="bucket in buckets"
        :key="bucket.key"
        class="recent-reflux-bucket"
        :class="[
          `recent-reflux-bucket--${bucket.key}`,
          { 'recent-reflux-bucket--locked': !isUnlocked(bucket.key) },
        ]"
      >
        <view class="recent-reflux-bucket__head">
          <view>
            <text class="recent-reflux-bucket__title">{{ bucket.title }}</text>
            <text class="recent-reflux-bucket__subtitle">{{
              bucket.subtitle
            }}</text>
          </view>
        </view>

        <view
          v-if="isUnlocked(bucket.key) && bucket.items.length"
          class="recent-reflux-list"
        >
          <view
            v-for="item in bucket.items.slice(0, 4)"
            :key="`recent-reflux-${bucket.key}-${item.block_key}`"
            class="recent-reflux-item"
            :class="[
              resolvePriceToneClass(item.price),
              `recent-reflux-item--${bucket.key}`,
            ]"
          >
            <view class="recent-reflux-item__beam" />
            <text class="recent-reflux-item__name">{{ item.block_name }}</text>
            <text class="recent-reflux-item__price">¥{{ item.price }}</text>
            <text class="recent-reflux-item__time">{{
              formatRecentRefluxMinuteLabel(item.minutes_ago)
            }}</text>
          </view>
        </view>

        <text
          v-else-if="isUnlocked(bucket.key)"
          class="recent-reflux-bucket__empty"
          >暂无</text
        >
        <view v-else class="recent-reflux-bucket-lock" @tap="emit('upgrade')">
          <view class="recent-reflux-bucket-lock__icon" aria-hidden="true">
            <view class="recent-reflux-bucket-lock__shackle" />
            <view class="recent-reflux-bucket-lock__body" />
          </view>
          <text class="recent-reflux-bucket-lock__tag"
            >{{ formatRecentRefluxBucketRequiredTier(bucket.key) }} 解锁</text
          >
          <text class="recent-reflux-bucket-lock__copy"
            >{{ bucket.title }}明细</text
          >
        </view>
      </view>
    </view>

    <TicketRecentRefluxLock v-else @upgrade="emit('upgrade')" />
  </view>
</template>

<script setup lang="ts">
import TicketRecentRefluxLock from './TicketRecentRefluxLock.vue'
import type {
  TicketWatchRecentRefluxBucket,
  TicketWatchRecentRefluxPanelMode,
  TicketWatchRecentRefluxBucketKey,
} from '../helpers'
import radarIcon from '../../../static/ticket-watch/radar.svg'
import { isRecentRefluxBucketUnlocked } from '../helpers'
import {
  formatRecentRefluxMinuteLabel,
  formatRecentRefluxBucketRequiredTier,
  resolvePriceToneClass,
} from '../presentation'
const props = defineProps<{
  mode: TicketWatchRecentRefluxPanelMode
  buckets: TicketWatchRecentRefluxBucket[]
  membershipTier: string
}>()
const emit = defineEmits<{ (e: 'upgrade'): void }>()
function isUnlocked(key: TicketWatchRecentRefluxBucketKey) {
  return isRecentRefluxBucketUnlocked(key, props.membershipTier)
}
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.price-tone--neutral {
  --fi-component-ticket-tone-rgb: var(
    --fi-color-ticket-price-tone-neutral-tone-rgb
  );
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-neutral-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-neutral-tone-text
  );
}

.price-tone--vip {
  --fi-component-ticket-tone-rgb: var(
    --fi-color-ticket-price-tone-vip-tone-rgb
  );
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-vip-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-vip-tone-text
  );
}

.price-tone--s {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-s-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-s-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-s-tone-text
  );
}

.price-tone--a {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-a-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-a-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-a-tone-text
  );
}

.price-tone--b {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-b-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-b-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-b-tone-text
  );
}

.price-tone--c {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-c-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-c-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-c-tone-text
  );
}

.price-tone--d {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-d-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-d-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-d-tone-text
  );
}

.price-tone--e {
  --fi-component-ticket-tone-rgb: var(--fi-color-ticket-price-tone-e-tone-rgb);
  --fi-component-ticket-tone-strong-rgb: var(
    --fi-color-ticket-price-tone-e-tone-strong-rgb
  );
  --fi-component-ticket-tone-text: var(
    --fi-color-ticket-price-tone-e-tone-text
  );
}

.recent-reflux-panel {
  position: relative;
  overflow: hidden;
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-history-trend-summary-item-background-2),
    var(--fi-color-ticket-recent-reflux-panel-background)
  );
  border-color: var(--fi-color-ticket-recent-reflux-panel-border-color);
  box-shadow: var(--fi-shadow-ticket-recent-reflux-panel);
}

.recent-reflux-panel__heading {
  display: grid;
  grid-template-columns: 34rpx minmax(0, 1fr);
  column-gap: var(--fi-space-8);
  align-items: center;
}

.recent-reflux-panel__heading .section-title {
  grid-column: 2;
}

.recent-reflux-panel__icon {
  grid-column: 1;
  align-self: center;
  width: 34rpx;
  height: 34rpx;
  border-radius: var(--fi-radius-ticket-12);
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-recent-reflux-panel-icon-background),
    var(--fi-color-ticket-recent-reflux-panel-icon-background-2)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-panel-icon-border);
}

.recent-reflux-panel__icon-image {
  width: 20rpx;
  height: 20rpx;
}

.recent-reflux-buckets {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-18);
}

.recent-reflux-bucket {
  min-width: 0;
  border-radius: var(--fi-radius-sm);
  padding: var(--fi-space-16) var(--fi-space-12) var(--fi-space-14);
  background: var(--fi-color-ticket-recent-reflux-bucket-background);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-bucket-border);
}

.recent-reflux-bucket--within3 {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-recent-reflux-bucket-within3-background),
    var(--fi-color-ticket-recent-reflux-bucket-within3-background-2)
  );
  border-color: var(
    --fi-color-ticket-recent-reflux-bucket-within3-border-color
  );
}

.recent-reflux-bucket--within10 {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-recent-reflux-bucket-within10-background),
    var(--fi-color-ticket-recent-reflux-bucket-within3-background-2)
  );
}

.recent-reflux-bucket--within30 {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-recent-reflux-bucket-within30-background),
    var(--fi-color-ticket-recent-reflux-bucket-within3-background-2)
  );
}

.recent-reflux-bucket--locked {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-recent-reflux-bucket-locked-background),
    var(--fi-color-ticket-recent-reflux-bucket-locked-background-2)
  );
  border-color: var(--fi-color-ticket-recent-reflux-bucket-locked-border-color);
}

.recent-reflux-bucket__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-8);
}

.recent-reflux-bucket__title {
  display: block;
  color: var(--fi-color-ticket-recent-reflux-lock-title-color);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.recent-reflux-bucket__subtitle,
.recent-reflux-bucket__empty {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-ticket-recent-reflux-bucket-subtitle-color);
  font-size: var(--fi-font-18);
  line-height: var(--fi-leading-tight);
}

.recent-reflux-bucket-lock {
  position: relative;
  overflow: hidden;
  margin-top: var(--fi-space-14);
  min-height: 82rpx;
  border-radius: var(--fi-radius-ticket-14);
  padding: var(--fi-space-12);
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: var(--fi-space-8);
  background: linear-gradient(
    135deg,
    var(--fi-color-ticket-recent-reflux-bucket-locked-background),
    var(--fi-color-ticket-recent-reflux-bucket-lock-background)
  );
  border: var(--fi-border-width) dashed
    var(--fi-color-ticket-recent-reflux-bucket-lock-border);
  box-sizing: border-box;
}

.recent-reflux-bucket-lock:active {
  transform: scale(0.985);
  background: linear-gradient(
    135deg,
    var(--fi-color-ticket-recent-reflux-bucket-lock-background-2),
    var(--fi-color-ticket-recent-reflux-bucket-lock-background-3)
  );
}

.recent-reflux-bucket-lock__icon {
  position: absolute;
  right: 12rpx;
  top: 14rpx;
  width: 34rpx;
  height: 34rpx;
  opacity: 0.72;
}

.recent-reflux-bucket-lock__shackle {
  position: absolute;
  left: 8rpx;
  top: 2rpx;
  width: 18rpx;
  height: 18rpx;
  border: var(--fi-border-width-4) solid
    var(--fi-color-ticket-recent-reflux-bucket-lock-shackle-border);
  border-bottom: 0;
  border-radius: var(--fi-radius-ticket-16) var(--fi-radius-ticket-16) 0 0;
  box-sizing: border-box;
}

.recent-reflux-bucket-lock__body {
  position: absolute;
  left: 4rpx;
  bottom: 2rpx;
  width: 26rpx;
  height: 20rpx;
  border-radius: var(--fi-radius-ticket-7);
  background: var(--fi-color-ticket-recent-reflux-bucket-lock-body-background);
  box-shadow: var(--fi-shadow-ticket-recent-reflux-bucket-lock-body);
}

.recent-reflux-bucket-lock__body::after {
  content: '';
  position: absolute;
  left: 11rpx;
  top: 6rpx;
  width: 4rpx;
  height: 8rpx;
  border-radius: var(--fi-radius-round);
  background: var(
    --fi-color-ticket-recent-reflux-bucket-lock-body-background-2
  );
}

.recent-reflux-bucket-lock__tag {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-self: flex-start;
  max-width: 100%;
  height: 30rpx;
  line-height: 30rpx;
  padding: 0 var(--fi-space-10);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-recent-reflux-bucket-lock-tag-background);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-17);
  font-weight: var(--fi-weight-extrabold);
  box-sizing: border-box;
}

.recent-reflux-bucket-lock__copy {
  position: relative;
  z-index: 1;
  display: block;
  color: var(--fi-color-ticket-recent-reflux-bucket-lock-copy-color);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-tight);
}

.recent-reflux-list {
  display: grid;
  gap: var(--fi-space-8);
  margin-top: var(--fi-space-14);
}

.recent-reflux-item {
  --recent-reflux-beam-rgb: var(
    --fi-color-ticket-recent-reflux-item-recent-reflux-beam-rgb
  );
  position: relative;
  overflow: hidden;
  min-width: 0;
  border-radius: var(--fi-radius-ticket-14);
  padding: var(--fi-space-10) var(--fi-space-10) var(--fi-space-9);
  background: rgba(var(--fi-component-ticket-tone-rgb), 0.08);
  border: var(--fi-border-width) solid
    rgba(var(--fi-component-ticket-tone-rgb), 0.16);
}

.recent-reflux-item--within3 {
  --recent-reflux-beam-rgb: var(
    --fi-color-ticket-recent-reflux-item-within3-recent-reflux-beam-rgb
  );
  border-color: var(--fi-color-ticket-recent-reflux-item-within3-border-color);
}

.recent-reflux-item--within10 {
  --recent-reflux-beam-rgb: var(
    --fi-color-ticket-recent-reflux-item-within10-recent-reflux-beam-rgb
  );
  border-color: var(--fi-color-ticket-recent-reflux-item-within10-border-color);
}

.recent-reflux-item--within30 {
  --recent-reflux-beam-rgb: var(
    --fi-color-ticket-recent-reflux-item-within30-recent-reflux-beam-rgb
  );
  border-color: var(--fi-color-ticket-recent-reflux-item-within30-border-color);
}

.recent-reflux-item__beam {
  position: absolute;
  z-index: 0;
  pointer-events: none;
  left: -72rpx;
  top: -3rpx;
  width: 72rpx;
  height: 6rpx;
  border-radius: var(--fi-radius-round);
  background: linear-gradient(
    90deg,
    transparent,
    rgba(var(--recent-reflux-beam-rgb), 0.95),
    transparent
  );
  --fi-component-ticket-beam-shadow: 0 0 var(--fi-space-18)
    rgba(var(--recent-reflux-beam-rgb), 0.38);
  box-shadow: var(--fi-component-ticket-beam-shadow);
  animation: recent-reflux-item-border-beam 3.8s linear infinite;
}

.recent-reflux-item__name {
  display: block;
  color: var(--fi-component-ticket-tone-text);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  position: relative;
  z-index: 1;
}

.recent-reflux-item__price,
.recent-reflux-item__time {
  display: block;
  margin-top: var(--fi-space-6);
  font-size: var(--fi-font-16);
  line-height: var(--fi-leading-none);
  position: relative;
  z-index: 1;
}

.recent-reflux-item__price {
  color: rgba(var(--fi-component-ticket-tone-strong-rgb), 0.72);
  font-weight: var(--fi-weight-bold);
}

.recent-reflux-item__time {
  color: var(--fi-color-ticket-recent-reflux-bucket-lock-copy-color);
}

@keyframes recent-reflux-item-border-beam {
  0% {
    left: -72rpx;
    top: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  34% {
    left: 100%;
    top: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  35% {
    left: auto;
    right: -3rpx;
    top: -54rpx;
    width: 6rpx;
    height: 54rpx;
  }
  50% {
    left: auto;
    right: -3rpx;
    top: 100%;
    width: 6rpx;
    height: 54rpx;
  }
  51% {
    right: -72rpx;
    left: auto;
    top: auto;
    bottom: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  84% {
    right: 100%;
    left: auto;
    top: auto;
    bottom: -3rpx;
    width: 72rpx;
    height: 6rpx;
  }
  85% {
    right: auto;
    left: -3rpx;
    bottom: -54rpx;
    width: 6rpx;
    height: 54rpx;
  }
  100% {
    right: auto;
    left: -3rpx;
    bottom: 100%;
    width: 6rpx;
    height: 54rpx;
  }
}
</style>
