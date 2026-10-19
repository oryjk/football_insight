<template>
  <view class="notice">
    <image class="notice__icon" src="/static/seat-swap/info.svg" mode="aspectFit" />
    <text class="notice__text">{{ text }}</text>
    <button v-if="actionLabel" class="notice__action" hover-class="notice__action--pressed" @tap="emit('action')">
      {{ actionLabel }}
    </button>
  </view>
</template>

<script setup lang="ts">
defineProps<{ text: string; actionLabel?: string }>()
const emit = defineEmits<{ (e: 'action'): void }>()
</script>

<style scoped>
.notice {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-18) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
}

.notice__icon {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.notice__text {
  flex: 1;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.notice__action {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
}

.notice__action::after {
  border: 0;
}

.notice__action--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}
</style>
