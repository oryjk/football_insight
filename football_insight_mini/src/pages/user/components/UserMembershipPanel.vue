<template>
  <view>
    <view class="panel privilege-panel" :class="toneClass">
      <view class="privilege-panel__header">
        <view class="privilege-panel__title-row">
          <text class="privilege-panel__title">{{ code }} 专属权益</text>
        </view>
      </view>

      <view class="privilege-grid">
        <view v-for="item in benefitItems" :key="item.key" class="privilege-card">
          <view class="privilege-card__icon">
            <image class="privilege-card__icon-image" :src="benefitIconMap[item.iconName]" mode="aspectFit" />
          </view>
          <view class="privilege-card__body">
            <text class="privilege-card__title">{{ item.title }}</text>
            <text class="privilege-card__caption">{{ item.caption }}</text>
          </view>
        </view>
      </view>
    </view>

    <view class="panel upgrade-panel">
      <view class="upgrade-panel__title-row">
        <image class="upgrade-panel__info-icon" :src="infoIcon" mode="aspectFit" />
        <text class="upgrade-panel__title">升级说明</text>
      </view>
      <text class="upgrade-panel__body">{{ ruleSummary }}</text>
      <view v-if="upgradeSteps.length" class="upgrade-step-list">
        <view
          v-for="step in upgradeSteps"
          :key="step.key"
          class="tier-ladder-card tier-ladder-card--compact"
          :class="[step.toneClass, { 'tier-ladder-card--active': step.isCurrent }]"
        >
          <view class="tier-ladder-card__top">
            <view class="tier-ladder-card__heading">
              <view class="upgrade-step-card__title-row">
                <text class="tier-ladder-card__code">{{ step.code }}</text>
                <text class="tier-ladder-card__name">{{ step.name }}</text>
              </view>
              <text class="tier-ladder-card__condition">{{ step.condition }}</text>
            </view>

            <view class="upgrade-step-card__aside">
              <text class="tier-ladder-card__refresh">{{ step.refreshLabel }}</text>
              <text v-if="step.isCurrent" class="upgrade-step-card__current">当前等级</text>
            </view>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import infoIcon from '../../../static/user/info.svg'
import activityIcon from '../../../static/user/activity.svg'
import historyIcon from '../../../static/user/history.svg'
import radarIcon from '../../../static/user/radar.svg'
import ticketIcon from '../../../static/user/ticket.svg'
import type { UserBenefitItem, UserUpgradeStep } from '../helpers'

defineProps<{
  code: string
  toneClass: string
  benefitItems: UserBenefitItem[]
  upgradeSteps: UserUpgradeStep[]
  ruleSummary: string
}>()

const benefitIconMap: Record<UserBenefitItem['iconName'], string> = {
  ticket: ticketIcon,
  activity: activityIcon,
  radar: radarIcon,
  history: historyIcon,
}
</script>

<style scoped lang="css">
.hero-card, .panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: var(--fi-radius-xl);
  padding: 20rpx;
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.skeleton-line,
.skeleton-pill,
.skeleton-button,
.skeleton-account-cell,
.skeleton-privilege-card {
  background: linear-gradient(180deg, rgba(248, 246, 239, 0.96), rgba(235, 230, 216, 0.9));
  border: 2rpx solid rgba(232, 222, 198, 0.72);
}

.skeleton-privilege-grid {
  margin-top: 24rpx;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12rpx;
}

.skeleton-privilege-card {
  min-height: 150rpx;
  padding: 18rpx 12rpx;
  border-radius: var(--fi-radius-md);
  display: grid;
  justify-items: center;
  align-content: center;
  gap: 12rpx;
}

.skeleton-line--privilege-icon {
  width: 52rpx;
  height: 52rpx;
  border-radius: var(--fi-radius-round);
}

.skeleton-line--privilege-title {
  width: 82rpx;
  height: 26rpx;
}

.skeleton-line--privilege-caption {
  width: 98rpx;
  height: 22rpx;
}

.tier-ladder-panel {
  margin-top: 24rpx;
}

.tier-ladder-list {
  margin-top: 20rpx;
  display: grid;
  gap: 14rpx;
}

.tier-ladder-card {
  border-radius: var(--fi-radius-lg);
  padding: 22rpx;
  border: 2rpx solid var(--page-border);
  background: linear-gradient(180deg, rgba(255,255,255,0.98), rgba(246,248,252,0.98));
}

.tier-ladder-card.tier-tone--v1 {
  --tier-pulse: rgba(127, 144, 170, 0.14);
  border-color: rgba(205, 214, 231, 0.95);
  background: linear-gradient(180deg, rgba(249, 251, 255, 0.98), rgba(238, 244, 252, 0.98));
}

.tier-ladder-card.tier-tone--v2 {
  --tier-pulse: rgba(106, 158, 142, 0.14);
  border-color: rgba(170, 205, 190, 0.95);
  background: linear-gradient(180deg, rgba(244, 252, 248, 0.98), rgba(232, 244, 238, 0.98));
}

.tier-ladder-card.tier-tone--v3 {
  --tier-pulse: rgba(220, 38, 38, 0.14);
  border-color: rgba(254, 205, 211, 0.95);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 226, 226, 0.98));
}

.tier-ladder-card.tier-tone--v4 {
  --tier-pulse: rgba(207, 105, 78, 0.16);
  border-color: rgba(233, 191, 179, 0.95);
  background: linear-gradient(180deg, rgba(255, 250, 248, 0.98), rgba(250, 235, 229, 0.98));
}

.tier-ladder-card.tier-tone--v5 {
  --tier-pulse: rgba(142, 91, 199, 0.16);
  border-color: rgba(214, 197, 239, 0.95);
  background: linear-gradient(180deg, rgba(251, 248, 255, 0.98), rgba(242, 235, 252, 0.98));
}

.tier-ladder-card.tier-tone--v6 {
  --tier-pulse: rgba(61, 156, 114, 0.16);
  border-color: rgba(181, 223, 203, 0.95);
  background: linear-gradient(180deg, rgba(247, 253, 250, 0.98), rgba(232, 247, 240, 0.98));
}

.tier-ladder-card.tier-tone--v7 {
  --tier-pulse: rgba(59, 120, 196, 0.16);
  border-color: rgba(192, 211, 239, 0.95);
  background: linear-gradient(180deg, rgba(247, 250, 255, 0.98), rgba(233, 242, 252, 0.98));
}

.tier-ladder-card.tier-tone--v8 {
  --tier-pulse: rgba(203, 84, 77, 0.16);
  border-color: rgba(236, 193, 190, 0.95);
  background: linear-gradient(180deg, rgba(255, 249, 249, 0.98), rgba(250, 235, 235, 0.98));
}

.tier-ladder-card.tier-tone--v9 {
  --tier-pulse: rgba(185, 28, 28, 0.16);
  border-color: rgba(252, 165, 165, 0.95);
  background: linear-gradient(180deg, rgba(255, 241, 242, 0.98), rgba(254, 202, 202, 0.72));
}

.tier-ladder-card.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(220, 38, 38, 0.18);
  transform: translateY(-2rpx);
}

.tier-ladder-card.tier-tone--v1.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(127, 144, 170, 0.22);
}

.tier-ladder-card.tier-tone--v2.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(106, 158, 142, 0.22);
}

.tier-ladder-card.tier-tone--v3.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(220, 38, 38, 0.18);
}

.tier-ladder-card.tier-tone--v4.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(207, 105, 78, 0.22);
}

.tier-ladder-card.tier-tone--v5.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(142, 91, 199, 0.22);
}

.tier-ladder-card.tier-tone--v6.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(61, 156, 114, 0.22);
}

.tier-ladder-card.tier-tone--v7.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(59, 120, 196, 0.22);
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--active {
  box-shadow: 0 16rpx 44rpx rgba(203, 84, 77, 0.22);
}

.tier-ladder-card.tier-tone--v9.tier-ladder-card--active {
  box-shadow: 0 16rpx 56rpx rgba(190, 130, 0, 0.65) !important;
}

.tier-ladder-card__top {
  display: flex;
  align-items: center;
  gap: 18rpx;
}

.tier-ladder-card__heading {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6rpx;
}

.tier-ladder-card__code {
  color: #14161c;
  font-size: var(--fi-font-30);
  font-weight: 800;
  line-height: 1;
}

.tier-ladder-card__name {
  color: #7a808d;
  font-size: var(--fi-font-22);
  font-weight: 700;
}

.tier-ladder-card__refresh {
  align-self: flex-start;
  padding: 10rpx 14rpx;
  border-radius: var(--fi-radius-round);
  background: rgba(255,255,255,0.78);
  border: 2rpx solid rgba(237, 230, 214, 0.94);
  color: #76684b;
  font-size: var(--fi-font-22);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.tier-ladder-card__condition,
.tier-ladder-card__body {
  display: block;
  margin-top: 14rpx;
  color: #1c1f25;
  font-size: var(--fi-font-24);
  line-height: 1.65;
}

.tier-ladder-card__body {
  margin-top: 8rpx;
  color: #6a707d;
}

.tier-ladder-panel__footnote {
  display: block;
  margin-top: 16rpx;
  color: #8a8f9a;
  font-size: var(--fi-font-22);
  line-height: 1.6;
}

.info-panel,
.privilege-panel,
.upgrade-panel {
  padding: 22rpx;
  border-radius: var(--fi-radius-lg);
}

.info-panel,
.privilege-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.privilege-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 160ms both;
}

.upgrade-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 240ms both;
}

.info-panel__title,
.privilege-panel__title,
.upgrade-panel__title {
  color: #1b1c20;
  font-size: var(--fi-font-30);
  font-weight: 800;
  line-height: 1;
}

.privilege-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}

.privilege-panel__title-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
}

.privilege-grid {
  margin-top: 22rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12rpx;
}

.privilege-card {
  min-width: 0;
  padding: 18rpx;
  border-radius: 22rpx;
  border: 1rpx solid rgba(232, 235, 241, 0.98);
  background: rgba(248, 249, 251, 0.78);
  display: flex;
  align-items: center;
  gap: 14rpx;
  text-align: left;
}

.privilege-card__icon {
  grid-area: icon;
  width: 38rpx;
  height: 38rpx;
  display: flex;
  align-items: center;
  justify-content: center;
}

.privilege-card__icon-image {
  width: 36rpx;
  height: 36rpx;
}

.privilege-card__body {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 10rpx;
}

.privilege-card__title {
  display: block;
  color: #1e2025;
  font-size: 25rpx;
  font-weight: 800;
  line-height: 1.15;
}

.privilege-card__caption {
  display: block;
  color: #8a8f9b;
  font-size: var(--fi-font-22);
  line-height: 1;
}

.upgrade-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.upgrade-panel__title-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.upgrade-panel__info-icon {
  width: 36rpx;
  height: 36rpx;
  flex-shrink: 0;
}

.upgrade-panel__body {
  display: block;
  margin-top: 16rpx;
  color: #737986;
  font-size: var(--fi-font-24);
  line-height: 1.7;
}

.upgrade-step-list {
  margin-top: 22rpx;
  display: grid;
  gap: 14rpx;
}

.tier-ladder-card--compact {
  padding: 20rpx 22rpx;
  border-radius: var(--fi-radius-md);
  border: 0 !important;
  box-shadow: none;
  transform: none;
}

.tier-ladder-card.tier-ladder-card--compact.tier-ladder-card--active {
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v1.tier-ladder-card--compact {
  background: #f3f5f8 !important;
}

.tier-ladder-card.tier-tone--v2.tier-ladder-card--compact {
  background: #edf2f3 !important;
}

.tier-ladder-card.tier-tone--v3.tier-ladder-card--compact {
  background: #f1eadc !important;
}

.tier-ladder-card.tier-tone--v4.tier-ladder-card--compact {
  background: #eadfd9 !important;
}

.tier-ladder-card.tier-tone--v5.tier-ladder-card--compact {
  background: #ded8e8 !important;
}

.tier-ladder-card.tier-tone--v6.tier-ladder-card--compact {
  background: #d5e2da !important;
}

.tier-ladder-card.tier-tone--v7.tier-ladder-card--compact {
  background: #cad8e9 !important;
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact {
  background: #c95656 !important;
}

.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact {
  background: #9e2f2f !important;
}

.tier-ladder-card.tier-tone--v1.tier-ladder-card--compact.tier-ladder-card--active {
  background: #e6ebf1 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v2.tier-ladder-card--compact.tier-ladder-card--active {
  background: #dde8e6 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v3.tier-ladder-card--compact.tier-ladder-card--active {
  background: #e6dcc7 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v4.tier-ladder-card--compact.tier-ladder-card--active {
  background: #dfcfc7 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v5.tier-ladder-card--compact.tier-ladder-card--active {
  background: #d0c8df !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v6.tier-ladder-card--compact.tier-ladder-card--active {
  background: #c6d8cc !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v7.tier-ladder-card--compact.tier-ladder-card--active {
  background: #b8cae1 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact.tier-ladder-card--active {
  background: #b94141 !important;
  box-shadow: none;
}

.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact.tier-ladder-card--active {
  background: #842525 !important;
  box-shadow: none !important;
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__code,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__name,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__condition,
.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__refresh,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__code,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__name,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__condition,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__refresh {
  color: rgba(255, 255, 255, 0.94);
}

.tier-ladder-card.tier-tone--v8.tier-ladder-card--compact .tier-ladder-card__refresh,
.tier-ladder-card.tier-tone--v9.tier-ladder-card--compact .tier-ladder-card__refresh {
  background: rgba(255, 255, 255, 0.94);
  border-color: transparent;
  color: #8f2525;
}

.tier-ladder-card--compact .tier-ladder-card__top {
  align-items: flex-start;
}

.tier-ladder-card--compact .tier-ladder-card__condition {
  margin-top: 10rpx;
  font-size: var(--fi-font-22);
  line-height: 1.6;
}

.upgrade-step-card__title-row {
  display: flex;
  align-items: center;
  gap: 10rpx;
  flex-wrap: wrap;
}

.upgrade-step-card__aside {
  display: grid;
  justify-items: end;
  gap: 10rpx;
}

.upgrade-step-card__current {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8rpx 14rpx;
  border-radius: var(--fi-radius-round);
  background: #15171d;
  border: 0;
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-20);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}
</style>
