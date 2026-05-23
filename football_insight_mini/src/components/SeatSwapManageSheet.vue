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
        <text class="seat-swap-manage__note">已匹配成功。如线下协商取消,请填写说明并上传双方达成一致的截图。</text>
        <view class="seat-swap-manage__field seat-swap-manage__field--full">
          <text class="seat-swap-manage__field-label">撤销说明</text>
          <textarea
            :value="cancelReason"
            class="seat-swap-manage__textarea"
            placeholder="撤销说明"
            @input="emit('update:cancelReason', normalizeInputValue($event))"
          />
        </view>
        <view class="seat-swap-manage__evidence">
          <button class="seat-swap-manage__ghost seat-swap-manage__evidence-pick" @tap="emit('choose-evidence')">
            {{ evidenceFileName ? '重新选择截图' : '选择截图' }}
          </button>
          <text v-if="evidenceFileName" class="seat-swap-manage__evidence-name">{{ evidenceFileName }}</text>
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
import FiBottomSheet from './FiBottomSheet.vue'
import type { SeatSwapRequest } from '../types/seatSwap'
import { formatSeatSwapSeatLabel } from '../utils/stadiumRegions'

defineProps<{
  visible: boolean
  request?: SeatSwapRequest | null
  statusLabel: string
  desiredSummary: string
  cancelReason: string
  evidenceFileName: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:cancelReason', value: string): void
  (e: 'close'): void
  (e: 'edit'): void
  (e: 'delete'): void
  (e: 'choose-evidence'): void
  (e: 'submit-matched-cancel'): void
}>()

function normalizeInputValue(event: Event): string {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement | null
  return target?.value || ''
}
</script>

<style scoped>
.seat-swap-manage {
  padding-bottom: 14rpx;
}

.seat-swap-manage__summary {
  padding: 18rpx 22rpx;
  margin: 18rpx 0 0;
  border-radius: 16rpx;
  background: rgba(255, 251, 242, 0.6);
  border: 1rpx solid rgba(230, 220, 198, 0.92);
}

.seat-swap-manage__summary-row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 16rpx;
  padding: 6rpx 0;
}

.seat-swap-manage__summary-label {
  color: #8f7c5f;
  font-size: 22rpx;
}

.seat-swap-manage__summary-value {
  color: #121212;
  font-size: 24rpx;
  font-weight: 400;
  text-align: right;
  flex: 1;
}

.seat-swap-manage__note {
  display: block;
  margin: 12rpx 4rpx;
  color: #988f84;
  font-size: 22rpx;
  line-height: 1.5;
}

.seat-swap-manage__field {
  display: flex;
  gap: 16rpx;
  margin-top: 12rpx;
}

.seat-swap-manage__field--full {
  flex-direction: column;
}

.seat-swap-manage__field-label {
  display: block;
  color: #8f7c5f;
  font-size: 22rpx;
  font-weight: 400;
  margin-bottom: 6rpx;
}

.seat-swap-manage__textarea {
  width: 100%;
  min-height: 140rpx;
  padding: 14rpx 18rpx;
  background: rgba(255, 251, 242, 0.6);
  border: 1rpx solid rgba(230, 220, 198, 0.92);
  border-radius: 16rpx;
  font-size: 26rpx;
  color: #121212;
  box-sizing: border-box;
}

.seat-swap-manage__evidence {
  margin-top: 14rpx;
  display: flex;
  align-items: center;
  gap: 14rpx;
  flex-wrap: wrap;
}

.seat-swap-manage__evidence-pick {
  flex-shrink: 0;
}

.seat-swap-manage__evidence-name {
  flex: 1;
  color: #988f84;
  font-size: 22rpx;
  word-break: break-all;
}

.seat-swap-manage__actions {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 16rpx;
}

.seat-swap-manage__primary {
  flex: 1;
  padding: 20rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  font-size: 28rpx;
  font-weight: 400;
  box-shadow: inset 0 1rpx 0 rgba(255, 255, 255, 0.08), 0 6rpx 14rpx rgba(21, 22, 27, 0.16);
}

.seat-swap-manage__primary--danger {
  background: linear-gradient(180deg, #c4321c, #a82716);
}

.seat-swap-manage__primary::after {
  border: 0;
}

.seat-swap-manage__ghost {
  flex-shrink: 0;
  padding: 20rpx 28rpx;
  border-radius: 999rpx;
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  border: 1rpx solid rgba(230, 220, 198, 0.92);
  color: #9c855c;
  font-size: 26rpx;
  font-weight: 400;
}

.seat-swap-manage__ghost--danger {
  border-color: rgba(180, 35, 24, 0.3);
  color: #b42318;
  background: linear-gradient(180deg, rgba(252, 235, 232, 0.9), rgba(250, 228, 224, 0.9));
}

.seat-swap-manage__ghost::after {
  border: 0;
}
</style>
