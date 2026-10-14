<template>
  <FiPanel>
    <FiSectionHeading :kicker="`${code} 专属权益`" />

    <view class="privilege-grid">
      <view v-for="item in benefitItems" :key="item.key" class="privilege-card">
        <image class="privilege-card__icon" :src="benefitIconMap[item.iconName]" mode="aspectFit" />
        <view class="privilege-card__body">
          <text class="privilege-card__title">{{ item.title }}</text>
          <text class="privilege-card__caption">{{ item.caption }}</text>
        </view>
      </view>
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
import activityIcon from '../../../static/user/activity.svg'
import historyIcon from '../../../static/user/history.svg'
import radarIcon from '../../../static/user/radar.svg'
import ticketIcon from '../../../static/user/ticket.svg'
import type { MembershipBenefitItem } from '../../../utils/membershipBenefits'

defineProps<{
  code: string
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
.privilege-grid {
  margin-top: var(--fi-space-16);
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-12);
}

.privilege-card {
  min-width: 0;
  padding: var(--fi-space-18);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
}

.privilege-card__icon {
  flex-shrink: 0;
  width: 36rpx;
  height: 36rpx;
}

.privilege-card__body {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.privilege-card__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-tight);
}

.privilege-card__caption {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-tight);
}
</style>
