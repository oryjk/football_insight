<template>
  <view class="sheet-mask" @tap="emit('close')">
    <view class="sheet-card" @tap.stop>
      <view class="section-heading">
        <view>
          <text class="section-kicker">首次微信登录</text>
          <text class="section-title">补充头像和昵称</text>
        </view>
        <text class="meta-note">完成后直接登录</text>
      </view>

      <text class="account-form-panel__summary">
        完成一次绑定后，后续就能直接用微信进入。邀请码和推荐码都可选。
      </text>

      <view class="mini-wechat-bind-form">
        <button
          class="avatar-picker"
          open-type="chooseAvatar"
          @tap.stop
          @chooseavatar="handleChooseAvatar"
        >
          <image
            v-if="form.avatarPreviewUrl"
            :src="form.avatarPreviewUrl"
            mode="aspectFill"
            class="avatar-picker__image"
          />
          <text v-else class="avatar-picker__placeholder">选择头像</text>
        </button>

        <input v-model="form.displayName" type="nickname" class="auth-input" placeholder="请输入昵称" />
        <input v-model="form.inviteCode" class="auth-input" placeholder="请输入邀请码（可选）" />
        <input v-model="form.referralCode" class="auth-input" placeholder="请输入推荐码（可选，填推荐人的邀请码）" />
      </view>

      <view class="sheet-actions">
        <button class="primary-action primary-action--ghost" @click="emit('close')">取消</button>
        <button class="primary-action" @click="submit">完成绑定</button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { MiniWechatBindingRequiredResponse } from '../../../types/auth'

const props = defineProps<{
  bindState: MiniWechatBindingRequiredResponse | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'bind', payload: {
    bindToken: string
    inviteCode: string
    referralCode: string
    displayName: string
    avatarDataUrl: string
  }): void
}>()

const form = reactive({
  inviteCode: '',
  referralCode: '',
  displayName: '',
  avatarPreviewUrl: '',
})

watch(
  () => props.bindState,
  (state) => {
    if (!state) {
      return
    }

    form.inviteCode = ''
    form.referralCode = ''
    form.displayName = normalizeTextValue(state.display_name)
    form.avatarPreviewUrl = normalizeTextValue(state.avatar_url)
  },
)

function normalizeTextValue(value: unknown): string {
  return typeof value === 'string' ? value.trim() : ''
}

function handleChooseAvatar(event: { detail?: { avatarUrl?: unknown } }): void {
  const avatarUrl = normalizeTextValue(event.detail?.avatarUrl)
  if (!avatarUrl) {
    return
  }

  form.avatarPreviewUrl = avatarUrl
}

function inferMimeType(filePath: string): string {
  const lowerPath = filePath.toLowerCase()
  if (lowerPath.endsWith('.jpg') || lowerPath.endsWith('.jpeg')) {
    return 'image/jpeg'
  }
  if (lowerPath.endsWith('.webp')) {
    return 'image/webp'
  }
  return 'image/png'
}

function readAvatarAsDataUrl(filePath: string): Promise<string> {
  const normalizedPath = filePath.trim()
  if (!normalizedPath) {
    return Promise.reject(new Error('请先选择头像'))
  }

  if (normalizedPath.startsWith('data:image/')) {
    return Promise.resolve(normalizedPath)
  }

  return new Promise((resolve, reject) => {
    uni.getFileSystemManager().readFile({
      filePath: normalizedPath,
      encoding: 'base64',
      success: (result) => {
        const base64 = typeof result.data === 'string' ? result.data : ''
        if (!base64) {
          reject(new Error('头像读取失败'))
          return
        }

        resolve(`data:${inferMimeType(normalizedPath)};base64,${base64}`)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '头像读取失败'))
      },
    })
  })
}

async function submit(): Promise<void> {
  if (!props.bindState) {
    return
  }

  try {
    const avatarDataUrl = await readAvatarAsDataUrl(form.avatarPreviewUrl)
    emit('bind', {
      bindToken: props.bindState.bind_token,
      inviteCode: form.inviteCode.trim(),
      referralCode: form.referralCode.trim(),
      displayName: form.displayName.trim(),
      avatarDataUrl,
    })
  } catch (error) {
    uni.showToast({ title: error instanceof Error ? error.message : '绑定失败', icon: 'none' })
  }
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
