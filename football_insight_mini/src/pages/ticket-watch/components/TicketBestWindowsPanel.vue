<template>
  <view v-if="loading" class="fi-skeleton-panel best-windows__skeleton">
    <view class="fi-skeleton-block fi-skeleton-title" />
    <view class="fi-skeleton-block best-windows__skeleton-chart" />
    <view class="fi-skeleton-block" />
  </view>
  <FiPanel v-else-if="data">
    <view class="best-windows">
      <FiSectionHeading title="最佳盯盘时段" />
      <text class="best-windows__sample">近 {{ data.sample_size }} 场主场的补票分布</text>
      <template v-if="data.access === 'full'">
        <view class="best-windows__chart">
          <view
            v-for="bar in bars"
            :key="bar.hour"
            class="best-windows__column"
            role="button"
            :aria-label="bestWindowsHourDetail(bar.hour, data.sample_size, bar.hit_matches)"
            @tap="selectedHour = bar.hour"
          >
            <view class="best-windows__track">
              <view
                class="best-windows__bar"
                :class="{ 'best-windows__bar--peak': bar.isPeak }"
                :style="{ height: `${bar.heightPercent}%` }"
              />
            </view>
            <text class="best-windows__tick">{{ bar.label }}</text>
            <text v-if="bar.isNow" class="best-windows__now">现在</text>
          </view>
        </view>
        <text class="best-windows__conclusion">{{ formatBestWindowsConclusion(data.top_windows) }}</text>
        <text v-if="selectedBar" class="best-windows__detail">{{ bestWindowsHourDetail(selectedBar.hour, data.sample_size, selectedBar.hit_matches) }}</text>
        <text class="best-windows__note">按历史补票次数统计，仅供参考，不代表之后一定出票。</text>
      </template>
      <template v-else>
        <text class="best-windows__conclusion">最容易出票的时段：{{ formatBestWindow(data.top_windows[0]) }}</text>
        <view class="best-windows__lock">
          <text class="best-windows__lock-copy">开通 V6 查看全天分布和前三个高峰时段</text>
          <button class="best-windows__upgrade" @tap="emit('upgrade')">升级查看</button>
        </view>
      </template>
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
import type { TicketWatchBestWindowsResponse } from '../../../types/ticketWatch'
import { bestWindowsHourDetail, buildBestWindowsBars, formatBestWindow, formatBestWindowsConclusion } from '../bestWindowsHelpers'

const props = defineProps<{
  data: TicketWatchBestWindowsResponse | null
  loading: boolean
  visible: boolean
}>()
const emit = defineEmits<{ (e: 'upgrade'): void }>()
// Statistics use Beijing hours, independent of the device's timezone.
const beijingHour = () => new Date(Date.now() + 8 * 60 * 60 * 1000).getUTCHours()
const currentHour = ref(beijingHour())
const selectedHour = ref<number | null>(null)
const bars = computed(() => props.data ? buildBestWindowsBars(props.data, currentHour.value) : [])
const selectedBar = computed(() => bars.value.find(bar => bar.hour === selectedHour.value))
let timer: ReturnType<typeof setInterval> | null = null
function stopClock() {
  if (timer !== null) clearInterval(timer)
  timer = null
}
watch(() => props.visible && !props.loading && props.data?.access === 'full', (running) => {
  stopClock()
  if (running) {
    currentHour.value = beijingHour()
    timer = setInterval(() => { currentHour.value = beijingHour() }, 60_000)
  }
}, { immediate: true })
watch(() => props.data, () => { selectedHour.value = null })
onUnmounted(stopClock)
</script>

<style scoped>
.best-windows {
  display: flex;
  flex-direction: column;
  min-width: 0;
  gap: var(--fi-space-16);
}
.best-windows__sample,
.best-windows__note {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}
.best-windows__chart {
  display: flex;
  gap: var(--fi-space-4);
  min-width: 0;
  padding-bottom: var(--fi-space-30);
}
.best-windows__column {
  position: relative;
  flex: 1 1 0;
  min-width: 0;
}
.best-windows__track {
  height: calc(var(--fi-space-100) + var(--fi-space-100));
  display: flex;
  align-items: flex-end;
}
.best-windows__bar {
  width: 100%;
  border-radius: var(--fi-radius-sm) var(--fi-radius-sm) 0 0;
  background: var(--fi-color-divider);
}
.best-windows__bar--peak { background: var(--fi-color-text-primary); }
.best-windows__tick {
  display: block;
  min-height: var(--fi-space-24);
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-18);
  line-height: var(--fi-leading-none);
}
.best-windows__now {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-18);
  line-height: var(--fi-leading-normal);
}
.best-windows__column:first-child .best-windows__now { left: 0; transform: none; }
.best-windows__column:last-child .best-windows__now { left: auto; right: 0; transform: none; }
.best-windows__conclusion,
.best-windows__detail,
.best-windows__lock-copy {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
  overflow-wrap: anywhere;
}
.best-windows__lock {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-page);
}
.best-windows__upgrade {
  align-self: stretch;
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: var(--fi-space-72);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}
.best-windows__upgrade::after { border: none; }
.best-windows__skeleton {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-24);
  min-height: calc(var(--fi-space-100) * 4);
}
.best-windows__skeleton-chart {
  width: 100%;
  height: calc(var(--fi-space-100) * 2);
  border-radius: var(--fi-radius-sm);
}
</style>
