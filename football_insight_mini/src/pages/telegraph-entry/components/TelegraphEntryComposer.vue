<template>
  <view
    v-if="visible"
    class="composer"
    :class="{ 'composer--lifted': keyboardHeight > 0 }"
    :style="{ bottom: `${keyboardHeight}px` }"
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

      <view class="composer__row">
        <input
          :value="draft"
          :focus="inputFocused"
          class="composer__input"
          maxlength="200"
          :disabled="submitting"
          :adjust-position="false"
          confirm-type="send"
          :placeholder="inputPlaceholder"
          @input="handleInput"
          @blur="inputFocused = false"
          @confirm="emit('submit')"
        />
        <button
          class="composer__send"
          :disabled="submitting || !draft.trim()"
          hover-class="composer__pressed"
          @tap="emit('submit')"
        >
          {{ submitting ? '发送中' : '发送' }}
        </button>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { TelegraphComment } from '../../../types/dailyTopic'

const props = defineProps<{
  visible: boolean
  loggedIn: boolean
  profileComplete: boolean
  draft: string
  replyTarget: TelegraphComment | null
  submitting: boolean
  focusTrigger: number
  keyboardHeight: number
}>()

const emit = defineEmits<{
  (e: 'update:draft', value: string): void
  (e: 'login'): void
  (e: 'setup-profile'): void
  (e: 'cancel-reply'): void
  (e: 'submit'): void
}>()

const inputFocused = ref(false)
const inputPlaceholder = computed(() =>
  props.replyTarget ? `回复 @${props.replyTarget.author.displayName}…` : '发布你的看法（最多 200 字）',
)

watch(
  () => props.focusTrigger,
  async (trigger) => {
    if (!trigger) {
      return
    }
    inputFocused.value = false
    await nextTick()
    inputFocused.value = true
  },
)

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
</script>

<style scoped>
.composer {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 30;
  padding: var(--fi-space-14) var(--fi-space-24) calc(var(--fi-space-18) + env(safe-area-inset-bottom));
  background: var(--fi-color-surface);
  border-top: var(--fi-border-divider);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
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

.composer__row {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.composer__input {
  flex: 1;
  min-width: 0;
  height: 76rpx;
  padding: 0 var(--fi-space-24);
  box-sizing: border-box;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-card);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
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
