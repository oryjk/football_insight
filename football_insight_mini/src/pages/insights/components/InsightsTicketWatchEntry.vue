<template>
  <FiPanel flush>
    <view
      class="ticket-watch-entry"
      role="button"
      aria-label="打开回流看板"
      hover-class="ticket-watch-entry--pressed"
      :hover-stay-time="80"
      @tap="emit('open')"
    >
      <view class="ticket-watch-entry__icon">
        <image :src="ticketIcon" class="ticket-watch-entry__icon-image" mode="aspectFit" />
      </view>
      <view class="ticket-watch-entry__copy">
        <text class="ticket-watch-entry__title">回流看板</text>
        <text class="ticket-watch-entry__caption">{{ locked ? '登录后查看实时余票与回流' : '实时余票监控 · 最近回流速览' }}</text>
      </view>
      <text v-if="locked" class="ticket-watch-entry__badge">登录后</text>
      <image v-else :src="chevronIcon" class="ticket-watch-entry__chevron" mode="aspectFit" />
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import ticketIcon from '../../../static/user/ticket.svg'
import chevronIcon from '../../../static/rankings/chevron-right.svg'

defineProps<{ locked: boolean }>()

const emit = defineEmits<{
  (e: 'open'): void
}>()
</script>

<style scoped lang="css">
.ticket-watch-entry {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-28) var(--fi-space-20);
}
.ticket-watch-entry--pressed { opacity: var(--fi-opacity-insights-pressed); }
.ticket-watch-entry__icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-insights-icon);
  background: var(--fi-color-insights-entry-icon);
}
.ticket-watch-entry__icon-image,
.ticket-watch-entry__chevron {
  flex-shrink: 0;
  width: var(--fi-space-36);
  height: var(--fi-space-36);
}
.ticket-watch-entry__copy { flex: 1; min-width: 0; }
.ticket-watch-entry__title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-extrabold);
}
.ticket-watch-entry__caption {
  display: block;
  margin-top: var(--fi-space-8);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}
.ticket-watch-entry__badge {
  flex-shrink: 0;
  padding: var(--fi-space-10) var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}
</style>
