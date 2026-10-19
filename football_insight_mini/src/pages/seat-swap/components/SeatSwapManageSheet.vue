<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="我的发布"
    :title="statusLabel"
    @update:visible="emit('update:visible', $event)"
    @close="emit('close')"
  >
    <view v-if="request" class="seat-swap-manage">
      <view class="seat-swap-manage__summary">
        <view class="seat-swap-manage__summary-row">
          <text class="seat-swap-manage__summary-label">当前</text>
          <text class="seat-swap-manage__summary-value">{{ formatSeatSwapSeatLabel(request) }}</text>
        </view>
        <view class="seat-swap-manage__summary-row">
          <text class="seat-swap-manage__summary-label">目标</text>
          <text class="seat-swap-manage__summary-value">{{ desiredSummary }}</text>
        </view>
      </view>

      <template v-if="request.status !== 'matched'">
        <text class="seat-swap-manage__note">你可以编辑或撤销当前发布。撤销后已发出的"确认换座"会一并失效。</text>
      </template>
      <template v-else>
        <text class="seat-swap-manage__note">已匹配成功。如双方协商取消，填写原因后即可取消当前匹配。</text>
        <view class="seat-swap-manage__field seat-swap-manage__field--full">
          <text class="seat-swap-manage__field-label">撤销说明</text>
          <textarea
            :value="cancelReason"
            class="seat-swap-manage__textarea"
            placeholder="撤销说明"
            @input="emit('update:cancelReason', normalizeInputValue($event))"
          />
        </view>
      </template>
    </view>

    <template #footer>
      <view class="seat-swap-manage__actions">
        <template v-if="request?.status !== 'matched'">
          <button class="seat-swap-manage__ghost seat-swap-manage__ghost--danger" @tap="emit('delete')">撤销发布</button>
          <button class="seat-swap-manage__primary" @tap="emit('edit')">编辑发布</button>
        </template>
        <template v-else>
          <button class="seat-swap-manage__ghost" @tap="emit('close')">取消</button>
          <button class="seat-swap-manage__primary seat-swap-manage__primary--danger" @tap="emit('submit-matched-cancel')">
            提交撤销申请
          </button>
        </template>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { SeatSwapRequest } from '../../../types/seatSwap'
import { formatSeatSwapSeatLabel } from '../../../utils/stadiumRegions'

defineProps<{
  visible: boolean
  request?: SeatSwapRequest | null
  statusLabel: string
  desiredSummary: string
  cancelReason: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:cancelReason', value: string): void
  (e: 'close'): void
  (e: 'edit'): void
  (e: 'delete'): void
  (e: 'submit-matched-cancel'): void
}>()

function normalizeInputValue(event: Event): string {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement | null
  return target?.value || ''
}
</script>

<style scoped>
.seat-swap-manage {
  padding-bottom: var(--fi-space-14);
}

.seat-swap-manage__summary {
  padding: var(--fi-space-18) var(--fi-space-22);
  margin: var(--fi-space-18) 0 0;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
}

.seat-swap-manage__summary-row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: var(--fi-space-16);
  padding: var(--fi-space-6) 0;
}

.seat-swap-manage__summary-label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.seat-swap-manage__summary-value {
  flex: 1;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  text-align: right;
}

.seat-swap-manage__note {
  display: block;
  margin: var(--fi-space-12) var(--fi-space-4);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.seat-swap-manage__field {
  display: flex;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-12);
}

.seat-swap-manage__field--full {
  flex-direction: column;
}

.seat-swap-manage__field-label {
  display: block;
  margin-bottom: var(--fi-space-6);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.seat-swap-manage__textarea {
  width: 100%;
  min-height: 140rpx;
  padding: var(--fi-space-14) var(--fi-space-18);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  border-radius: var(--fi-radius-sm);
  font-size: var(--fi-font-26);
  color: var(--fi-color-text-strong);
  box-sizing: border-box;
}

.seat-swap-manage__actions {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
}

.seat-swap-manage__primary {
  flex: 1;
  padding: var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  box-shadow: var(--fi-shadow-soft);
}

.seat-swap-manage__primary--danger {
  background: var(--fi-color-primary-deep);
}

.seat-swap-manage__primary::after {
  border: 0;
}

.seat-swap-manage__ghost {
  flex-shrink: 0;
  padding: var(--fi-space-20) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.seat-swap-manage__ghost--danger {
  border-color: rgba(var(--fi-primitive-red-rgb), 0.3);
  color: var(--fi-color-primary-deep);
  background: rgba(var(--fi-primitive-red-rgb), 0.06);
}

.seat-swap-manage__ghost::after {
  border: 0;
}
</style>
