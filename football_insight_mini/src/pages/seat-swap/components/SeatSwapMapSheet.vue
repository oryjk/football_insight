<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="球场地图"
    title="点分区筛选意向"
    height="80vh"
    fit-content
    @update:visible="emit('update:visible', $event)"
  >
    <view class="map-sheet">
      <SeatSwapRegionPanel
        embedded
        :mode="mode"
        :regions="regions"
        :badges="badges"
        :filter-key="filterKey"
        :current-key="currentKey"
        :desired-keys="desiredKeys"
        :has-my-request="hasMyRequest"
        @region-tap="(key: string) => emit('region-tap', key)"
      />
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import SeatSwapRegionPanel from './SeatSwapRegionPanel.vue'
import type { TicketWatchRegion } from '../../../types/ticketWatch'

defineProps<{
  visible: boolean
  mode: 'browse' | 'filter' | 'published'
  regions: TicketWatchRegion[]
  badges: Record<string, number>
  filterKey: string
  currentKey: string
  desiredKeys: string[]
  hasMyRequest: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'region-tap', key: string): void
}>()
</script>

<style scoped>
.map-sheet {
  padding-bottom: var(--fi-space-24);
}
</style>
