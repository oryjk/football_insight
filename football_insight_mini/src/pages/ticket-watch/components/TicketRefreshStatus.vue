<template>
  <view class="refresh-status" :class="`refresh-status--${state}`" role="status">
    <view class="refresh-status__summary">
      <view class="refresh-status__dot" />
      <text>{{ statusLabel }}</text>
    </view>
    <text class="refresh-status__freshness">{{ freshnessLabel }}</text>
  </view>
</template>
<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { formatFreshness, formatRefreshInterval, resolveRefreshState } from '../presentation'

const props = defineProps<{
  monitoring: boolean
  refreshing: boolean
  pollFailing: boolean
  intervalSeconds: number
  lastUpdatedAtMs: number | null
  ticking: boolean
}>()

const now = ref(Date.now())
const state = computed(() => resolveRefreshState(props))
const intervalLabel = computed(() => formatRefreshInterval(props.intervalSeconds))
const freshness = computed(() => formatFreshness(now.value, props.lastUpdatedAtMs))
const statusLabel = computed(() => {
  if (state.value === 'idle') return '未开启监控 · 数据不会自动刷新'
  if (state.value === 'refreshing') return '刷新中…'
  if (state.value === 'failing') return '刷新失败，将自动重试'
  return intervalLabel.value ? `监控中 · 每 ${intervalLabel.value} 刷新` : '监控中'
})
const freshnessLabel = computed(() => {
  if (freshness.value === '尚未更新') return freshness.value
  return state.value === 'failing' ? `上次成功 ${freshness.value}` : `${freshness.value} 更新`
})

let timer: ReturnType<typeof setInterval> | null = null
function stopClock(): void {
  if (timer !== null) clearInterval(timer)
  timer = null
}
watch(() => [props.monitoring, props.ticking], () => {
  stopClock()
  if (!props.ticking) return
  now.value = Date.now()
  timer = setInterval(() => { now.value = Date.now() }, props.monitoring ? 1_000 : 15_000)
}, { immediate: true })
onUnmounted(stopClock)
</script>

<style scoped lang="css">
.refresh-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--fi-space-8) var(--fi-space-12);
  padding: var(--fi-space-14) var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-medium);
  line-height: var(--fi-leading-normal);
}

.refresh-status__summary {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.refresh-status__dot {
  width: var(--fi-space-10);
  height: var(--fi-space-10);
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-muted);
}

.refresh-status__freshness { color: var(--fi-color-text-muted); }
.refresh-status--refreshing .refresh-status__dot,
.refresh-status--failing .refresh-status__dot { background: var(--fi-color-primary); }
.refresh-status--live .refresh-status__dot { background: var(--fi-color-success-accent); }
.refresh-status--refreshing .refresh-status__dot,
.refresh-status--live .refresh-status__dot { animation: refresh-status-pulse 1.6s ease-in-out infinite; }

@keyframes refresh-status-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: var(--fi-opacity-pressed); }
}

@media (prefers-reduced-motion: reduce) {
  .refresh-status__dot { animation: none !important; }
}
</style>
