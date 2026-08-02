<template>
  <view class="fi-login-float">
    <view class="fi-login-float__copy">
      <text class="fi-login-float__title">{{ title }}</text>
      <text class="fi-login-float__desc">{{ desc }}</text>
    </view>
    <button
      class="fi-login-float__action"
      :disabled="disabled"
      @click="handleAction"
    >
      {{ actionText }}
    </button>
  </view>
</template>

<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    title?: string
    desc?: string
    actionText?: string
    disabled?: boolean
  }>(),
  {
    title: '请先登录',
    desc: '重新登录后，可以继续关注主队和使用会员功能。',
    actionText: '去登录',
    disabled: false,
  },
)

const emit = defineEmits<{
  (e: 'action'): void
}>()

function handleAction(): void {
  if (props.disabled) {
    return
  }

  emit('action')
}
</script>

<style scoped lang="css">
.fi-login-float {
  position: fixed;
  left: 28rpx;
  right: 28rpx;
  bottom: calc(var(--window-bottom, 0px) + 12rpx);
  z-index: 40;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 20rpx;
  padding: 22rpx 24rpx;
  border-radius: 28rpx;
  background: rgba(21, 23, 29, 0.96);
  box-shadow: 0 20rpx 46rpx rgba(16, 18, 24, 0.24);
}

.fi-login-float__copy {
  min-width: 0;
  display: grid;
  gap: 6rpx;
}

.fi-login-float__title,
.fi-login-float__desc {
  display: block;
}

.fi-login-float__title {
  color: #ffffff;
  font-size: 28rpx;
  font-weight: 800;
}

.fi-login-float__desc {
  color: rgba(255, 255, 255, 0.72);
  font-size: 22rpx;
  line-height: 1.35;
}

.fi-login-float__action {
  min-width: 148rpx;
  height: 64rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 999rpx;
  background: #ffffff;
  color: #15171d;
  font-size: 26rpx;
  font-weight: 800;
  line-height: 1;
}

.fi-login-float__action::after {
  border: none;
}
</style>
