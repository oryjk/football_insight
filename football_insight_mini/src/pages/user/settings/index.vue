<template>
  <view class="settings-page">
    <view class="settings-section">
      <view class="settings-item">
        <text class="settings-item__label">当前版本</text>
        <text class="settings-item__value">{{ MINI_PROGRAM_VERSION }}</text>
      </view>

      <view class="settings-item">
        <text class="settings-item__label">审核状态</text>
        <text class="settings-item__value" :class="{ 'settings-item__value--reviewing': isReviewing }">{{ statusText }}</text>
      </view>

      <view class="settings-item settings-item--stacked">
        <view class="settings-item__row">
          <text class="settings-item__label">切换审核状态</text>
          <text class="settings-item__caption">切换后全量用户的会员/支付等入口显隐会立即变化；仅管理员账号可切换。</text>
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

    <!-- #ifdef H5 -->
    <view class="settings-section">
      <view class="settings-item settings-item--stacked" v-if="h5TestUsers.length">
        <view class="settings-item__row">
          <text class="settings-item__label">H5 测试登录</text>
          <text class="settings-item__caption">一键以测试身份登录，用于在 H5 上验证需要登录的场景。</text>
        </view>

        <view class="settings-options settings-options--stacked">
          <view
            v-for="user in h5TestUsers"
            :key="user.id"
            class="settings-option"
            @click="handleH5TestLogin(user)"
          >
            <text>{{ h5TestUserLabel(user) }}</text>
          </view>
        </view>
      </view>
    </view>
    <!-- #endif -->
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { getMiniReviewStatus, putMiniReviewReviewStatus } from '../../../api/miniReview'
import { MINI_PROGRAM_VERSION } from '../../../api/system'
import { extractApiErrorMessage } from '../../../utils/apiError'
import type { H5TestLoginUser } from '../../../types/auth'

// #ifdef H5
import { listH5TestLoginUsers, loginAsH5TestUser } from '../../../api/auth'

const h5TestUsers = ref<H5TestLoginUser[]>([])

function h5TestUserLabel(user: H5TestLoginUser): string {
  return user.display_name || user.account_identifier
}

async function loadH5TestUsers() {
  try {
    const result = await listH5TestLoginUsers()
    h5TestUsers.value = result.items
  } catch {
    // 后端未开放（403）时隐藏面板
    h5TestUsers.value = []
  }
}

async function handleH5TestLogin(user: H5TestLoginUser) {
  try {
    await loginAsH5TestUser(user.id)
    uni.showToast({ title: `已切换为 ${h5TestUserLabel(user)}`, icon: 'none' })
    setTimeout(() => {
      uni.reLaunch({ url: '/pages/user/index' })
    }, 600)
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '测试登录失败'), icon: 'none' })
  }
}

loadH5TestUsers()
// #endif

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
.settings-page {
  min-height: 100vh;
  background: var(--fi-color-page);
  padding: 24rpx 24rpx 40rpx;
}

.settings-section {
  display: grid;
  gap: 18rpx;
}

.settings-item {
  padding: 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: var(--fi-primitive-white);
  display: grid;
  gap: 10rpx;
}

.settings-item--stacked {
  gap: 18rpx;
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
  font-size: 26rpx;
  font-weight: 700;
}

.settings-item__value--reviewing {
  color: var(--fi-color-primary);
}

.settings-options {
  display: flex;
  gap: 14rpx;
}

.settings-options--stacked {
  flex-direction: column;
}

.settings-option {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 20rpx 24rpx;
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
