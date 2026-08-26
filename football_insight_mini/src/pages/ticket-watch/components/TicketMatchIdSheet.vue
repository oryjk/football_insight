<template>
  <view v-if="visible" class="match-id-mask" @tap="close">
    <view class="match-id-card" @tap.stop>
      <view class="match-id-card__head">
        <view>
          <text class="match-id-card__kicker">比赛 ID</text>
          <text class="match-id-card__title">{{ state === 'unlocked' ? '已解锁' : '获取比赛 ID' }}</text>
        </view>
        <button class="match-id-card__close" :disabled="paying" @tap="close">×</button>
      </view>

      <text class="match-id-card__match">{{ matchLabel }}</text>

      <view v-if="state === 'unlocked'" class="match-id-card__body">
        <view class="match-id-value-panel">
          <text class="match-id-value-panel__value">{{ matchId }}</text>
          <button class="match-id-value-panel__copy" @tap="copy">复制</button>
        </view>
        <text class="match-id-card__source">{{ sourceLabel }}</text>
      </view>

      <view v-else class="match-id-card__body">
        <text class="match-id-card__copy">V6 及以上会员可免费查看，或支付 ¥5 解锁本场比赛 ID。</text>
        <view class="match-id-actions">
          <button class="match-id-actions__button match-id-actions__button--ghost" :disabled="paying" @tap="upgrade">
            升级到 V6
          </button>
          <button class="match-id-actions__button match-id-actions__button--pay" :disabled="paying" @tap="pay">
            {{ paying ? '处理中...' : '¥5 解锁本场' }}
          </button>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { buildMatchIdSourceLabel } from '../helpers'

const props = defineProps<{
  visible: boolean
  matchId: number | null
  matchLabel: string
  state: 'loading' | 'locked' | 'unlocked' | 'paying'
  via: 'membership' | 'purchase' | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'pay'): void
  (e: 'upgrade'): void
}>()

const paying = computed(() => props.state === 'paying')

const sourceLabel = computed(() => buildMatchIdSourceLabel(props.via))

function close(): void {
  if (paying.value) {
    return
  }

  emit('close')
}

function pay(): void {
  if (paying.value) {
    return
  }

  emit('pay')
}

function upgrade(): void {
  if (paying.value) {
    return
  }

  emit('upgrade')
}

function copy(): void {
  if (props.matchId === null) {
    return
  }

  uni.setClipboardData({
    data: String(props.matchId),
    success: () => {
      uni.showToast({ title: '已复制', icon: 'none' })
    },
  })
}
</script>

<style scoped lang="css">
.match-id-mask {
  position: fixed;
  inset: 0;
  bottom: var(--window-bottom, 0px);
  z-index: 90;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 32rpx;
  animation: fi-overlay-fade-in 180ms ease both;
}

.match-id-card {
  width: 100%;
  max-width: 640rpx;
  box-sizing: border-box;
  border-radius: var(--fi-radius-lg);
  background: rgba(255, 255, 255, 0.98);
  padding: 28rpx 26rpx 32rpx;
  box-shadow: 0 24rpx 56rpx rgba(12, 14, 20, 0.12);
  animation: fi-fade-in-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.match-id-card__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12rpx;
}

.match-id-card__kicker {
  display: block;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: 700;
  letter-spacing: 3rpx;
}

.match-id-card__title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: var(--fi-font-44);
  line-height: 1.08;
  font-weight: 800;
}

.match-id-card__close {
  padding: 0;
  width: 56rpx;
  height: 56rpx;
  line-height: 52rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-32);
}

.match-id-card__close::after {
  border: none;
}

.match-id-card__match {
  display: block;
  margin-top: 14rpx;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: 1.5;
}

.match-id-card__body {
  margin-top: 22rpx;
  display: flex;
  flex-direction: column;
  gap: 18rpx;
}

.match-id-value-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
  padding: 26rpx 24rpx;
  border-radius: var(--fi-radius-md);
  border: 1rpx solid var(--fi-border-subtle);
  background: rgba(248, 249, 251, 0.85);
}

.match-id-value-panel__value {
  color: var(--fi-primitive-ink);
  font-size: var(--fi-font-48);
  font-weight: 800;
  letter-spacing: 2rpx;
  line-height: 1;
  word-break: break-all;
}

.match-id-value-panel__copy {
  flex-shrink: 0;
  margin: 0;
  min-width: 112rpx;
  height: 64rpx;
  padding: 0 24rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-24);
  font-weight: 700;
  line-height: 64rpx;
}

.match-id-value-panel__copy::after {
  border: none;
}

.match-id-card__source {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.match-id-card__copy {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: 1.6;
}

.match-id-actions {
  display: flex;
  gap: 16rpx;
}

.match-id-actions__button {
  flex: 1;
  margin: 0;
  height: 84rpx;
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-26);
  font-weight: 700;
  line-height: 84rpx;
}

.match-id-actions__button::after {
  border: none;
}

.match-id-actions__button--ghost {
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}

.match-id-actions__button--pay {
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
}

.match-id-actions__button[disabled] {
  opacity: 0.55;
}
</style>
