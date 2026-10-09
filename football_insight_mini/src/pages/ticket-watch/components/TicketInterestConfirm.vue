<template>
  <view class="interest-confirm-mask" @tap="emit('close')">
    <view class="interest-confirm-dialog" @tap.stop>
      <text class="interest-confirm-dialog__title">确认钓位</text>
      <text class="interest-confirm-dialog__content">
        确认把 {{ blockName }} 设为你的钓位吗？
      </text>
      <view class="interest-confirm-dialog__actions">
        <button
          class="interest-confirm-dialog__button interest-confirm-dialog__button--ghost"
          @tap="emit('close')"
        >
          取消
        </button>
        <button
          class="interest-confirm-dialog__button interest-confirm-dialog__button--primary"
          @tap="emit('confirm')"
        >
          确认
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{ blockName: string }>()
const emit = defineEmits<{ (e: 'close'): void; (e: 'confirm'): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.interest-confirm-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-32);
  background: var(--fi-color-ticket-interest-confirm-mask-background);
}

.interest-confirm-dialog {
  width: 100%;
  max-width: 560rpx;
  border-radius: var(--fi-radius-ticket-32);
  background: var(--fi-color-ticket-history-trend-summary-item-background-2);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-interest-confirm-dialog-border);
  box-shadow: var(--fi-shadow-ticket-interest-confirm-dialog);
  padding: var(--fi-space-32) var(--fi-space-28) var(--fi-space-24);
}

.interest-confirm-dialog__title {
  display: block;
  color: var(--fi-color-ticket-interest-confirm-dialog-title-color);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-tight);
}

.interest-confirm-dialog__content {
  display: block;
  margin-top: var(--fi-space-16);
  color: var(--fi-color-ticket-interest-confirm-dialog-content-color);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-ticket-1-65);
}

.interest-confirm-dialog__actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-14);
  margin-top: var(--fi-space-28);
}

.interest-confirm-dialog__button {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 76rpx;
  padding: 0 var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  text-align: center;
}

.interest-confirm-dialog__button::after {
  border: none;
}

.interest-confirm-dialog__button--ghost {
  background: var(--fi-color-ticket-history-chip-background);
  color: var(--fi-color-ticket-interest-confirm-dialog-button-ghost-color);
}

.interest-confirm-dialog__button--primary {
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
}
</style>
