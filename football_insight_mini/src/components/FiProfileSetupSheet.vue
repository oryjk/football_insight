<template>
  <FiBottomSheet
    :visible="visible"
    title="完善头像与昵称"
    eyebrow="互动身份设置"
    fit-content
    compact-footer
    :z-index="90"
    :close-on-mask="!busy"
    respect-window-bottom
    :keyboard-height="keyboardHeight"
    @update:visible="handleVisibleChange"
    @close="handleClose"
  >
    <view class="profile-sheet" @tap.stop>
      <text class="profile-sheet__summary">{{ subtitle }}</text>

      <view class="profile-sheet__avatar-section">
        <!-- #ifdef MP-WEIXIN -->
        <button
          class="profile-sheet__avatar-btn"
          open-type="chooseAvatar"
          :disabled="busy"
          hover-class="profile-sheet__pressed"
          @tap.stop
          @chooseavatar="handleChooseMiniAvatar"
        >
          <image
            v-if="previewAvatar"
            :src="previewAvatar"
            mode="aspectFill"
            class="profile-sheet__avatar-img"
          />
          <view v-else class="profile-sheet__avatar-empty">
            <text class="profile-sheet__avatar-plus">+</text>
            <text class="profile-sheet__avatar-text">上传头像</text>
          </view>
        </button>
        <!-- #endif -->

        <!-- #ifndef MP-WEIXIN -->
        <button
          class="profile-sheet__avatar-btn"
          :disabled="busy"
          hover-class="profile-sheet__pressed"
          @tap.stop="handleChooseH5Avatar"
        >
          <image
            v-if="previewAvatar"
            :src="previewAvatar"
            mode="aspectFill"
            class="profile-sheet__avatar-img"
          />
          <view v-else class="profile-sheet__avatar-empty">
            <text class="profile-sheet__avatar-plus">+</text>
            <text class="profile-sheet__avatar-text">上传头像</text>
          </view>
        </button>
        <!-- #endif -->

        <text class="profile-sheet__avatar-hint">
          {{ previewAvatar ? '点击头像可重新选择' : '点击选择清晰头像' }}
        </text>
      </view>

      <view class="profile-sheet__field">
        <text class="profile-sheet__label">球迷昵称</text>
        <!-- #ifdef MP-WEIXIN -->
        <input
          v-model="nickname"
          type="nickname"
          class="profile-sheet__input"
          maxlength="16"
          :disabled="busy"
          :adjust-position="false"
          placeholder="请输入 2-16 字昵称（不可用手机号）"
          @input="clearError"
          @blur="handleNicknameBlur"
        />
        <!-- #endif -->
        <!-- #ifndef MP-WEIXIN -->
        <input
          v-model="nickname"
          class="profile-sheet__input"
          maxlength="16"
          :disabled="busy"
          placeholder="请输入 2-16 字昵称（不可用手机号）"
          @input="clearError"
        />
        <!-- #endif -->
        <text class="profile-sheet__hint">支持中文、英文或数字，长度 2-16 个字符，将通过微信内容安全审核。</text>
      </view>

      <text v-if="errorMessage" class="profile-sheet__error">{{ errorMessage }}</text>
    </view>

    <template #footer>
      <view class="profile-sheet__footer">
        <button
          class="profile-sheet__submit"
          :disabled="busy"
          hover-class="profile-sheet__pressed"
          @tap="handleSubmit"
        >
          {{ busy ? '保存中…' : '保存并继续' }}
        </button>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed, onScopeDispose, ref, watch } from 'vue'
import FiBottomSheet from './FiBottomSheet.vue'
import { readAvatarAsDataUrl } from '../utils/avatarDataUrl'
import { hasValidAvatarUrl, validateNicknameInput } from '../utils/profileGate'

const props = withDefaults(defineProps<{
  visible: boolean
  submitting: boolean
  initialNickname: string
  initialAvatarUrl: string | null
  subtitle?: string
}>(), {
  subtitle: '发表评论前请先上传头像并设置昵称，向其他球迷展示你的专属身份。',
})

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'submit', payload: { display_name: string; avatar_data_url: string | null }): void
}>()

const nickname = ref('')
const selectedAvatarPath = ref('')
const errorMessage = ref('')
const readingAvatar = ref(false)
const keyboardHeight = ref(0)

const busy = computed(() => props.submitting || readingAvatar.value)
const previewAvatar = computed(() =>
  selectedAvatarPath.value
  || (hasValidAvatarUrl(props.initialAvatarUrl) ? (props.initialAvatarUrl ?? '') : ''),
)

watch(
  () => props.visible,
  (open) => {
    if (open) {
      nickname.value = props.initialNickname
      selectedAvatarPath.value = ''
      errorMessage.value = ''
      return
    }
    keyboardHeight.value = 0
  },
  { immediate: true },
)

const keyboardListener = (res: { height?: number }) => {
  keyboardHeight.value = props.visible && typeof res?.height === 'number' && res.height > 0 ? res.height : 0
}
if (typeof uni !== 'undefined' && typeof uni.onKeyboardHeightChange === 'function') {
  uni.onKeyboardHeightChange(keyboardListener)
}
onScopeDispose(() => {
  if (typeof uni !== 'undefined' && typeof uni.offKeyboardHeightChange === 'function') {
    uni.offKeyboardHeightChange(keyboardListener)
  }
})

function clearError(): void {
  errorMessage.value = ''
}

function handleVisibleChange(nextVisible: boolean): void {
  emit('update:visible', nextVisible)
}

function handleClose(): void {
  keyboardHeight.value = 0
}

function handleChooseMiniAvatar(event: unknown): void {
  const detail = (event as { detail?: { avatarUrl?: unknown } } | null)?.detail
  const avatarUrl = typeof detail?.avatarUrl === 'string' ? detail.avatarUrl.trim() : ''
  if (!avatarUrl) {
    return
  }
  selectedAvatarPath.value = avatarUrl
  errorMessage.value = ''
}

function handleChooseH5Avatar(): void {
  if (busy.value) {
    return
  }
  uni.chooseImage({
    count: 1,
    sizeType: ['compressed'],
    sourceType: ['album', 'camera'],
    success: (res) => {
      const first = Array.isArray(res.tempFilePaths) ? res.tempFilePaths[0] : ''
      if (typeof first === 'string' && first.trim()) {
        selectedAvatarPath.value = first.trim()
        errorMessage.value = ''
      }
    },
  })
}

function handleNicknameBlur(event: unknown): void {
  const detail = (event as { detail?: { value?: unknown } } | null)?.detail
  if (typeof detail?.value === 'string') {
    nickname.value = detail.value
  }
}

async function handleSubmit(): Promise<void> {
  if (busy.value) {
    return
  }

  const hasNewAvatar = Boolean(selectedAvatarPath.value)
  if (!hasNewAvatar && !hasValidAvatarUrl(props.initialAvatarUrl)) {
    errorMessage.value = '请先上传头像'
    uni.showToast({ title: errorMessage.value, icon: 'none' })
    return
  }

  const validNick = validateNicknameInput(nickname.value)
  if (!validNick) {
    errorMessage.value = '请输入 2-16 字昵称，不能使用手机号或默认名'
    uni.showToast({ title: errorMessage.value, icon: 'none' })
    return
  }

  let avatarDataUrl: string | null = null
  if (hasNewAvatar) {
    readingAvatar.value = true
    try {
      avatarDataUrl = await readAvatarAsDataUrl(selectedAvatarPath.value)
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : '头像读取失败，请重新选择'
      uni.showToast({ title: errorMessage.value, icon: 'none' })
      return
    } finally {
      readingAvatar.value = false
    }
  }

  errorMessage.value = ''
  emit('submit', {
    display_name: validNick,
    avatar_data_url: avatarDataUrl,
  })
}
</script>

<style scoped>
.profile-sheet {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-24);
  padding-bottom: var(--fi-space-12);
}

.profile-sheet__summary {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.profile-sheet__avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-8) 0;
}

.profile-sheet__avatar-btn {
  width: 152rpx;
  height: 152rpx;
  padding: 0;
  margin: 0;
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-card);
  background: var(--fi-color-page-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: var(--fi-shadow-soft);
}

.profile-sheet__avatar-btn::after,
.profile-sheet__submit::after {
  border: 0;
}

.profile-sheet__avatar-img {
  width: 100%;
  height: 100%;
}

.profile-sheet__avatar-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-6);
  color: var(--fi-color-text-muted);
}

.profile-sheet__avatar-plus {
  font-size: var(--fi-font-36);
  line-height: 1;
  font-weight: var(--fi-weight-bold);
}

.profile-sheet__avatar-text {
  font-size: var(--fi-font-22);
  line-height: 1;
}

.profile-sheet__avatar-hint {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.profile-sheet__field {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-10);
}

.profile-sheet__label {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.profile-sheet__input {
  width: 100%;
  height: 88rpx;
  padding: 0 var(--fi-space-24);
  box-sizing: border-box;
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-28);
}

.profile-sheet__hint {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.profile-sheet__error {
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.profile-sheet__footer {
  width: 100%;
}

.profile-sheet__submit {
  width: 100%;
  height: 88rpx;
  margin: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: 88rpx;
  text-align: center;
}

.profile-sheet__submit[disabled] {
  opacity: 0.55;
}

.profile-sheet__pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
