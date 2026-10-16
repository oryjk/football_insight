<template>
  <view class="watch-header">
    <FiSegmented v-model="selectedTab" :items="tabs" aria-label="回流看板视图" />
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiSegmented from '../../../components/FiSegmented.vue'
import type { FiSegmentedItem } from '../../../types/segmented'
import type { TicketWatchTab } from '../types'
import activityIdle from '../../../static/ticket-watch/activity-idle.svg'
import activityActive from '../../../static/ticket-watch/activity-active.svg'
import historyIdle from '../../../static/ticket-watch/history-idle.svg'
import historyActive from '../../../static/ticket-watch/history-active.svg'
import chartIdle from '../../../static/ticket-watch/chart-column-idle.svg'
import chartActive from '../../../static/ticket-watch/chart-column-active.svg'

const props = defineProps<{ activeTab: TicketWatchTab }>()
const emit = defineEmits<{ (e: 'tab', tab: TicketWatchTab): void }>()
const tabs: FiSegmentedItem[] = [
  { key: 'current', label: '当前比赛', icon: activityIdle, iconActive: activityActive },
  { key: 'history', label: '历史比赛', icon: historyIdle, iconActive: historyActive },
  { key: 'history-stats', label: '历史统计', icon: chartIdle, iconActive: chartActive },
]
const selectedTab = computed({
  get: () => props.activeTab,
  set: (tab: string) => { emit('tab', tab as TicketWatchTab) },
})
</script>

<style scoped lang="css">
.watch-header { display: flex; flex-direction: column; align-items: stretch; }
</style>
