<template>
  <view v-if="visible" class="sheet-mask" @tap="close">
    <view class="sheet-card" @tap.stop>
      <view class="section-heading">
        <view>
          <text class="section-kicker">邮箱提醒</text>
          <text class="section-title">编辑回流提醒邮箱</text>
        </view>
      </view>
      <text class="account-form-panel__summary">订阅回流提醒后，新增回流会发送到这个邮箱。</text>
      <input v-model="draft" class="auth-input" type="text" placeholder="请输入邮箱地址" />
      <view class="sheet-actions">
        <button class="primary-action primary-action--ghost" :disabled="saving" @click="close">取消</button>
        <button class="primary-action" :disabled="saving" @click="submit">
          {{ saving ? '保存中...' : '保存邮箱' }}
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { isValidNotificationEmail } from '../helpers'

const props = defineProps<{
  visible: boolean
  email: string
  saving: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'save', email: string): void
}>()

const draft = ref('')

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      draft.value = props.email
    }
  },
)

function close(): void {
  if (props.saving) {
    return
  }

  emit('update:visible', false)
}

function submit(): void {
  const email = draft.value.trim()
  if (!isValidNotificationEmail(email)) {
    uni.showToast({ title: '请填写有效邮箱', icon: 'none' })
    return
  }

  emit('save', email)
}
</script>

<style scoped lang="css">
.hero-card__top, .section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.hero-card__top, .section-heading { align-items: flex-start; gap: 12rpx; }

.eyebrow, .section-kicker {
  margin: 0;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: 700;
  letter-spacing: 3rpx;
}

.hero-card__title, .section-title {
  display: block;
  margin-top: 10rpx;
  color: #2a2c31;
  font-size: var(--fi-font-48);
  line-height: 1.08;
  font-weight: 800;
}

.section-title { font-size: var(--fi-font-44); }

.hero-card__summary, .account-form-panel__summary, .user-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: 1.7;
}

.meta-note {
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  flex: 0 0 auto;
  flex-shrink: 0;
  max-width: 100%;
  white-space: nowrap;
  line-height: 1;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: 700;
  letter-spacing: 1rpx;
  padding: 8rpx 0 0;
}

.meta-note::before {
  content: '';
  width: 12rpx;
  height: 12rpx;
  margin-right: 10rpx;
  border-radius: var(--fi-radius-round);
  background: #9aa0aa;
  box-shadow: 0 0 0 6rpx rgba(154, 160, 170, 0.12);
}

.meta-note--hero { padding-top: 14rpx; }

.auth-input {
  width: 100%;
  height: 92rpx;
  padding: 0 24rpx;
  border-radius: 22rpx;
  border: 2rpx solid var(--page-border);
  background: #f7f8fb;
  font-size: var(--fi-font-28);
}

.primary-action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  padding: 20rpx 30rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  white-space: nowrap;
  line-height: 1;
}

.primary-action--ghost {
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}

.primary-action[disabled] {
  opacity: 0.5;
}

.sheet-mask {
  position: fixed;
  inset: 0;
  bottom: var(--window-bottom, 0px);
  z-index: 80;
  background: rgba(18, 20, 28, 0.36);
  backdrop-filter: blur(8rpx);
  display: flex;
  align-items: flex-end;
  animation: fi-overlay-fade-in 180ms ease both;
}

.sheet-card {
  width: 100%;
  border-radius: var(--fi-radius-xl) 36rpx 0 0;
  background: rgba(255,255,255,0.98);
  padding: 28rpx 24rpx 40rpx;
  box-shadow: 0 -24rpx 56rpx rgba(12,14,20,0.12);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.password-login-sheet {
  max-width: 720rpx;
  margin: 0 auto;
  box-sizing: border-box;
}

.mini-wechat-bind-form,
.password-login-form {
  margin-top: 22rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}

.avatar-picker {
  width: 144rpx;
  height: 144rpx;
  padding: 0;
  border-radius: var(--fi-radius-round);
  background: linear-gradient(180deg, rgba(247,248,251,0.98), rgba(239,242,248,0.98));
  border: 2rpx solid var(--page-border);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.avatar-picker__image {
  width: 100%;
  height: 100%;
}

.avatar-picker__placeholder {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
</style>
