<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="已匹配成功"
    title="换座对象"
    fit-content
    compact-footer
    respect-window-bottom
    @update:visible="emit('update:visible', $event)"
    @close="emit('close')"
  >
    <view v-if="candidate" class="matched">
      <!-- 联系方式只在这里展示，换座池列表里不露出。 -->
      <view class="matched__seats">
        <SeatSwapSeatThumb
          :current-key="candidate.current_region_key"
          :desired-keys="desiredKeys"
        />
        <view class="matched__seats-info">
          <text class="matched__name">{{ candidate.display_name }}</text>
          <view class="matched__seat">
            <text class="matched__seat-label">当前</text>
            <text class="matched__seat-value">{{ formatSeatSwapSeatLabel(candidate) }}</text>
          </view>
          <view class="matched__seat">
            <text class="matched__seat-label">目标</text>
            <text class="matched__seat-value matched__seat-value--desired">{{ desiredSeatText }}</text>
          </view>
        </view>
      </view>

      <view class="matched__contacts">
        <view v-for="item in contactItems" :key="item.label" class="matched__contact">
          <text class="matched__contact-label">{{ item.label }}</text>
          <text class="matched__contact-value">{{ item.value }}</text>
          <button class="matched__copy" hover-class="matched__copy--pressed" @tap="copyValue(item)">复制</button>
        </view>
        <text v-if="!contactItems.length" class="matched__empty">对方没有留下联系方式</text>
      </view>
    </view>

    <template #footer>
      <view class="matched__actions">
        <button class="matched__cancel" @tap="emit('cancel-match', candidate?.request_id ?? '')">取消匹配</button>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import SeatSwapSeatThumb from './SeatSwapSeatThumb.vue'
import type { SeatSwapCandidate } from '../../../types/seatSwap'
import { formatSeatSwapDesiredSeats, formatSeatSwapSeatLabel } from '../../../utils/stadiumRegions'

const props = defineProps<{
  visible: boolean
  candidate: SeatSwapCandidate | null
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
  (e: 'cancel-match', requestId: string): void
}>()

const desiredKeys = computed(() => props.candidate?.desired_seats.map((seat) => seat.region_key) ?? [])
const desiredSeatText = computed(() => formatSeatSwapDesiredSeats(props.candidate?.desired_seats ?? []))
const contactItems = computed(() => {
  const contact = props.candidate?.contact
  const items: Array<{ label: string; value: string }> = []
  if (contact?.wechat_id) items.push({ label: '微信号', value: contact.wechat_id })
  if (contact?.phone_number) items.push({ label: '手机号', value: contact.phone_number })
  return items
})

function copyValue(item: { label: string; value: string }): void {
  uni.setClipboardData({
    data: item.value,
    success: () => {
      uni.showToast({ title: `${item.label}已复制`, icon: 'none' })
    },
  })
}
</script>

<style scoped>
.matched {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-16);
  padding: var(--fi-space-12) 0 var(--fi-space-14);
}

/* 绿色主卡：对方的座位信息 */
.matched__seats {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
  padding: var(--fi-space-18);
  border-radius: var(--fi-radius-md);
  background: var(--fi-primitive-green);
  border: var(--fi-border-width) solid var(--fi-primitive-green-deep);
}

.matched__seats-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
}

.matched__name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
}

.matched__seat {
  display: flex;
  align-items: center;
  align-self: flex-start;
  max-width: 100%;
  overflow: hidden;
  border-radius: var(--fi-radius-sm);
  background: rgba(var(--fi-primitive-white-rgb), 0.16);
  color: var(--fi-primitive-white);
}

.matched__seat-label {
  flex-shrink: 0;
  padding: var(--fi-space-6) var(--fi-space-10) var(--fi-space-6) var(--fi-space-12);
  background: rgba(var(--fi-primitive-white-rgb), 0.14);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.matched__seat-value {
  min-width: 0;
  padding: var(--fi-space-6) var(--fi-space-14) var(--fi-space-6) var(--fi-space-10);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.matched__seat-value--desired {
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

/* 联系方式：白底行，一行一项，右侧一键复制 */
.matched__contacts {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.matched__contact {
  display: flex;
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-14) var(--fi-space-14) var(--fi-space-14) var(--fi-space-22);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-surface);
  border: var(--fi-border-width) solid var(--fi-color-border-chip);
}

.matched__contact-label {
  flex-shrink: 0;
  width: 108rpx;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.matched__contact-value {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
}

.matched__copy {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-10) var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-green-soft);
  color: var(--fi-primitive-green-deep);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.matched__copy::after {
  border: 0;
}

.matched__copy--pressed {
  opacity: var(--fi-primitive-opacity-pressed);
}

.matched__empty {
  padding: var(--fi-space-20);
  text-align: center;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.matched__actions {
  width: 100%;
}

.matched__cancel {
  width: 100%;
  padding: var(--fi-space-20) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-red-rgb), 0.06);
  border: var(--fi-border-width) solid rgba(var(--fi-primitive-red-rgb), 0.3);
  color: var(--fi-color-primary-deep);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.matched__cancel::after {
  border: 0;
}
</style>
