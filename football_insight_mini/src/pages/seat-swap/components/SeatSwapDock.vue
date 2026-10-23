<template>
  <view class="dock">
    <button v-if="!hasMyRequest" class="dock-cta" hover-class="dock-cta--pressed" @tap="emit('cta')">
      <text class="dock-cta__icon">+</text>
      <text class="dock-cta__label">{{ isLoggedIn ? '发布我的换座' : '登录后发布' }}</text>
    </button>
    <view v-else class="dock-status" @tap="emit('manage')">
      <text v-if="interestedCount > 0" class="dock-status__badge">{{ formatSeatSwapBadgeCount(interestedCount) }}</text>
      <view class="dock-status__body">
        <!-- lucide armchair：标明这一条是"我的座位"发布 -->
        <image class="dock-status__icon" src="/static/seat-swap/armchair-ink.svg" mode="aspectFit" aria-hidden="true" />
        <text class="dock-status__seat dock-status__seat--current">{{ currentSeatLabel }}</text>
        <text class="dock-status__arrow">→</text>
        <text class="dock-status__seat dock-status__seat--desired">{{ desiredSummary }}</text>
        <text class="dock-status__manage">管理 ›</text>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { formatSeatSwapBadgeCount } from '../helpers'

defineProps<{
  hasMyRequest: boolean
  isLoggedIn: boolean
  currentSeatLabel: string
  desiredSummary: string
  /** 已确认想和我换座、等我确认的人数，> 0 时在右上角显示红色气泡 */
  interestedCount: number
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
  /* 整页唯一的主操作用品牌红；卡片内操作为描边胶囊、球场图浮窗为白底，层级逐级减弱。 */
  background: var(--fi-color-primary);
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
  background: rgba(var(--fi-primitive-white-rgb), 0.22);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
}

.dock-cta__label {
  font-size: var(--fi-font-28);
}

.dock-status {
  position: relative;
  display: flex;
  align-items: center;
  min-height: var(--fi-component-seat-swap-dock-height);
  box-sizing: border-box;
  padding: var(--fi-space-20) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  /* 已发布后是状态条而非主操作：白底承载红/绿座位块；唯一的操作「管理」用品牌红。 */
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  color: var(--fi-color-text-strong);
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

.dock-status__icon {
  flex: 0 0 auto;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
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
  color: var(--fi-color-text-caption);
}

.dock-status__badge {
  position: absolute;
  top: -14rpx;
  right: -8rpx;
  min-width: 36rpx;
  height: 36rpx;
  padding: 0 var(--fi-space-8);
  box-sizing: border-box;
  border-radius: var(--fi-radius-round);
  border: 3rpx solid var(--fi-primitive-white);
  background: var(--fi-color-primary);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: 30rpx;
  text-align: center;
}

.dock-status__manage {
  flex: 0 0 auto;
  margin-left: auto;
  padding: var(--fi-space-12) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-component-btn-bg);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
}
</style>
