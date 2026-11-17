<template>
  <view
    v-if="visible && isFocused"
    class="composer-backdrop"
    @touchmove.stop.prevent
    @tap="dismissKeyboard"
  />
  <view
    v-if="visible"
    class="composer"
    :class="{ 'composer--lifted': keyboardHeight > 0 }"
    :style="{ bottom: `${keyboardHeight}px` }"
    @touchmove.stop
    @tap.stop
  >
    <button
      v-if="!loggedIn"
      class="composer__gate"
      hover-class="composer__pressed"
      @tap="emit('login')"
    >
      登录后点赞或发表评论 ›
    </button>

    <button
      v-else-if="!profileComplete"
      class="composer__gate composer__gate--warn"
      hover-class="composer__pressed"
      @tap="emit('setup-profile')"
    >
      评论前请先上传头像并设置昵称 · 去完善 ›
    </button>

    <template v-else>
      <view v-if="replyTarget" class="composer__reply-bar">
        <text class="composer__reply-label">正在回复 @{{ replyTarget.author.displayName }}</text>
        <button
          class="composer__reply-cancel"
          hover-class="composer__pressed"
          @tap="emit('cancel-reply')"
        >
          取消
        </button>
      </view>

      <view class="composer__editor" :class="{ 'composer__editor--expanded': isExpanded }">
        <textarea
          :value="draft"
          :focus="programmaticFocus"
          fixed
          :adjust-position="false"
          :show-confirm-bar="false"
          :disable-default-padding="true"
          :cursor-spacing="0"
          :maxlength="200"
          :disabled="submitting"
          class="composer__textarea"
          :class="{ 'composer__textarea--expanded': isExpanded }"
          :placeholder="inputPlaceholder"
          @input="handleInput"
          @focus="handleFocus"
          @blur="handleBlur"
          @keyboardheightchange="handleKeyboardHeightChange"
        />
        <view class="composer__toolbar" :class="{ 'composer__toolbar--expanded': isExpanded }">
          <text v-if="isExpanded" class="composer__counter">{{ draftCharCount }} / 200</text>
          <button
            class="composer__send"
            :disabled="submitting || !draft.trim()"
            hover-class="composer__pressed"
            @tap="handleSendTap"
          >
            {{ submitting ? '发送中' : '发送' }}
          </button>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, onScopeDispose, ref, watch } from 'vue'
import { useSheetKeyboard } from '../../../composables/useSheetKeyboard'
import type { TelegraphComment } from '../../../types/dailyTopic'

const props = defineProps<{
  visible: boolean
  loggedIn: boolean
  profileComplete: boolean
  draft: string
  replyTarget: TelegraphComment | null
  submitting: boolean
  focusTrigger: number
}>()

const emit = defineEmits<{
  (e: 'update:draft', value: string): void
  (e: 'login'): void
  (e: 'setup-profile'): void
  (e: 'cancel-reply'): void
  (e: 'submit'): void
  (e: 'focus-change', focused: boolean): void
}>()

const {
  keyboardHeight,
  onInputFocus,
  onInputBlur,
  onKeyboardHeightChange,
  resetKeyboard,
} = useSheetKeyboard()

const isFocused = ref(false)
const programmaticFocus = ref(false)
let collapseTimer: ReturnType<typeof setTimeout> | null = null

const draftCharCount = computed(() => Array.from(props.draft).length)
const isExpanded = computed(() =>
  isFocused.value
  || props.submitting
  || draftCharCount.value > 0
  || Boolean(props.replyTarget),
)
const inputPlaceholder = computed(() =>
  props.replyTarget ? `回复 @${props.replyTarget.author.displayName}…` : '发布你的看法（最多 200 字）',
)

function clearCollapseTimer(): void {
  if (collapseTimer) {
    clearTimeout(collapseTimer)
    collapseTimer = null
  }
}

watch(isFocused, (focused) => {
  emit('focus-change', focused)
})

watch(
  () => props.visible,
  (open) => {
    if (!open) {
      clearCollapseTimer()
      programmaticFocus.value = false
      isFocused.value = false
      resetKeyboard()
    }
  },
)

watch(
  () => props.focusTrigger,
  async (trigger) => {
    if (!trigger || !props.visible) {
      return
    }
    clearCollapseTimer()
    isFocused.value = true
    programmaticFocus.value = false
    await nextTick()
    programmaticFocus.value = true
  },
)

const globalKeyboardListener = (res: { height?: number }) => {
  if (!props.visible || !isFocused.value) {
    return
  }
  const height = typeof res?.height === 'number' && res.height > 0 ? res.height : 0
  onKeyboardHeightChange({ detail: { height } })
  if (height === 0 && !props.draft.trim()) {
    isFocused.value = false
  }
}
if (typeof uni !== 'undefined' && typeof uni.onKeyboardHeightChange === 'function') {
  uni.onKeyboardHeightChange(globalKeyboardListener)
}

onScopeDispose(() => {
  clearCollapseTimer()
  if (typeof uni !== 'undefined' && typeof uni.offKeyboardHeightChange === 'function') {
    uni.offKeyboardHeightChange(globalKeyboardListener)
  }
})

function handleInput(event: unknown): void {
  const detail = (event as { detail?: { value?: unknown } } | null)?.detail
  if (typeof detail?.value === 'string') {
    emit('update:draft', detail.value)
    return
  }
  const target = (event as { target?: { value?: unknown } } | null)?.target
  if (typeof target?.value === 'string') {
    emit('update:draft', target.value)
  }
}

function handleFocus(event: unknown): void {
  clearCollapseTimer()
  isFocused.value = true
  onInputFocus(event)
}

function handleKeyboardHeightChange(event: unknown): void {
  const hadHeight = keyboardHeight.value > 0
  onKeyboardHeightChange(event)
  if (hadHeight && keyboardHeight.value === 0 && !props.draft.trim()) {
    isFocused.value = false
  }
}

function handleBlur(): void {
  programmaticFocus.value = false
  onInputBlur()
  clearCollapseTimer()
  // 延迟收起高度，避免点「发送」或「取消」时按钮位置先于 @tap 跳变。
  collapseTimer = setTimeout(() => {
    collapseTimer = null
    if (!programmaticFocus.value) {
      isFocused.value = false
    }
  }, 90)
}

function dismissKeyboard(): void {
  clearCollapseTimer()
  programmaticFocus.value = false
  isFocused.value = false
  resetKeyboard()
  if (typeof uni !== 'undefined' && typeof uni.hideKeyboard === 'function') {
    uni.hideKeyboard()
  }
}

function handleSendTap(): void {
  clearCollapseTimer()
  emit('submit')
}
</script>

<style scoped>
.composer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 29;
  background: var(--fi-color-overlay);
  opacity: 0.35;
}

.composer {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 30;
  padding: var(--fi-space-14) var(--fi-space-24) calc(var(--fi-space-18) + env(safe-area-inset-bottom));
  background: var(--fi-color-surface);
  border-top: var(--fi-border-divider);
  box-shadow: var(--fi-shadow-floating);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
  transition: bottom var(--fi-motion-duration-fast) var(--fi-motion-ease-out);
}

.composer--lifted {
  padding-bottom: var(--fi-space-14);
}

.composer__gate,
.composer__reply-cancel,
.composer__send {
  margin: 0;
}

.composer__gate::after,
.composer__reply-cancel::after,
.composer__send::after {
  border: 0;
}

.composer__gate {
  width: 100%;
  height: 78rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  border: var(--fi-border-card);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: 78rpx;
  text-align: center;
}

.composer__gate--warn {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  border-color: transparent;
}

.composer__reply-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page-soft);
}

.composer__reply-label {
  min-width: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.composer__reply-cancel {
  flex-shrink: 0;
  padding: 0 var(--fi-space-8);
  background: transparent;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: 1.4;
}

.composer__editor {
  display: flex;
  align-items: flex-end;
  gap: var(--fi-space-12);
}

.composer__editor--expanded {
  flex-direction: column;
  align-items: stretch;
  gap: var(--fi-space-12);
}

.composer__textarea {
  flex: 1;
  min-width: 0;
  width: auto;
  height: 76rpx;
  min-height: 76rpx;
  padding: var(--fi-space-16) var(--fi-space-24);
  box-sizing: border-box;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-card);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-snug);
}

.composer__textarea--expanded {
  flex: none;
  width: 100%;
  height: 184rpx;
  min-height: 184rpx;
  padding: var(--fi-space-18) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-normal);
}

.composer__toolbar {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.composer__toolbar--expanded {
  width: 100%;
  justify-content: space-between;
}

.composer__counter {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  font-variant-numeric: tabular-nums;
}

.composer__send {
  flex-shrink: 0;
  height: 76rpx;
  padding: 0 var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-primitive-ink);
  color: var(--fi-primitive-white);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: 76rpx;
}

.composer__send[disabled] {
  opacity: 0.45;
}

.composer__pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
