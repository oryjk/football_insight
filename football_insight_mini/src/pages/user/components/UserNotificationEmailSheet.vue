<template>
  <FiBottomSheet
    :visible="visible"
    eyebrow="邮箱提醒"
    title="编辑回流提醒邮箱"
    fit-content
    :z-index="80"
    respect-window-bottom
    :close-on-mask="!saving"
    @update:visible="close"
  >
    <text class="account-form-panel__summary">订阅回流提醒后，新增回流会发送到这个邮箱。</text>
    <input v-model="draft" class="auth-input" type="text" placeholder="请输入邮箱地址" />
    <view class="sheet-actions">
      <button class="primary-action primary-action--ghost" :disabled="saving" @click="close">取消</button>
      <button class="primary-action" :disabled="saving" @click="submit">
        {{ saving ? '保存中...' : '保存邮箱' }}
      </button>
    </view>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
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
.account-form-panel__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: var(--fi-font-28);
  line-height: 1.7;
}

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

.sheet-actions {
  margin-top: 24rpx;
  display: flex;
  gap: 16rpx;
}
</style>
