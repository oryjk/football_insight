<template>
  <view class="panel" :class="{ 'ticket-watch-entry--locked': locked }" @click="emit('open')">
    <view class="section-heading">
      <view>
        <text class="section-kicker">Ticket Watch</text>
        <text class="section-title">回流看板</text>
      </view>
      <text class="meta-pill">{{ locked ? '登录后' : '打开' }}</text>
    </view>
  </view>
</template>

<script setup lang="ts">
defineProps<{
  locked: boolean
}>()

const emit = defineEmits<{
  (e: 'open'): void
}>()
</script>

<style scoped lang="css">
.panel {
  position: relative;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.94);
  border-radius: var(--fi-radius-xl);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
  padding: var(--fi-space-20);
}

.ticket-watch-entry--locked {
  pointer-events: none;
}

.ticket-watch-entry--locked .section-heading {
  filter: blur(5rpx);
  opacity: 0.56;
}

.ticket-watch-entry--locked::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  background: rgba(255, 255, 255, 0.26);
  pointer-events: none;
}

.section-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.section-kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-component-title-size);
  line-height: 1.08;
  font-weight: var(--fi-weight-extrabold);
}

.meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
  padding: var(--fi-space-14) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}
</style>
