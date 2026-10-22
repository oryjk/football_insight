<template>
  <view class="upgrade-actions">
    <view class="upgrade-actions__invite">
      <text class="upgrade-actions__invite-title">两种方式升级</text>
      <text class="upgrade-actions__invite-copy">① 邀请好友（免费）：{{ hint }}</text>
      <text class="upgrade-actions__invite-copy">② 直接开通会员，立即生效。</text>
    </view>
    <view class="upgrade-actions__buttons">
      <button class="upgrade-actions__button upgrade-actions__button--invite" @tap="emit('invite')">
        邀请好友免费升级
      </button>
      <button class="upgrade-actions__button upgrade-actions__button--purchase" @tap="emit('purchase')">
        直接开通
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{ hint: string }>()
const emit = defineEmits<{ (e: 'invite'): void; (e: 'purchase'): void }>()
</script>

<style scoped lang="css">
.upgrade-actions {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  width: 100%;
}

.upgrade-actions__invite {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.upgrade-actions__invite-title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.upgrade-actions__invite-copy {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
  overflow-wrap: anywhere;
}

.upgrade-actions__buttons {
  display: flex;
  gap: var(--fi-space-16);
}

.upgrade-actions__button {
  flex: 1;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: var(--fi-space-72);
  padding: 0 var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  text-align: center;
}

.upgrade-actions__button::after {
  border: none;
}

.upgrade-actions__button--invite {
  flex: 1.4;
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
}

.upgrade-actions__button--purchase {
  background: transparent;
  border: var(--fi-border-card);
  color: var(--fi-color-text-primary);
}
</style>
