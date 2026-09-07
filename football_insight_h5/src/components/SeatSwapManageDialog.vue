<template>
  <div v-if="visible && request" class="manage">
    <div class="manage__scrim" @click="handleClose"></div>
    <div class="manage__panel">
      <div class="manage__head">
        <div class="manage__head-main">
          <span class="manage__kicker">我的换座</span>
          <span class="manage__title">{{ cancelMode ? '申请取消匹配' : '管理我的发布' }}</span>
        </div>
        <button type="button" class="manage__close" @click="handleClose">✕</button>
      </div>

      <div class="manage__body">
        <div class="manage__status">
          <span class="manage__status-dot"></span>
          <span class="manage__status-label">{{ statusLabel }}</span>
        </div>

        <div class="manage__row">
          <span class="manage__label">当前座位</span>
          <span class="manage__seat manage__seat--current">{{ seatLabel }}</span>
        </div>
        <div class="manage__row">
          <span class="manage__label">目标座位</span>
          <span class="manage__seat manage__seat--desired">{{ desiredSummary }}</span>
        </div>
        <div v-if="request.contact" class="manage__row">
          <span class="manage__label">联系方式</span>
          <div class="manage__contact">
            <span v-if="request.contact.wechat_id" class="manage__contact-line">微信：{{ request.contact.wechat_id }}</span>
            <span v-if="request.contact.phone_number" class="manage__contact-line">手机：{{ request.contact.phone_number }}</span>
          </div>
        </div>

        <div v-if="cancelMode" class="manage__cancel-box">
          <span class="manage__label">撤销说明（必填）</span>
          <textarea
            v-model="reasonText"
            class="manage__textarea"
            placeholder="简单说明取消原因，提交后对方会看到"
            :maxlength="200"
          ></textarea>
        </div>
      </div>

      <div class="manage__footer">
        <template v-if="cancelMode">
          <button type="button" class="manage__btn manage__btn--ghost" @click="emit('update:cancelMode', false)">返回</button>
          <button type="button" class="manage__btn manage__btn--danger" @click="emit('submit-matched-cancel')">提交取消申请</button>
        </template>
        <template v-else>
          <button type="button" class="manage__btn manage__btn--ghost" @click="emit('delete')">撤销发布</button>
          <button type="button" class="manage__btn manage__btn--primary" @click="emit('edit')">编辑发布</button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { SeatSwapRequest } from '../types/seatSwap'
import { formatSeatSwapSeatLabel } from '../utils/stadiumRegions'

const props = defineProps<{
  visible: boolean
  request?: SeatSwapRequest | null
  statusLabel: string
  desiredSummary: string
  cancelMode: boolean
  cancelReason: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:cancelMode', value: boolean): void
  (e: 'update:cancelReason', value: string): void
  (e: 'edit'): void
  (e: 'delete'): void
  (e: 'submit-matched-cancel'): void
}>()

const seatLabel = computed(() => (props.request ? formatSeatSwapSeatLabel(props.request) : ''))

const reasonText = computed({
  get: () => props.cancelReason,
  set: (value: string) => emit('update:cancelReason', value),
})

function handleClose(): void {
  emit('update:visible', false)
}
</script>

<style scoped>
.manage {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.manage__scrim {
  position: absolute;
  inset: 0;
  background: rgba(15, 17, 22, 0.46);
  backdrop-filter: blur(2px);
}

.manage__panel {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 520px;
  max-height: 80vh;
  border-radius: 14px;
  border: 1px solid rgba(238, 233, 224, 0.95);
  background: linear-gradient(180deg, rgba(255, 251, 242, 1), rgba(255, 255, 255, 0.99));
  box-shadow: 0 16px 36px rgba(12, 14, 20, 0.24);
  overflow: hidden;
}

.manage__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 22px 6px;
}

.manage__head-main {
  display: grid;
  gap: 3px;
}

.manage__kicker {
  color: #8f7c5f;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 1.5px;
}

.manage__title {
  color: #17181c;
  font-size: 18px;
  font-weight: 800;
}

.manage__close {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: linear-gradient(180deg, rgba(255, 251, 242, 0.98), rgba(248, 241, 227, 0.94));
  color: #9c855c;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.manage__body {
  flex: 1;
  min-height: 0;
  padding: 8px 22px 4px;
  box-sizing: border-box;
  overflow-y: auto;
}

.manage__status {
  display: flex;
  align-items: center;
  gap: 5px;
  margin-bottom: 12px;
}

.manage__status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: linear-gradient(180deg, #f6b44e, #d89b34);
  box-shadow: 0 0 0 3px rgba(216, 155, 52, 0.14);
}

.manage__status-label {
  color: #17181c;
  font-size: 13px;
  font-weight: 700;
}

.manage__row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
}

.manage__label {
  flex-shrink: 0;
  width: 64px;
  color: #988f84;
  font-size: 12px;
}

.manage__seat {
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 700;
}

.manage__seat--current {
  background: linear-gradient(180deg, #f7efe1, #f1e3ca);
  border: 1px solid rgba(220, 201, 165, 0.6);
  color: #927445;
}

.manage__seat--desired {
  background: linear-gradient(180deg, #eaf8ef, #dff1e6);
  border: 1px solid rgba(29, 138, 85, 0.3);
  color: #167348;
}

.manage__contact {
  display: grid;
  gap: 2px;
}

.manage__contact-line {
  color: #5b5f6a;
  font-size: 13px;
}

.manage__cancel-box {
  display: grid;
  gap: 5px;
  margin-top: 12px;
}

.manage__textarea {
  width: 100%;
  height: 80px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid rgba(230, 220, 198, 0.92);
  background: rgba(255, 251, 242, 0.5);
  color: #17181c;
  font-size: 13px;
  box-sizing: border-box;
  outline: none;
  resize: none;
  font-family: inherit;
}

.manage__textarea::placeholder {
  color: #988f84;
  font-size: 12px;
}

.manage__footer {
  display: flex;
  gap: 12px;
  padding: 14px 22px 18px;
  border-top: 1px solid rgba(238, 233, 224, 0.95);
}

.manage__btn {
  flex: 1;
  padding: 9px 0;
  border-radius: 999px;
  border: 0;
  font-size: 14px;
  font-weight: 700;
  text-align: center;
  cursor: pointer;
}

.manage__btn--primary {
  background: linear-gradient(180deg, #20242c, #191d26);
  color: #fff;
  box-shadow: 0 4px 10px rgba(21, 22, 27, 0.16);
}

.manage__btn--ghost {
  background: #fff;
  border: 1px solid rgba(32, 36, 44, 0.16);
  color: #5b5f6a;
}

.manage__btn--danger {
  background: #dd524d;
  color: #fff;
}

.manage__btn:active {
  transform: scale(0.98);
}

@media (max-width: 768px) {
  .manage {
    padding: 0;
    align-items: flex-end;
  }

  .manage__panel {
    max-width: none;
    max-height: 88vh;
    border-radius: 14px 14px 0 0;
  }
}
</style>
