<template>
  <view class="applicants">
    <view class="applicants__head">
      <text class="applicants__title">想和你换座</text>
      <text class="applicants__count">{{ applicants.length }} 人</text>
    </view>
    <view v-for="candidate in applicants" :key="candidate.request_id" class="applicants__item">
      <SeatSwapCandidateCard
        :candidate="candidate"
        action="confirm"
        @confirm="(id: string) => emit('confirm', id)"
      />
    </view>
  </view>
</template>

<script setup lang="ts">
import SeatSwapCandidateCard from './SeatSwapCandidateCard.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'

// 对方已邀请我的申请人列表：管理弹层和列表里的「查看」共用，在这里选择和谁换座。
defineProps<{
  applicants: SeatSwapCandidate[]
}>()

const emit = defineEmits<{
  (e: 'confirm', requestId: string): void
}>()
</script>

<style scoped>
.applicants__head {
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-12);
  margin-bottom: var(--fi-space-12);
}

.applicants__title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.applicants__count {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.applicants__item + .applicants__item {
  margin-top: var(--fi-space-12);
}
</style>
