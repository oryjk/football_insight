<template>
  <view v-if="visible" class="ai-chat-mask" @tap="handleMaskTap">
    <view class="ai-chat-sheet" @tap.stop>
      <view class="ai-chat-sheet__header">
        <view class="ai-chat-sheet__identity">
          <view class="ai-chat-sheet__avatar-shell">
            <view class="ai-chat-sheet__avatar-glow"></view>
            <image :src="activeAvatar" mode="aspectFill" class="ai-chat-sheet__avatar" />
          </view>
          <view class="ai-chat-sheet__identity-copy">
            <text class="ai-chat-sheet__eyebrow">AI Football Mate</text>
            <view class="ai-chat-sheet__title-row">
              <text class="ai-chat-sheet__title">问问小罗，嗡嗡嗡～</text>
              <button
                v-if="aiCapabilityNotice"
                class="ai-chat-sheet__info-badge"
                @click="handleShowAiCapabilityNotice"
              >
                ?
              </button>
            </view>
            <text class="ai-chat-sheet__caption">球队、球员、中超、欧冠、五大联赛、世界杯都可以聊</text>
          </view>
        </view>
        <button class="ai-chat-sheet__close" @click="handleClose">关闭</button>
      </view>

      <view class="ai-chat-sheet__mode-switch">
        <button
          class="ai-chat-sheet__mode-pill"
          :class="{ 'ai-chat-sheet__mode-pill--active': interactionMode === 'text' }"
          @click="handleSwitchInteractionMode('text')"
        >
          文字聊天
        </button>
        <button
          class="ai-chat-sheet__mode-pill"
          :class="{ 'ai-chat-sheet__mode-pill--active': interactionMode === 'image' }"
          @click="handleSwitchInteractionMode('image')"
        >
          生成图片
        </button>
      </view>

      <view v-if="messages.length" class="ai-chat-sheet__meta">
        <button v-if="messages.length" class="ai-chat-sheet__clear" @click="handleClearHistory">清空历史</button>
      </view>

      <scroll-view
        scroll-y
        class="ai-chat-sheet__messages"
        :scroll-into-view="scrollIntoView"
        scroll-with-animation
      >
        <view v-if="!messages.length" class="ai-chat-sheet__empty">
          <text class="ai-chat-sheet__empty-title">{{ interactionMeta.emptyTitle }}</text>
          <text class="ai-chat-sheet__empty-copy">{{ interactionMeta.emptyCopy }}</text>

          <view v-if="interactionMode === 'text'" class="ai-chat-sheet__suggestions">
            <view
              v-for="suggestion in suggestionPrompts"
              :key="suggestion"
              class="ai-chat-sheet__suggestion"
              @tap="handleSuggestionTap(suggestion)"
            >
              <text>{{ suggestion }}</text>
            </view>
          </view>
        </view>

        <view v-else class="ai-chat-sheet__message-list">
          <view
            v-for="message in messages"
            :key="message.id"
            class="ai-chat-message"
            :class="message.role === 'user' ? 'ai-chat-message--user' : 'ai-chat-message--assistant'"
          >
            <view v-if="message.role === 'assistant'" class="ai-chat-message__avatar-shell">
              <image :src="assistantAvatar" mode="aspectFill" class="ai-chat-message__avatar" />
            </view>
            <view
              class="ai-chat-message__bubble"
              :class="{ 'ai-chat-message__bubble--thinking': isThinkingMessage(message) }"
            >
              <view v-if="isThinkingMessage(message)" class="ai-chat-message__thinking">
                <text class="ai-chat-message__thinking-label">思考中</text>
                <view class="ai-chat-message__thinking-dots">
                  <text v-for="dot in 3" :key="dot" class="ai-chat-message__thinking-dot">•</text>
                </view>
              </view>
              <view v-else class="ai-chat-message__text-wrap">
                <image
                  v-if="message.imageUrl"
                  :src="message.imageUrl"
                  mode="widthFix"
                  class="ai-chat-message__image"
                />
                <text class="ai-chat-message__text">{{ message.content }}</text>
                <text v-if="showsTypingCursor(message)" class="ai-chat-message__cursor">|</text>
              </view>
            </view>
          </view>
        </view>

        <view id="ai-chat-bottom-anchor" class="ai-chat-sheet__anchor"></view>
      </scroll-view>

      <view class="ai-chat-sheet__composer">
        <textarea
          v-model="draft"
          class="ai-chat-sheet__textarea"
          maxlength="4000"
          auto-height
          :disabled="sending"
          :placeholder="interactionMeta.placeholder"
          confirm-type="send"
          @confirm="handleSubmit()"
        />
        <button class="ai-chat-sheet__send" :disabled="sending" @click="handleSubmit()">
          {{ sending ? loadingLabel : interactionMeta.submitLabel }}
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { CurrentUser } from '../types/auth'
import type { AiChatHistoryMessagePayload, AiChatMessage } from '../types/ai'
import type { AiChatMode } from '../types/system'
import {
  getAiChatCapabilityNotice,
  getAiInteractionMeta,
  type AiInteractionMode,
} from '../config/aiChat'
import { generateAiImage, streamAiChat, type AiChatStreamHandle } from '../api/ai'
import { clearAiChatHistory, getAiChatHistory, setAiChatHistory } from '../utils/aiChatStorage'
import aiRonaldinhoAvatar from '../static/ai/ronaldinho-avatar.png'
import generateImageAvatar from '../static/ai/generate-image-avatar.png'

const props = defineProps<{
  visible: boolean
  currentUser: CurrentUser | null
  aiChatMode?: AiChatMode
}>()

const emit = defineEmits<{
  close: []
}>()

const draft = ref('')
const sending = ref(false)
const messages = ref<AiChatMessage[]>([])
const scrollIntoView = ref('')
const activeStream = ref<AiChatStreamHandle | null>(null)
const hasShownOpenNotice = ref(false)
const interactionMode = ref<AiInteractionMode>('text')

const suggestionPrompts = [
  '中超榜首最近为什么这么胶着？',
  '欧冠淘汰赛里最值得关注的球队是谁？',
  '罗纳尔迪尼奥巅峰期最大的特点是什么？',
]

const canChat = computed(() => !!props.currentUser?.id)
const aiCapabilityNotice = computed(() => getAiChatCapabilityNotice(props.aiChatMode))
const interactionMeta = computed(() => getAiInteractionMeta(interactionMode.value))
const activeAvatar = computed(() =>
  interactionMode.value === 'image' ? generateImageAvatar : aiRonaldinhoAvatar,
)
const assistantAvatar = computed(() =>
  interactionMode.value === 'image' ? generateImageAvatar : aiRonaldinhoAvatar,
)
const loadingLabel = computed(() =>
  interactionMode.value === 'image' ? '生成中...' : '回答中...',
)
const activeAssistantMessageId = computed(() => {
  const lastMessage = messages.value[messages.value.length - 1]
  if (!lastMessage || lastMessage.role !== 'assistant') {
    return ''
  }

  return lastMessage.id
})

function createMessage(role: AiChatMessage['role'], content: string): AiChatMessage {
  return {
    id: `ai-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    role,
    content,
    createdAt: new Date().toISOString(),
  }
}

function syncScrollToBottom() {
  nextTick(() => {
    scrollIntoView.value = ''
    scrollIntoView.value = 'ai-chat-bottom-anchor'
  })
}

function persistHistory() {
  if (!props.currentUser?.id) {
    return
  }

  setAiChatHistory(props.currentUser.id, messages.value)
}

function loadHistory() {
  if (!props.currentUser?.id) {
    messages.value = []
    return
  }

  messages.value = getAiChatHistory(props.currentUser.id)
  syncScrollToBottom()
}

function stopStreaming() {
  activeStream.value?.abort()
  activeStream.value = null
  sending.value = false
}

function handleMaskTap() {
  handleClose()
}

function handleClose() {
  stopStreaming()
  interactionMode.value = 'text'
  emit('close')
}

function handleClearHistory() {
  if (!props.currentUser?.id) {
    return
  }

  clearAiChatHistory(props.currentUser.id)
  messages.value = []
}

function handleShowAiCapabilityNotice() {
  if (!aiCapabilityNotice.value) {
    return
  }

  uni.showModal({
    title: aiCapabilityNotice.value.title,
    content: aiCapabilityNotice.value.content,
    showCancel: false,
    confirmText: '知道了',
  })
}

function normalizeHistoryPayload(items: AiChatMessage[]): AiChatHistoryMessagePayload[] {
  return items
    .filter((item) => item.content.trim().length > 0)
    .map((item) => ({
      role: item.role,
      content: item.content,
    }))
}

function isActiveAssistantMessage(message: AiChatMessage): boolean {
  return sending.value && message.role === 'assistant' && message.id === activeAssistantMessageId.value
}

function isThinkingMessage(message: AiChatMessage): boolean {
  return isActiveAssistantMessage(message) && !message.content.trim()
}

function showsTypingCursor(message: AiChatMessage): boolean {
  return isActiveAssistantMessage(message) && !!message.content.trim()
}

function handleSuggestionTap(suggestion: string) {
  if (sending.value) {
    return
  }

  draft.value = suggestion
  handleSubmit(suggestion)
}

function handleSwitchInteractionMode(mode: AiInteractionMode) {
  if (sending.value || interactionMode.value === mode) {
    return
  }

  interactionMode.value = mode
}

async function handleSubmit(overrideMessage?: string) {
  if (!canChat.value || sending.value) {
    return
  }

  const content = (overrideMessage ?? draft.value).trim()
  if (!content) {
    uni.showToast({
      title: '请输入内容',
      icon: 'none',
    })
    return
  }

  const userMessage = createMessage('user', content)
  const assistantMessage = createMessage('assistant', '')

  messages.value = [...messages.value, userMessage, assistantMessage]
  draft.value = ''
  sending.value = true
  syncScrollToBottom()
  persistHistory()

  if (interactionMode.value === 'image') {
    try {
      const result = await generateAiImage(content)
      messages.value = messages.value.map((message) =>
        message.id === assistantMessage.id
          ? {
              ...message,
              kind: 'image',
              imageUrl: result.imageUrl,
              content: result.revisedPrompt ? `已根据你的描述生成图片：${result.revisedPrompt}` : '图片生成完成',
            }
          : message,
      )
      sending.value = false
      persistHistory()
      syncScrollToBottom()
    } catch (error) {
      const assistantIndex = messages.value.findIndex((item) => item.id === assistantMessage.id)
      if (assistantIndex >= 0 && !messages.value[assistantIndex].content.trim()) {
        messages.value.splice(assistantIndex, 1)
      }

      sending.value = false
      persistHistory()
      uni.showToast({
        title: error instanceof Error ? error.message : '图片生成失败',
        icon: 'none',
        duration: 2400,
      })
    }
    return
  }

  activeStream.value = streamAiChat(
    {
      message: content,
      history: normalizeHistoryPayload(messages.value.filter((item) => item.id !== assistantMessage.id)),
    },
    {
      onStarted: () => {},
      onDelta: ({ content: delta }) => {
        messages.value = messages.value.map((message) =>
          message.id === assistantMessage.id
            ? { ...message, content: `${message.content}${delta}` }
            : message,
        )
        syncScrollToBottom()
      },
      onDone: ({ reply }) => {
        messages.value = messages.value.map((message) =>
          message.id === assistantMessage.id
            ? { ...message, content: reply || message.content }
            : message,
        )
        sending.value = false
        activeStream.value = null
        persistHistory()
        syncScrollToBottom()
      },
      onError: (message) => {
        const assistantIndex = messages.value.findIndex((item) => item.id === assistantMessage.id)
        if (assistantIndex >= 0 && !messages.value[assistantIndex].content.trim()) {
          messages.value.splice(assistantIndex, 1)
        }

        sending.value = false
        activeStream.value = null
        persistHistory()

        uni.showToast({
          title: message,
          icon: 'none',
          duration: 2400,
        })
      },
    },
    {
      mode: props.aiChatMode,
    },
  )
}

watch(
  () => [props.visible, props.currentUser?.id] as const,
  ([visible]) => {
    if (!visible) {
      hasShownOpenNotice.value = false
      interactionMode.value = 'text'
      return
    }

    if (aiCapabilityNotice.value && !hasShownOpenNotice.value) {
      hasShownOpenNotice.value = true
      handleShowAiCapabilityNotice()
    }

    loadHistory()
  },
  { immediate: true },
)
</script>

<style scoped lang="css">
.ai-chat-mask {
  position: fixed;
  inset: 0;
  z-index: 40;
  background: rgba(12, 16, 24, 0.32);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 36rpx 24rpx;
  box-sizing: border-box;
}

.ai-chat-sheet {
  width: 100%;
  height: 78vh;
  max-height: 1080rpx;
  background:
    radial-gradient(circle at top right, rgba(255, 220, 109, 0.18), transparent 28%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.99), rgba(246, 248, 252, 0.98));
  border-radius: 36rpx;
  border: 2rpx solid rgba(233, 235, 241, 0.96);
  box-shadow: 0 36rpx 90rpx rgba(16, 19, 26, 0.2);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.ai-chat-sheet__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 24rpx;
  padding: 30rpx 30rpx 16rpx;
}

.ai-chat-sheet__mode-switch {
  display: flex;
  gap: 14rpx;
  padding: 0 30rpx 18rpx;
}

.ai-chat-sheet__mode-pill {
  flex: 1 1 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 66rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(228, 229, 236, 0.96);
  background: rgba(255, 255, 255, 0.86);
  color: #666b77;
  font-size: 24rpx;
  font-weight: 700;
}

.ai-chat-sheet__mode-pill--active {
  background: linear-gradient(135deg, rgba(34, 38, 49, 0.98), rgba(61, 67, 84, 0.98));
  border-color: rgba(34, 38, 49, 0.98);
  color: #fff7e3;
}

.ai-chat-sheet__identity {
  display: flex;
  align-items: center;
  gap: 20rpx;
  min-width: 0;
}

.ai-chat-sheet__avatar-shell,
.ai-chat-message__avatar-shell {
  position: relative;
  flex: 0 0 auto;
}

.ai-chat-sheet__avatar-shell {
  width: 108rpx;
  height: 108rpx;
  animation: ai-avatar-float 2.8s ease-in-out infinite;
}

.ai-chat-sheet__avatar-glow {
  position: absolute;
  inset: -10rpx;
  border-radius: 999rpx;
  background: radial-gradient(circle, rgba(255, 214, 102, 0.58), rgba(255, 214, 102, 0));
  animation: ai-avatar-glow 2.2s ease-in-out infinite;
}

.ai-chat-sheet__avatar,
.ai-chat-message__avatar {
  position: relative;
  width: 100%;
  height: 100%;
  border-radius: 999rpx;
  border: 4rpx solid rgba(255, 255, 255, 0.92);
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 12rpx 24rpx rgba(23, 25, 35, 0.12);
}

.ai-chat-sheet__identity-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6rpx;
}

.ai-chat-sheet__title-row {
  display: flex;
  align-items: center;
  gap: 12rpx;
}

.ai-chat-sheet__eyebrow {
  color: #7f7157;
  font-size: 22rpx;
  font-weight: 700;
  letter-spacing: 2rpx;
}

.ai-chat-sheet__title {
  color: #151515;
  font-size: 44rpx;
  font-weight: 800;
}

.ai-chat-sheet__info-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40rpx;
  height: 40rpx;
  border-radius: 999rpx;
  background: rgba(255, 214, 102, 0.2);
  border: 2rpx solid rgba(226, 182, 64, 0.55);
  color: #8b6400;
  font-size: 24rpx;
  font-weight: 800;
  line-height: 1;
  padding: 0;
  flex: 0 0 auto;
}

.ai-chat-sheet__caption {
  color: #70747f;
  font-size: 24rpx;
  line-height: 1.45;
}

.ai-chat-sheet__close,
.ai-chat-sheet__clear {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  padding: 0 24rpx;
  height: 70rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(228, 229, 236, 0.96);
  background: rgba(255, 255, 255, 0.9);
  color: #666b77;
  font-size: 24rpx;
  line-height: 1;
}

.ai-chat-sheet__meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 16rpx;
  padding: 0 30rpx 18rpx;
}

.ai-chat-sheet__messages {
  flex: 1;
  min-height: 0;
  padding: 0 30rpx 12rpx;
  box-sizing: border-box;
}

.ai-chat-sheet__empty {
  min-height: 100%;
  padding: 26rpx 6rpx 20rpx;
  display: flex;
  flex-direction: column;
  gap: 20rpx;
}

.ai-chat-sheet__empty-title {
  color: #161616;
  font-size: 36rpx;
  font-weight: 800;
}

.ai-chat-sheet__empty-copy {
  color: #70747f;
  font-size: 28rpx;
  line-height: 1.7;
}

.ai-chat-sheet__suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 14rpx;
}

.ai-chat-sheet__suggestion {
  padding: 18rpx 24rpx;
  border-radius: 26rpx;
  border: 2rpx solid rgba(231, 233, 240, 0.96);
  background: rgba(255, 255, 255, 0.94);
  color: #1b1d22;
  font-size: 26rpx;
  line-height: 1.4;
}

.ai-chat-sheet__message-list {
  display: flex;
  flex-direction: column;
  gap: 18rpx;
  padding-top: 8rpx;
}

.ai-chat-message {
  display: flex;
  gap: 14rpx;
}

.ai-chat-message--user {
  justify-content: flex-end;
}

.ai-chat-message--assistant {
  justify-content: flex-start;
}

.ai-chat-message__avatar-shell {
  width: 56rpx;
  height: 56rpx;
  margin-top: 4rpx;
}

.ai-chat-message__bubble {
  max-width: 78%;
  padding: 18rpx 22rpx;
  border-radius: 26rpx;
  box-shadow: 0 12rpx 26rpx rgba(18, 20, 28, 0.06);
}

.ai-chat-message__bubble--thinking {
  min-width: 180rpx;
}

.ai-chat-message--assistant .ai-chat-message__bubble {
  background: rgba(255, 255, 255, 0.95);
  border: 2rpx solid rgba(233, 235, 241, 0.98);
}

.ai-chat-message--user .ai-chat-message__bubble {
  background: linear-gradient(135deg, rgba(32, 36, 47, 0.98), rgba(52, 58, 75, 0.98));
}

.ai-chat-message__text {
  font-size: 28rpx;
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-word;
}

.ai-chat-message__text-wrap,
.ai-chat-message__thinking {
  display: inline-flex;
  align-items: center;
}

.ai-chat-message__text-wrap {
  flex-wrap: wrap;
  flex-direction: column;
  align-items: flex-start;
  gap: 12rpx;
}

.ai-chat-message__image {
  width: 100%;
  border-radius: 22rpx;
  background: rgba(245, 246, 250, 0.9);
}

.ai-chat-message--assistant .ai-chat-message__text {
  color: #1c1f26;
}

.ai-chat-message--user .ai-chat-message__text {
  color: #ffffff;
}

.ai-chat-message__thinking {
  gap: 14rpx;
}

.ai-chat-message__thinking-label {
  color: #5b6270;
  font-size: 26rpx;
  font-weight: 600;
}

.ai-chat-message__thinking-dots {
  display: inline-flex;
  align-items: center;
  gap: 6rpx;
}

.ai-chat-message__thinking-dot {
  color: #d4a53a;
  font-size: 28rpx;
  line-height: 1;
  animation: ai-thinking-bounce 1s ease-in-out infinite;
}

.ai-chat-message__thinking-dot:nth-child(2) {
  animation-delay: 0.12s;
}

.ai-chat-message__thinking-dot:nth-child(3) {
  animation-delay: 0.24s;
}

.ai-chat-message__cursor {
  display: inline-block;
  margin-left: 6rpx;
  color: #d4a53a;
  font-size: 30rpx;
  font-weight: 700;
  line-height: 1;
  animation: ai-cursor-blink 1s steps(2, start) infinite;
}

.ai-chat-sheet__anchor {
  height: 1rpx;
}

.ai-chat-sheet__composer {
  display: flex;
  align-items: flex-end;
  gap: 18rpx;
  padding: 18rpx 24rpx 28rpx;
  border-top: 2rpx solid rgba(232, 234, 240, 0.96);
  background: rgba(252, 252, 253, 0.96);
}

.ai-chat-sheet__textarea {
  flex: 1;
  min-height: 92rpx;
  max-height: 220rpx;
  padding: 22rpx 24rpx;
  border-radius: 26rpx;
  background: rgba(244, 246, 250, 0.96);
  color: #15181f;
  font-size: 28rpx;
  line-height: 1.6;
  box-sizing: border-box;
}

.ai-chat-sheet__send {
  flex: 0 0 auto;
  width: 144rpx;
  height: 92rpx;
  border-radius: 26rpx;
  background: linear-gradient(135deg, #1f2430, #394053);
  color: #ffffff;
  font-size: 28rpx;
  font-weight: 700;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.ai-chat-sheet__send[disabled] {
  opacity: 0.68;
}

@keyframes ai-avatar-float {
  0%, 100% {
    transform: translateY(0) rotate(-2deg);
  }
  50% {
    transform: translateY(-6rpx) rotate(2deg);
  }
}

@keyframes ai-avatar-glow {
  0%, 100% {
    transform: scale(0.92);
    opacity: 0.72;
  }
  50% {
    transform: scale(1.08);
    opacity: 1;
  }
}

@keyframes ai-thinking-bounce {
  0%, 80%, 100% {
    transform: translateY(0);
    opacity: 0.42;
  }

  40% {
    transform: translateY(-5rpx);
    opacity: 1;
  }
}

@keyframes ai-cursor-blink {
  0%, 49% {
    opacity: 1;
  }

  50%, 100% {
    opacity: 0;
  }
}
</style>
