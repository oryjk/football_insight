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

.account-hero.tier-tone--v1,
.privilege-panel.tier-tone--v1 {
  --member-highlight: rgba(206, 220, 246, 0.9);
  --member-wash: rgba(228, 236, 250, 0.76);
  --member-border: rgba(197, 209, 232, 0.94);
  --member-eyebrow: #66799a;
  --member-accent: #7f95bd;
  --member-accent-strong: #5d749c;
  --member-badge-from: #90a5ca;
  --member-badge-to: #697fa7;
  --member-soft-surface: rgba(236, 242, 252, 0.84);
  --member-soft-border: rgba(203, 214, 235, 0.94);
  --member-medal-ink: #6a82ab;
  --member-medal-crown: rgba(120, 144, 181, 0.7);
  --member-glow: rgba(123, 148, 193, 0.2);
  --member-privilege-border: rgba(212, 220, 239, 0.84);
  --member-privilege-bg: rgba(239, 244, 252, 0.96);
  --member-privilege-icon-bg: rgba(121, 146, 189, 0.14);
  --member-privilege-icon-color: #6b82ab;
}

.account-hero.tier-tone--v3,
.privilege-panel.tier-tone--v3 {
  --member-highlight: rgba(255, 232, 179, 0.92);
  --member-wash: rgba(255, 243, 214, 0.68);
  --member-border: rgba(235, 211, 155, 0.92);
  --member-eyebrow: #b91c1c;
  --member-accent: #dc2626;
  --member-accent-strong: #991b1b;
  --member-badge-from: #ef4444;
  --member-badge-to: #b91c1c;
  --member-soft-surface: rgba(255, 241, 242, 0.72);
  --member-soft-border: rgba(254, 205, 211, 0.94);
  --member-medal-ink: #b91c1c;
  --member-medal-crown: rgba(220, 38, 38, 0.62);
  --member-glow: rgba(220, 38, 38, 0.18);
  --member-privilege-border: rgba(254, 205, 211, 0.76);
  --member-privilege-bg: rgba(255, 241, 242, 0.88);
  --member-privilege-icon-bg: rgba(220, 38, 38, 0.1);
  --member-privilege-icon-color: #b91c1c;
}

.account-hero.tier-tone--v4,
.privilege-panel.tier-tone--v4 {
  --member-highlight: rgba(254, 202, 202, 0.92);
  --member-wash: rgba(255, 235, 229, 0.74);
  --member-border: rgba(236, 188, 173, 0.92);
  --member-eyebrow: #ab5a4d;
  --member-accent: #d47a5d;
  --member-accent-strong: #af5843;
  --member-badge-from: #de8769;
  --member-badge-to: #b55c45;
  --member-soft-surface: rgba(255, 241, 236, 0.78);
  --member-soft-border: rgba(239, 200, 187, 0.92);
  --member-medal-ink: #c55f45;
  --member-medal-crown: rgba(205, 102, 78, 0.72);
  --member-glow: rgba(213, 122, 93, 0.22);
  --member-privilege-border: rgba(238, 205, 195, 0.8);
  --member-privilege-bg: rgba(252, 239, 235, 0.96);
  --member-privilege-icon-bg: rgba(212, 122, 93, 0.12);
  --member-privilege-icon-color: #c06049;
}

.account-hero.tier-tone--v5,
.privilege-panel.tier-tone--v5 {
  --member-highlight: rgba(228, 214, 255, 0.92);
  --member-wash: rgba(243, 236, 255, 0.76);
  --member-border: rgba(211, 194, 239, 0.92);
  --member-eyebrow: #7c58b2;
  --member-accent: #9468cf;
  --member-accent-strong: #7045ac;
  --member-badge-from: #a27ddd;
  --member-badge-to: #754db5;
  --member-soft-surface: rgba(244, 238, 255, 0.8);
  --member-soft-border: rgba(217, 203, 240, 0.94);
  --member-medal-ink: #8458c3;
  --member-medal-crown: rgba(139, 93, 199, 0.72);
  --member-glow: rgba(148, 104, 207, 0.22);
  --member-privilege-border: rgba(221, 208, 244, 0.82);
  --member-privilege-bg: rgba(246, 241, 255, 0.96);
  --member-privilege-icon-bg: rgba(148, 104, 207, 0.12);
  --member-privilege-icon-color: #8257c1;
}

.account-hero.tier-tone--v6,
.privilege-panel.tier-tone--v6 {
  --member-highlight: rgba(205, 245, 223, 0.92);
  --member-wash: rgba(231, 251, 240, 0.76);
  --member-border: rgba(178, 223, 200, 0.92);
  --member-eyebrow: #2f8b65;
  --member-accent: #47b283;
  --member-accent-strong: #2d8c63;
  --member-badge-from: #55bf8f;
  --member-badge-to: #2d8b63;
  --member-soft-surface: rgba(236, 252, 243, 0.8);
  --member-soft-border: rgba(189, 228, 207, 0.94);
  --member-medal-ink: #36956d;
  --member-medal-crown: rgba(61, 156, 114, 0.72);
  --member-glow: rgba(71, 178, 131, 0.22);
  --member-privilege-border: rgba(196, 231, 213, 0.82);
  --member-privilege-bg: rgba(240, 253, 246, 0.96);
  --member-privilege-icon-bg: rgba(71, 178, 131, 0.12);
  --member-privilege-icon-color: #368f69;
}

.account-hero.tier-tone--v7,
.privilege-panel.tier-tone--v7 {
  --member-highlight: rgba(209, 226, 255, 0.92);
  --member-wash: rgba(233, 242, 255, 0.76);
  --member-border: rgba(187, 209, 241, 0.92);
  --member-eyebrow: #3a69b2;
  --member-accent: #4c84d7;
  --member-accent-strong: #315fa7;
  --member-badge-from: #5c93e3;
  --member-badge-to: #3465b3;
  --member-soft-surface: rgba(238, 245, 255, 0.8);
  --member-soft-border: rgba(193, 214, 243, 0.94);
  --member-medal-ink: #3e70c0;
  --member-medal-crown: rgba(59, 120, 196, 0.72);
  --member-glow: rgba(76, 132, 215, 0.22);
  --member-privilege-border: rgba(201, 219, 244, 0.82);
  --member-privilege-bg: rgba(239, 245, 255, 0.96);
  --member-privilege-icon-bg: rgba(76, 132, 215, 0.12);
  --member-privilege-icon-color: #3f73c2;
}

.account-hero.tier-tone--v8,
.privilege-panel.tier-tone--v8 {
  --member-highlight: rgba(255, 212, 220, 0.92);
  --member-wash: rgba(255, 236, 239, 0.76);
  --member-border: rgba(237, 190, 198, 0.92);
  --member-eyebrow: #ad4b58;
  --member-accent: #d36576;
  --member-accent-strong: #b14353;
  --member-badge-from: #db7384;
  --member-badge-to: #b44757;
  --member-soft-surface: rgba(255, 239, 242, 0.8);
  --member-soft-border: rgba(239, 200, 207, 0.94);
  --member-medal-ink: #c54f62;
  --member-medal-crown: rgba(203, 84, 77, 0.72);
  --member-glow: rgba(211, 101, 118, 0.22);
  --member-privilege-border: rgba(239, 206, 213, 0.82);
  --member-privilege-bg: rgba(255, 241, 244, 0.96);
  --member-privilege-icon-bg: rgba(211, 101, 118, 0.12);
  --member-privilege-icon-color: #c24f61;
}

.account-hero.tier-tone--v9,
.privilege-panel.tier-tone--v9 {
  --member-highlight: rgba(254, 202, 202, 0.94);
  --member-wash: rgba(254, 226, 226, 0.78);
  --member-border: rgba(254, 205, 211, 0.94);
  --member-eyebrow: #b91c1c;
  --member-accent: #dc2626;
  --member-accent-strong: #991b1b;
  --member-badge-from: #ef4444;
  --member-badge-to: #b91c1c;
  --member-soft-surface: rgba(255, 241, 242, 0.84);
  --member-soft-border: rgba(254, 205, 211, 0.96);
  --member-medal-ink: #b91c1c;
  --member-medal-crown: rgba(220, 38, 38, 0.62);
  --member-glow: rgba(220, 38, 38, 0.18);
  --member-privilege-border: rgba(254, 205, 211, 0.84);
  --member-privilege-bg: rgba(255, 241, 242, 0.92);
  --member-privilege-icon-bg: rgba(220, 38, 38, 0.1);
  --member-privilege-icon-color: #b91c1c;
}
</style>
