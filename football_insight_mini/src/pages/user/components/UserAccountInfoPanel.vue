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
        <view v-if="showOfficialBind" class="account-info-item" @tap="emit('bind-official')">
          <image class="account-info-item__icon" :src="iconMap['badge-check']" mode="aspectFit" />
          <text class="account-info-item__label">绑定公众号</text>
          <text class="account-info-item__value">去绑定 ›</text>
        </view>
      </view>
    </view>

    <view class="panel student-entry-panel" hover-class="student-entry-panel--pressed" @tap="emit('open-student')">
      <view class="student-entry-panel__body">
        <text class="student-entry-panel__title">学员功能</text>
        <text class="student-entry-panel__value">服务购买、购买记录与提醒邮箱</text>
      </view>
      <image :src="chevronIcon" class="student-entry-panel__arrow" mode="aspectFit" aria-hidden="true" />
    </view>
  </view>
</template>

<script setup lang="ts">
import chevronIcon from '../../../static/rankings/chevron-right.svg'
import type { UserAccountInfoItem } from '../helpers'

defineProps<{
  items: UserAccountInfoItem[]
  iconMap: Record<UserAccountInfoItem['iconName'], string>
  showOfficialBind: boolean
}>()

const emit = defineEmits<{
  (e: 'open-student'): void
  (e: 'bind-official'): void
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
  background: var(--fi-color-surface-frosted);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.student-entry-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  border-color: var(--fi-color-border-card);
  background: var(--fi-color-surface-panel);
}

.student-entry-panel--pressed {
  opacity: var(--fi-opacity-pressed);
}

.student-entry-panel__body {
  min-width: 0;
  display: grid;
  gap: var(--fi-space-10);
}

.student-entry-panel__title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
}

.student-entry-panel__value {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
  word-break: break-all;
}

.student-entry-panel__arrow {
  display: block;
  flex-shrink: 0;
  width: var(--fi-space-36);
  height: var(--fi-space-36);
}

.info-panel,
.privilege-panel,
.upgrade-panel {
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-lg);
}

.info-panel,
.privilege-panel {
  border-color: var(--fi-color-border-card);
  background: var(--fi-color-surface-panel);
}

.info-panel {
  animation: fi-fade-in-up 520ms cubic-bezier(0.22, 1, 0.36, 1) 80ms both;
}

.info-panel__title,
.privilege-panel__title,
.upgrade-panel__title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
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
