<template>
<view  class="recent-reflux-lock">
      <view class="recent-reflux-lock__preview">
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--red" />
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--middle" />
        <view class="recent-reflux-lock__bar recent-reflux-lock__bar--green" />
      </view>
      <view class="recent-reflux-lock__body">
        <text class="recent-reflux-lock__title">V6 及以上会员可查看，邀请好友就能免费升级</text>
        <text class="recent-reflux-lock__copy"
          >V6 可查看 30 分钟内，V7 可查看 10 分钟内，V8/V9 可查看 3
          分钟内的具体座位区域。</text
        >
      </view>
      <TicketUpgradeActions
        :hint="inviteHint"
        @invite="emit('invite')"
        @purchase="emit('upgrade')"
      />
    </view>
</template>
<script setup lang="ts">
import TicketUpgradeActions from './TicketUpgradeActions.vue'
defineProps<{ inviteHint: string }>()
const emit = defineEmits<{ (e: 'upgrade'): void; (e: 'invite'): void }>()
</script>
<style scoped lang="css">
.recent-reflux-lock {
  position: relative;
  overflow: hidden;
  display: grid;
  grid-template-columns: calc(var(--fi-space-100) + var(--fi-space-8)) minmax(0, 1fr);
  gap: var(--fi-space-18);
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-ticket-20);
  background: var(--fi-color-page);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-lock-border);
}

.recent-reflux-lock::after {
  content: '';
  position: absolute;
  right: calc(calc(var(--fi-space-36) + var(--fi-space-2)) * -1);
  top: calc(var(--fi-space-48) * -1);
  width: calc(var(--fi-space-100) + var(--fi-space-84) + var(--fi-space-6));
  height: calc(var(--fi-space-100) + var(--fi-space-84) + var(--fi-space-6));
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-recent-reflux-lock-background-5);
  pointer-events: none;
}
.recent-reflux-lock__preview {
  position: relative;
  z-index: 1;
  height: calc(var(--fi-space-100) + var(--fi-space-12));
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-recent-reflux-lock-preview-background);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-recent-reflux-lock-preview-border);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-8);
}

.recent-reflux-lock__bar {
  width: var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  box-shadow: var(--fi-shadow-ticket-recent-reflux-lock-bar);
}

.recent-reflux-lock__bar--red {
  height: calc(var(--fi-space-72) + var(--fi-space-2));
  color: var(--fi-color-ticket-recent-reflux-lock-bar-red-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-red-background);
}

.recent-reflux-lock__bar--middle {
  height: var(--fi-space-56);
  color: var(--fi-color-ticket-recent-reflux-lock-bar-middle-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-middle-background);
}

.recent-reflux-lock__bar--green {
  height: calc(var(--fi-space-40) + var(--fi-space-2));
  color: var(--fi-color-ticket-recent-reflux-lock-bar-green-color);
  background: var(--fi-color-ticket-recent-reflux-lock-bar-green-background);
}

.recent-reflux-lock__body {
  position: relative;
  z-index: 1;
  min-width: 0;
}

.recent-reflux-lock__title {
  display: block;
  color: var(--fi-color-ticket-recent-reflux-lock-title-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
}

.recent-reflux-lock__copy {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-ticket-recent-reflux-lock-copy-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.recent-reflux-lock__action {
  position: relative;
  z-index: 1;
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: center;
  height: var(--fi-space-72);
  line-height: var(--fi-leading-none);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  text-align: center;
}

.recent-reflux-lock__action::after {
  border: none;
}
</style>
