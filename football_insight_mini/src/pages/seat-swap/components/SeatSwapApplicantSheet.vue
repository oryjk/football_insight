<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="换座邀请"
    title="选择和谁换座"
    fit-content
    respect-window-bottom
    @update:visible="emit('update:visible', $event)"
    @close="emit('close')"
  >
    <view class="applicant-sheet">
      <SeatSwapApplicantList :applicants="applicants" @confirm="(id: string) => emit('confirm', id)" />
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import SeatSwapApplicantList from './SeatSwapApplicantList.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'

defineProps<{
  visible: boolean
  applicants: SeatSwapCandidate[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
  (e: 'confirm', requestId: string): void
}>()
</script>

<style scoped>
.applicant-sheet {
  padding: var(--fi-space-12) 0 var(--fi-space-14);
}
</style>
