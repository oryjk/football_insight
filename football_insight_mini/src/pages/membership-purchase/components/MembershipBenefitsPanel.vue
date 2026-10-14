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

  </view>
</template>

<script setup lang="ts">
import activityIcon from '../../../static/user/activity.svg'
import historyIcon from '../../../static/user/history.svg'
import radarIcon from '../../../static/user/radar.svg'
import ticketIcon from '../../../static/user/ticket.svg'
import type { MembershipBenefitItem } from '../../../utils/membershipBenefits'

defineProps<{
  code: string
  toneClass: string
  benefitItems: MembershipBenefitItem[]
}>()

const benefitIconMap: Record<MembershipBenefitItem['iconName'], string> = {
  ticket: ticketIcon,
  activity: activityIcon,
  radar: radarIcon,
  history: historyIcon,
}
</script>

<style scoped lang="css">
.panel {
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

.privilege-panel {
  padding: 22rpx;
  border-radius: var(--fi-radius-lg);
}

.privilege-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.privilege-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 160ms both;
}

.privilege-panel__title {
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
</style>
