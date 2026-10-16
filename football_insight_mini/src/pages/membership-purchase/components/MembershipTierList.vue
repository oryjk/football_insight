<template>
  <view class="tier-list">
    <template v-for="group in groups" :key="group.key">
      <text v-if="group.title" class="tier-group__title">{{ group.title }}</text>
      <view
        v-for="option in group.options"
        :key="option.target_tier"
        class="tier-row"
        :class="{ 'tier-row--active': option.target_tier === selectedTier }"
        @tap="emit('select', option.target_tier)"
      >
        <view class="tier-row__radio">
          <view v-if="option.target_tier === selectedTier" class="tier-row__radio-dot" />
        </view>

        <view class="tier-row__main">
          <text class="tier-row__code">{{ formatMembershipTierLabel(option.target_tier) }}</text>
          <text class="tier-row__subtitle">{{ summaries[option.target_tier] ?? option.subtitle }}</text>
          <text v-if="isMembershipUpgradePrice(option)" class="tier-row__note">{{ formatMembershipUpgradeNote(option) }}</text>
        </view>

        <view class="tier-row__price">
          <view class="tier-row__price-line">
            <text class="tier-row__currency">¥</text>
            <text class="tier-row__amount">{{ formatMembershipPrice(getMembershipPayPriceCents(option)) }}</text>
          </view>
          <text v-if="isMembershipUpgradePrice(option)" class="tier-row__original">
            ¥{{ formatMembershipPrice(getMembershipOriginalPriceCents(option)) }}
          </text>
          <text v-else class="tier-row__duration">{{ formatMembershipDuration(option) }}</text>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MembershipProductOption } from '../../../types/payment'
import {
  formatMembershipDuration,
  formatMembershipPrice,
  formatMembershipTierLabel,
  formatMembershipUpgradeNote,
  getMembershipOriginalPriceCents,
  getMembershipPayPriceCents,
  isMembershipUpgradePrice,
  groupMembershipProductOptions,
} from '../helpers'

const props = defineProps<{
  options: MembershipProductOption[]
  selectedTier: string
  /** 档位 → 一句话描述（回流时间窗 · 刷新速度），缺省时用商品副标题。 */
  summaries: Record<string, string>
}>()

const groups = computed(() => groupMembershipProductOptions(props.options))

const emit = defineEmits<{
  (e: 'select', tier: string): void
}>()
</script>

<style scoped lang="css">
.tier-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
  margin-top: var(--fi-space-16);
}

.tier-group__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.tier-row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-20) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  border: var(--fi-primitive-border-width) solid var(--fi-color-divider);
  background: var(--fi-color-surface);
}

.tier-row--active {
  border-color: var(--fi-color-primary);
  background: rgba(var(--fi-primitive-red-rgb), 0.04);
}

.tier-row__radio {
  flex-shrink: 0;
  width: 32rpx;
  height: 32rpx;
  border-radius: var(--fi-radius-round);
  border: 3rpx solid var(--fi-color-divider);
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
}

.tier-row--active .tier-row__radio {
  border-color: var(--fi-color-primary);
}

.tier-row__radio-dot {
  width: 16rpx;
  height: 16rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
}

.tier-row__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.tier-row__code {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-tight);
}

.tier-row__subtitle {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-snug);
}

.tier-row__note {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
}

.tier-row__price {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--fi-space-4);
}

.tier-row__price-line {
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-2);
}

.tier-row__currency {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.tier-row__amount {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-40);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.tier-row--active .tier-row__currency,
.tier-row--active .tier-row__amount {
  color: var(--fi-color-primary);
}

.tier-row__original {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  text-decoration: line-through;
}

.tier-row__duration {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}
</style>
