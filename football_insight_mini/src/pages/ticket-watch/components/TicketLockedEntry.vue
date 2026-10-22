<template>
  <view class="locked-entry">
    <view class="locked-entry__row" @tap="expanded = !expanded">
      <view class="locked-entry__badge"><text>{{ badge }}</text></view>
      <view class="locked-entry__main">
        <text class="locked-entry__title">{{ title }}</text>
        <text class="locked-entry__summary">{{ summary }}</text>
      </view>
      <view class="locked-entry__toggle">
        <text>{{ expanded ? '收起' : '升级' }}</text>
        <text class="locked-entry__arrow" :class="{ 'locked-entry__arrow--open': expanded }">›</text>
      </view>
    </view>
    <view v-if="expanded" class="locked-entry__detail">
      <text class="locked-entry__detail-copy">{{ detail }}</text>
      <TicketUpgradeActions :hint="hint" @invite="emit('invite')" @purchase="emit('purchase')" />
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import TicketUpgradeActions from './TicketUpgradeActions.vue'

/** 会员锁定内容的紧凑入口：默认一行，点开才展开「邀请 / 开通」两种升级方式。 */
withDefaults(defineProps<{
  title: string
  summary: string
  detail?: string
  hint: string
  badge?: string
}>(), { detail: '', badge: 'V6+' })
const emit = defineEmits<{ (e: 'invite'): void; (e: 'purchase'): void }>()
const expanded = ref(false)
</script>

<style scoped lang="css">
.locked-entry {
  display: flex;
  flex-direction: column;
  padding: var(--fi-space-20) var(--fi-space-24);
  border-radius: var(--fi-component-panel-radius);
  background: var(--fi-component-panel-bg);
  border: var(--fi-component-panel-border);
  box-shadow: var(--fi-component-panel-shadow);
}

.locked-entry__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
}

.locked-entry__badge {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: var(--fi-space-72);
  height: var(--fi-space-72);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
}

.locked-entry__main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.locked-entry__title {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
}

.locked-entry__summary {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.locked-entry__toggle {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: var(--fi-space-6);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.locked-entry__arrow {
  display: inline-block;
  font-size: var(--fi-font-34);
  line-height: var(--fi-leading-none);
  transform: rotate(90deg);
}

.locked-entry__arrow--open {
  transform: rotate(-90deg);
}

.locked-entry__detail {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-20);
  padding-top: var(--fi-space-20);
  border-top: var(--fi-border-card);
}

.locked-entry__detail-copy {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}
</style>
