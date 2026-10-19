<template>
  <view v-if="visible" class="confirm-mask" @tap="emit('cancel')" @touchmove.stop.prevent>
    <view class="confirm-card" role="dialog" :aria-label="title" aria-modal="true" @tap.stop @touchmove.stop.prevent>
      <text class="confirm-card__title">{{ title }}</text>
      <text class="confirm-card__message">{{ message }}</text>
      <view class="confirm-card__actions">
        <button class="confirm-card__button confirm-card__button--cancel" hover-class="confirm-card__button--pressed" @tap="emit('cancel')">取消</button>
        <button class="confirm-card__button confirm-card__button--confirm" hover-class="confirm-card__button--pressed" @tap="emit('confirm')">确认</button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean; title: string; message: string }>()
const emit = defineEmits<{
  (event: 'confirm'): void
  (event: 'cancel'): void
}>()
</script>

<style scoped>
.confirm-mask {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--fi-space-32);
  box-sizing: border-box;
  background: var(--fi-color-overlay);
}

.confirm-card {
  box-sizing: border-box;
  width: 100%;
  max-width: calc(var(--fi-space-64) * 10);
  padding: var(--fi-space-32);
  border: var(--fi-border-card);
  border-radius: var(--fi-radius-xl);
  background: var(--fi-color-surface);
  box-shadow: var(--fi-shadow-floating);
}

.confirm-card__title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  text-align: center;
}

.confirm-card__message {
  display: block;
  margin-top: var(--fi-space-20);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
  text-align: center;
}

.confirm-card__actions {
  display: flex;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-32);
}

.confirm-card__button {
  flex: 1;
  margin: 0;
  padding: var(--fi-space-20) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.confirm-card__button::after {
  border: 0;
}

.confirm-card__button--cancel {
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-primary);
}

.confirm-card__button--confirm {
  border: var(--fi-border-width) solid var(--fi-color-primary-deep);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
}

.confirm-card__button--pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
