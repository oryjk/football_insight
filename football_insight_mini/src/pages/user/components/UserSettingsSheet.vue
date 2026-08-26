<template>
  <view class="sheet-mask" @tap="emit('close')">
    <view class="sheet-card" @tap.stop>
      <view class="section-heading">
        <view>
          <text class="section-kicker">设置</text>
          <text class="section-title">运营工具</text>
        </view>
        <button class="sheet-close" @click="emit('close')">关闭</button>
      </view>

      <view class="settings-item">
        <text class="settings-item__label">当前版本审核状态</text>
        <text class="settings-item__value">{{ statusText }}</text>
      </view>

      <view class="settings-item settings-item--stacked">
        <view class="settings-item__row">
          <text class="settings-item__label">切换审核状态</text>
          <text class="settings-item__caption">影响全量用户的会员/支付等入口显隐</text>
        </view>

        <view class="settings-options">
          <view
            class="settings-option"
            :class="{ 'settings-option--active': isReviewing }"
            @click="!isReviewing && handleToggle(true)"
          >
            <text>审核中</text>
          </view>
          <view
            class="settings-option"
            :class="{ 'settings-option--active': !isReviewing }"
            @click="isReviewing && handleToggle(false)"
          >
            <text>已过审</text>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { getMiniReviewStatus, putMiniReviewReviewStatus } from '../../../api/miniReview'
import { MINI_PROGRAM_VERSION } from '../../../api/system'
import { extractApiErrorMessage } from '../../../utils/apiError'

const emit = defineEmits<{
  (e: 'close'): void
}>()

const isReviewing = ref(false)
const statusText = ref('')

async function loadStatus() {
  statusText.value = '查询中…'
  try {
    const status = await getMiniReviewStatus()
    isReviewing.value = status.is_reviewing
    statusText.value = status.status_text
  } catch (error) {
    console.warn('[user-settings] load review status failed', error)
    statusText.value = '查询失败'
  }
}

function handleToggle(nextReviewing: boolean) {
  const nextLabel = nextReviewing ? '审核中' : '已过审'
  uni.showModal({
    title: '切换审核状态',
    content: `当前版本 ${MINI_PROGRAM_VERSION} 将切换为「${nextLabel}」，全量用户的会员/支付等入口显隐会立即变化。`,
    success: ({ confirm }) => {
      if (!confirm) {
        return
      }
      void applyToggle(nextReviewing)
    },
  })
}

async function applyToggle(nextReviewing: boolean) {
  try {
    const status = await putMiniReviewReviewStatus(undefined, undefined, nextReviewing)
    isReviewing.value = status.is_reviewing
    statusText.value = status.status_text
    uni.showToast({ title: nextReviewing ? '已切为审核中' : '已切为已过审', icon: 'none' })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '切换失败'), icon: 'none' })
  }
}

loadStatus()
</script>

<style scoped lang="css">
.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(var(--fi-primitive-ink-rgb), 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
}

.sheet-card {
  width: 100%;
  max-height: 78vh;
  border-radius: 36rpx 36rpx 0 0;
  background: rgba(255, 255, 255, 0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12, 14, 20, 0.12);
  overflow-y: auto;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12rpx;
}

.section-heading > view {
  display: grid;
  gap: 8rpx;
  min-width: 0;
}

.section-kicker {
  display: block;
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 3rpx;
}

.section-title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 44rpx;
  line-height: 1.16;
  font-weight: 800;
}

.sheet-close {
  display: inline-flex;
  margin: 0 0 0 auto;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 12rpx 18rpx;
  border-radius: 999rpx;
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: 24rpx;
}

.settings-item {
  margin-top: 22rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: var(--fi-primitive-white);
  display: grid;
  gap: 10rpx;
}

.settings-item--stacked {
  gap: 16rpx;
}

.settings-item__row {
  display: grid;
  gap: 6rpx;
}

.settings-item__label {
  color: var(--fi-color-text-strong);
  font-size: 28rpx;
  font-weight: 700;
}

.settings-item__caption {
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
  line-height: 1.4;
}

.settings-item__value {
  color: var(--fi-color-primary);
  font-size: 24rpx;
  font-weight: 700;
}

.settings-options {
  display: flex;
  gap: 14rpx;
}

.settings-option {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 18rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: 26rpx;
  font-weight: 700;
  line-height: 1;
}

.settings-option--active {
  border-color: rgba(var(--fi-primitive-red-rgb), 0.4);
  background: rgba(var(--fi-primitive-red-rgb), 0.08);
  color: var(--fi-color-primary);
}
</style>
