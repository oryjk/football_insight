<template>
  <view class="skeleton-stack">
    <view class="hero-card account-hero skeleton-panel skeleton-account-hero">
      <view class="skeleton-copy-group">
        <view class="skeleton-line skeleton-line--kicker" />
        <view class="skeleton-line skeleton-line--title" />
      </view>

      <view class="skeleton-profile-row">
        <view class="skeleton-line skeleton-line--avatar" />
        <view class="skeleton-profile-body">
          <view class="skeleton-line skeleton-line--profile-name" />
          <view class="skeleton-line skeleton-line--profile-badge" />
        </view>
        <view class="skeleton-line skeleton-line--medal" />
      </view>

      <view class="skeleton-invite-row">
        <view class="skeleton-copy-group">
          <view class="skeleton-line skeleton-line--label" />
          <view class="skeleton-line skeleton-line--invite-code" />
        </view>
        <view class="skeleton-button skeleton-button--compact" />
      </view>
    </view>

    <view class="panel skeleton-panel">
      <view class="skeleton-line skeleton-line--section" />
      <view class="skeleton-account-grid">
        <view
          v-for="index in 5"
          :key="`user-account-skeleton-${index}`"
          class="skeleton-account-cell"
        >
          <view class="skeleton-line skeleton-line--account-icon" />
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--label" />
            <view class="skeleton-line skeleton-line--account-value" />
          </view>
        </view>
      </view>
    </view>

    <view class="panel skeleton-panel">
      <view class="skeleton-line skeleton-line--section" />
      <view class="skeleton-privilege-grid">
        <view
          v-for="index in 4"
          :key="`user-privilege-skeleton-${index}`"
          class="skeleton-privilege-card"
        >
          <view class="skeleton-line skeleton-line--privilege-icon" />
          <view class="skeleton-copy-group">
            <view class="skeleton-line skeleton-line--privilege-title" />
            <view class="skeleton-line skeleton-line--privilege-caption" />
          </view>
        </view>
      </view>
    </view>

    <view class="panel skeleton-panel">
      <view class="skeleton-line skeleton-line--section" />
      <view class="skeleton-line skeleton-line--body" />
      <view class="skeleton-line skeleton-line--body skeleton-line--body-short" />
    </view>
  </view>
</template>

<script setup lang="ts">
</script>

<style scoped lang="css">
/* 卡片基类（与页面 .panel 同款玻璃卡）。 */
.hero-card,
.panel {
  position: relative;
  z-index: 1;
  background: rgba(255, 255, 255, 0.72);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-20);
  border: 2rpx solid rgba(255, 255, 255, 0.55);
  box-shadow: 0 20rpx 48rpx rgba(26, 28, 36, 0.06);
  backdrop-filter: blur(18rpx);
  -webkit-backdrop-filter: blur(18rpx);
}

.skeleton-stack {
  position: relative;
  z-index: 1;
  display: grid;
  gap: 16rpx;
}

.skeleton-panel {
  position: relative;
  overflow: hidden;
}

.skeleton-panel::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(
    90deg,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.66) 48%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: translateX(-100%);
  animation: skeleton-shimmer 1.35s ease-in-out infinite;
  pointer-events: none;
  z-index: 2;
}

.skeleton-copy-group {
  display: grid;
  gap: 10rpx;
  min-width: 0;
}

.skeleton-line,
.skeleton-pill,
.skeleton-button,
.skeleton-account-cell,
.skeleton-privilege-card {
  background: linear-gradient(180deg, rgba(248, 246, 239, 0.96), rgba(235, 230, 216, 0.9));
  border: 2rpx solid rgba(232, 222, 198, 0.72);
}

.skeleton-line {
  border-radius: var(--fi-radius-round);
}

.skeleton-line--kicker {
  width: 150rpx;
  height: 24rpx;
}

.skeleton-line--title {
  width: 330rpx;
  max-width: 100%;
  height: 44rpx;
  border-radius: 22rpx;
}

.skeleton-line--section {
  width: 220rpx;
  max-width: 100%;
  height: 38rpx;
  border-radius: 22rpx;
}

.skeleton-line--label {
  width: 112rpx;
  height: 22rpx;
}

.skeleton-line--body {
  width: 90%;
  height: 26rpx;
  margin-top: 20rpx;
}

.skeleton-line--body-short {
  width: 62%;
}

.skeleton-pill {
  width: 150rpx;
  height: 52rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-pill--short {
  width: 112rpx;
}

.skeleton-button {
  width: 150rpx;
  height: 60rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-button--compact {
  width: 160rpx;
  height: 58rpx;
}

.skeleton-account-hero {
  min-height: 330rpx;
}

.skeleton-profile-row {
  margin-top: 28rpx;
  display: flex;
  align-items: center;
  gap: 22rpx;
}

.skeleton-line--avatar {
  width: 122rpx;
  height: 122rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-profile-body {
  flex: 1;
  min-width: 0;
  display: grid;
  gap: 18rpx;
}

.skeleton-line--profile-name {
  width: 260rpx;
  height: 46rpx;
  border-radius: 22rpx;
}

.skeleton-line--profile-badge {
  width: 180rpx;
  height: 46rpx;
  border-radius: var(--fi-radius-round);
}

.skeleton-line--medal {
  width: 146rpx;
  height: 146rpx;
  border-radius: 42rpx;
  flex-shrink: 0;
  background: linear-gradient(180deg, rgba(246, 229, 180, 0.98), rgba(220, 181, 91, 0.82));
}

.skeleton-invite-row {
  margin-top: 26rpx;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18rpx;
  padding: 18rpx 20rpx;
  border-radius: var(--fi-radius-lg);
  border: 2rpx solid rgba(232, 212, 166, 0.82);
  background: rgba(255, 252, 244, 0.74);
}

.skeleton-line--invite-code {
  width: 310rpx;
  height: 34rpx;
  border-radius: var(--fi-radius-sm);
}

.skeleton-account-grid {
  margin-top: 24rpx;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border-radius: var(--fi-radius-lg);
  overflow: hidden;
  border: 2rpx solid rgba(234, 226, 209, 0.92);
}

.skeleton-account-cell {
  min-height: 104rpx;
  padding: 20rpx;
  display: flex;
  align-items: center;
  gap: 16rpx;
  border-width: 0 2rpx 2rpx 0;
  border-radius: 0;
}

.skeleton-account-cell:nth-child(2n) {
  border-right-width: 0;
}

.skeleton-account-cell:nth-last-child(1) {
  grid-column: span 2;
  border-bottom-width: 0;
}

.skeleton-line--account-icon {
  width: 54rpx;
  height: 54rpx;
  border-radius: var(--fi-radius-round);
  flex-shrink: 0;
}

.skeleton-line--account-value {
  width: 130rpx;
  height: 30rpx;
  border-radius: var(--fi-radius-sm);
}

.skeleton-privilege-grid {
  margin-top: 24rpx;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 12rpx;
}

.skeleton-privilege-card {
  min-height: 150rpx;
  padding: 18rpx 12rpx;
  border-radius: var(--fi-radius-md);
  display: grid;
  justify-items: center;
  align-content: center;
  gap: 12rpx;
}

.skeleton-line--privilege-icon {
  width: 52rpx;
  height: 52rpx;
  border-radius: var(--fi-radius-round);
}

.skeleton-line--privilege-title {
  width: 82rpx;
  height: 26rpx;
}

.skeleton-line--privilege-caption {
  width: 98rpx;
  height: 22rpx;
}

@keyframes skeleton-shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}
</style>
