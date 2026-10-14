<template>
  <FiPanel flush clip>
    <view class="ticket-watch-entry" :class="{ 'ticket-watch-entry--locked': locked }" @click="emit('open')">
      <FiSectionHeading kicker="Ticket Watch" title="回流看板">
        <template #meta>
          <text class="meta-pill">{{ locked ? '登录后' : '打开' }}</text>
        </template>
      </FiSectionHeading>
    </view>
    <view v-if="locked" class="ticket-watch-entry__veil" />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
defineProps<{
  locked: boolean
}>()

const emit = defineEmits<{
  (e: 'open'): void
}>()
</script>

<style scoped lang="css">
.ticket-watch-entry {
  padding: var(--fi-component-panel-padding);
}

.ticket-watch-entry--locked {
  pointer-events: none;
  filter: blur(5rpx);
  opacity: 0.56;
}

/* 锁定态白色薄罩覆盖整张卡（含内边距），由 FiPanel clip 裁剪到圆角内。 */
.ticket-watch-entry__veil {
  position: absolute;
  inset: 0;
  background: rgba(255, 255, 255, 0.26);
  pointer-events: none;
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
