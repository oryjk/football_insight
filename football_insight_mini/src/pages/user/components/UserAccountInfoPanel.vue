<template>
  <view class="account-info-stack">
    <view class="panel info-panel">
      <view class="info-panel__header">
        <text class="info-panel__title">账户信息</text>
      </view>

      <view class="account-info-list">
        <view v-for="item in items" :key="item.key" class="account-info-item">
          <image class="account-info-item__icon" :src="iconMap[item.iconName]" mode="aspectFit" />
          <text class="account-info-item__label">{{ item.label }}</text>
          <text class="account-info-item__value">{{ item.value }}</text>
        </view>
      </view>
    </view>

    <view class="panel notification-email-panel">
      <view class="notification-email-panel__body">
        <text class="notification-email-panel__title">回流提醒邮箱</text>
        <text class="notification-email-panel__value">{{ emailLabel }}</text>
      </view>
      <button class="notification-email-panel__action" @click="emit('edit-email')">
        {{ hasEmail ? '编辑' : '填写' }}
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { UserAccountInfoItem } from '../helpers'

defineProps<{
  items: UserAccountInfoItem[]
  iconMap: Record<UserAccountInfoItem['iconName'], string>
  emailLabel: string
  hasEmail: boolean
}>()

const emit = defineEmits<{
  (e: 'edit-email'): void
}>()
</script>

<style scoped lang="css">
.account-info-stack {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-card-gap);
}

.hero-card, .panel {
  position: relative;
  z-index: 1;
  background: rgba(255,255,255,0.72);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: 2rpx solid rgba(255,255,255,0.55);
  box-shadow: 0 20rpx 48rpx rgba(26,28,36,0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.notification-email-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.notification-email-panel__body {
  min-width: 0;
  display: grid;
  gap: var(--fi-space-10);
}

.notification-email-panel__title {
  color: var(--fi-primitive-ink);
  font-size: var(--fi-font-30);
  font-weight: 800;
}

.notification-email-panel__value {
  color: #747985;
  font-size: var(--fi-font-24);
  line-height: 1.4;
  word-break: break-all;
}

.notification-email-panel__action {
  flex-shrink: 0;
  margin: 0 0 0 auto;
  min-width: 128rpx;
  height: 68rpx;
  padding: 0 26rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-26);
  font-weight: 800;
  line-height: 68rpx;
  box-shadow: none;
}

.notification-email-panel__action::after {
  border: none;
}

.info-panel,
.privilege-panel,
.upgrade-panel {
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-lg);
}

.info-panel,
.privilege-panel {
  border-color: rgba(229, 232, 238, 0.96);
  background: rgba(255, 255, 255, 0.94);
}

.info-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 80ms both;
}

.info-panel__title,
.privilege-panel__title,
.upgrade-panel__title {
  color: #1b1c20;
  font-size: var(--fi-font-30);
  font-weight: 800;
  line-height: 1;
}

.account-info-list {
  margin-top: var(--fi-space-8);
  display: flex;
  flex-direction: column;
}

.account-info-item {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-20) 0;
}

.account-info-item + .account-info-item {
  border-top: var(--fi-border-width) solid var(--fi-color-divider);
}

.account-info-item__icon {
  width: var(--fi-icon-md);
  height: var(--fi-icon-md);
  flex-shrink: 0;
}

.account-info-item__label {
  flex: 1;
  min-width: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
}

.account-info-item__value {
  flex-shrink: 0;
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  text-align: right;
}
</style>
