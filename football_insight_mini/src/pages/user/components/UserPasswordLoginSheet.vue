<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="H5 登录"
    title="账号密码登录"
    fit-content
    :z-index="80"
    respect-window-bottom
    :close-on-mask="!submitting"
    @update:visible="close"
  >
    <view class="password-login-form">
      <input
        v-model="form.accountIdentifier"
        class="auth-input"
        type="text"
        placeholder="请输入账号或手机号"
        confirm-type="next"
      />
      <input
        v-model="form.password"
        class="auth-input"
        type="text"
        password
        placeholder="请输入密码"
        confirm-type="done"
        @confirm="submit"
      />
    </view>

    <view class="sheet-actions">
      <button class="primary-action primary-action--ghost" :disabled="submitting" @click="close">取消</button>
      <button class="primary-action" :disabled="submitting" @click="submit">
        {{ submitting ? '登录中...' : '登录' }}
      </button>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import { reactive } from 'vue'
import { validatePasswordLoginForm } from '../loginHelpers'

const props = defineProps<{
  visible: boolean
  submitting: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'submit', form: { accountIdentifier: string; password: string }): void
}>()

const form = reactive({
  accountIdentifier: '',
  password: '',
})

function close(): void {
  if (props.submitting) {
    return
  }

  emit('update:visible', false)
  form.password = ''
}

function submit(): void {
  const validationError = validatePasswordLoginForm(form)
  if (validationError) {
    uni.showToast({ title: validationError, icon: 'none' })
    return
  }

  emit('submit', { accountIdentifier: form.accountIdentifier, password: form.password })
  form.password = ''
}
</script>

<style scoped lang="css">
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

.password-login-form {
  margin-top: 22rpx;
  display: flex;
  flex-direction: column;
  gap: 16rpx;
}

.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
</style>
