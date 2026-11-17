<template>
  <FiBottomSheet
    :visible="visible"
    :title="sheetTitle"
    eyebrow="电报讨论"
    :height="sheetHeight"
    compact-footer
    :z-index="60"
    respect-window-bottom
    :keyboard-height="keyboardHeight"
    @update:visible="emit('update:visible', $event)"
    @close="keyboardHeight = 0"
  >
    <view class="comment-sheet__body" @tap.stop>
      <view v-if="entry" class="comment-sheet__entry">
        <text class="comment-sheet__entry-time">{{ entry.timeLabel }}</text>
        <text class="comment-sheet__entry-text">
          <text v-if="entry.title" class="comment-sheet__entry-title">【{{ entry.title }}】</text>{{ entry.text }}
        </text>
      </view>

      <view v-if="loading && !comments.length" class="comment-sheet__state">
        <text>正在加载评论…</text>
      </view>

      <view v-else-if="errorMessage && !comments.length" class="comment-sheet__state">
        <text class="comment-sheet__error">{{ errorMessage }}</text>
        <button class="comment-sheet__retry" @tap="emit('retry')">重新加载</button>
      </view>

      <view v-else-if="!comments.length" class="comment-sheet__state">
        <text>还没有球迷评论，来抢沙发吧</text>
      </view>

      <view v-else class="comment-sheet__list">
        <view v-for="comment in comments" :key="comment.id" class="comment-card">
          <view class="comment-card__avatar">
            <image
              v-if="comment.author.avatarUrl"
              :src="comment.author.avatarUrl"
              mode="aspectFill"
              class="comment-card__avatar-img"
            />
            <text v-else class="comment-card__avatar-fallback">{{ authorInitial(comment.author.displayName) }}</text>
          </view>

          <view class="comment-card__main">
            <view class="comment-card__header">
              <view class="comment-card__author-wrap">
                <text class="comment-card__author">{{ comment.author.displayName }}</text>
                <text v-if="comment.isMine" class="comment-card__mine-tag">我</text>
              </view>
              <text class="comment-card__time">{{ comment.createdLabel }}</text>
            </view>

            <text class="comment-card__content">
              <text v-if="comment.replyTo" class="comment-card__reply-prefix">
                回复 @{{ comment.replyTo.authorDisplayName }}：
              </text>{{ comment.content }}
            </text>

            <view class="comment-card__actions">
              <button
                class="comment-card__action"
                hover-class="comment-sheet__pressed"
                @tap="emit('reply', comment)"
              >
                回复
              </button>
              <button
                v-if="comment.isMine"
                class="comment-card__action comment-card__action--danger"
                :disabled="deletingId === comment.id"
                hover-class="comment-sheet__pressed"
                @tap="confirmDelete(comment)"
              >
                {{ deletingId === comment.id ? '删除中…' : '删除' }}
              </button>
            </view>
          </view>
        </view>

        <view class="comment-sheet__more">
          <button
            v-if="hasMore"
            class="comment-sheet__more-btn"
            :disabled="loadingMore"
            hover-class="comment-sheet__pressed"
            @tap="emit('load-more')"
          >
            {{ loadingMore ? '正在加载更多…' : '加载更多评论' }}
          </button>
          <text v-else class="comment-sheet__more-end">已显示全部评论</text>
        </view>
      </view>
    </view>

    <template #footer>
      <view class="comment-sheet__footer" @tap.stop>
        <button
          v-if="!loggedIn"
          class="comment-sheet__gate-btn"
          hover-class="comment-sheet__pressed"
          @tap="emit('login')"
        >
          登录后参与电报评论 ›
        </button>

        <button
          v-else-if="!profileComplete"
          class="comment-sheet__gate-btn comment-sheet__gate-btn--warn"
          hover-class="comment-sheet__pressed"
          @tap="emit('setup-profile')"
        >
          评论前请先上传头像并设置昵称 · 去完善 ›
        </button>

        <template v-else>
          <view v-if="replyTarget" class="comment-sheet__reply-bar">
            <text class="comment-sheet__reply-label">正在回复 @{{ replyTarget.author.displayName }}</text>
            <button
              class="comment-sheet__reply-cancel"
              hover-class="comment-sheet__pressed"
              @tap="emit('cancel-reply')"
            >
              取消
            </button>
          </view>

          <view class="comment-sheet__composer">
            <input
              :value="draft"
              class="comment-sheet__input"
              maxlength="280"
              :disabled="submitting"
              :adjust-position="false"
              confirm-type="send"
              :placeholder="inputPlaceholder"
              @input="handleDraftInput"
              @confirm="emit('submit')"
            />
            <button
              class="comment-sheet__send"
              :disabled="submitting || !draft.trim()"
              hover-class="comment-sheet__pressed"
              @tap="emit('submit')"
            >
              {{ submitting ? '发送中' : '发送' }}
            </button>
          </view>
        </template>
      </view>
    </template>
  </FiBottomSheet>
</template>

<script setup lang="ts">
import { computed, onScopeDispose, ref, watch } from 'vue'
import FiBottomSheet from '../../../components/FiBottomSheet.vue'
import type { DailyTopicEntry, TelegraphComment } from '../../../types/dailyTopic'

const props = defineProps<{
  visible: boolean
  entry: DailyTopicEntry | null
  comments: TelegraphComment[]
  loading: boolean
  loadingMore: boolean
  hasMore: boolean
  errorMessage: string
  draft: string
  replyTarget: TelegraphComment | null
  submitting: boolean
  deletingId: string
  loggedIn: boolean
  profileComplete: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:draft', value: string): void
  (e: 'retry'): void
  (e: 'load-more'): void
  (e: 'reply', comment: TelegraphComment): void
  (e: 'cancel-reply'): void
  (e: 'submit'): void
  (e: 'delete', comment: TelegraphComment): void
  (e: 'login'): void
  (e: 'setup-profile'): void
}>()

const keyboardHeight = ref(0)
const sheetTitle = computed(() =>
  props.entry && props.entry.commentsCount > 0 ? `评论 (${props.entry.commentsCount})` : '评论',
)
const sheetHeight = computed(() => (keyboardHeight.value > 0 ? '48vh' : '76vh'))
const inputPlaceholder = computed(() =>
  props.replyTarget ? `回复 @${props.replyTarget.author.displayName}…` : '说说你的看法（最多 280 字）',
)

watch(
  () => props.visible,
  (open) => {
    if (!open) {
      keyboardHeight.value = 0
    }
  },
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

function authorInitial(name: string): string {
  const trimmed = name.trim()
  return trimmed ? trimmed.slice(0, 1).toUpperCase() : '球'
}

function handleDraftInput(event: unknown): void {
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

function confirmDelete(comment: TelegraphComment): void {
  if (props.deletingId) {
    return
  }
  uni.showModal({
    title: '删除评论',
    content: '确认删除这条评论吗？删除后不可恢复。',
    confirmColor: '#dc2626',
    success: (res) => {
      if (res.confirm) {
        emit('delete', comment)
      }
    },
  })
}
</script>

<style scoped>
.comment-sheet__body {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-20);
  padding-bottom: var(--fi-space-24);
}

.comment-sheet__entry {
  padding: var(--fi-space-16) var(--fi-space-20);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-card);
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.comment-sheet__entry-time {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-20);
}

.comment-sheet__entry-text {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
}

.comment-sheet__entry-title {
  color: var(--fi-color-text-primary);
  font-weight: var(--fi-weight-bold);
}

.comment-sheet__state {
  padding: var(--fi-space-40) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-16);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-26);
  text-align: center;
}

.comment-sheet__error {
  color: var(--fi-color-primary);
}

.comment-sheet__retry,
.comment-sheet__more-btn,
.comment-sheet__gate-btn,
.comment-sheet__reply-cancel,
.comment-sheet__send,
.comment-card__action {
  margin: 0;
}

.comment-sheet__retry::after,
.comment-sheet__more-btn::after,
.comment-sheet__gate-btn::after,
.comment-sheet__reply-cancel::after,
.comment-sheet__send::after,
.comment-card__action::after {
  border: 0;
}

.comment-sheet__retry {
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-24);
  line-height: 1.4;
}

.comment-sheet__list {
  display: flex;
  flex-direction: column;
}

.comment-card {
  display: flex;
  align-items: flex-start;
  gap: var(--fi-space-16);
  padding: var(--fi-space-20) 0;
  border-bottom: var(--fi-border-divider);
}

.comment-card:last-of-type {
  border-bottom: 0;
}

.comment-card__avatar {
  width: 68rpx;
  height: 68rpx;
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-card);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.comment-card__avatar-img {
  width: 100%;
  height: 100%;
}

.comment-card__avatar-fallback {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.comment-card__main {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-8);
}

.comment-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.comment-card__author-wrap {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.comment-card__author {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.comment-card__mine-tag {
  flex-shrink: 0;
  padding: var(--fi-space-2) var(--fi-space-10);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: 1.3;
}

.comment-card__time {
  flex-shrink: 0;
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-20);
}

.comment-card__content {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-normal);
  word-break: break-word;
  white-space: pre-wrap;
}

.comment-card__reply-prefix {
  color: var(--fi-color-primary);
  font-weight: var(--fi-weight-bold);
}

.comment-card__actions {
  display: flex;
  align-items: center;
  gap: var(--fi-space-24);
  margin-top: var(--fi-space-4);
}

.comment-card__action {
  padding: var(--fi-space-4) 0;
  background: transparent;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: 1.4;
}

.comment-card__action--danger {
  color: var(--fi-color-primary);
}

.comment-sheet__more {
  padding: var(--fi-space-20) 0 var(--fi-space-8);
  display: flex;
  justify-content: center;
}

.comment-sheet__more-btn {
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: 1.4;
}

.comment-sheet__more-end {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.comment-sheet__footer {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-12);
}

.comment-sheet__gate-btn {
  width: 100%;
  height: 82rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  border: var(--fi-border-card);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: 82rpx;
  text-align: center;
}

.comment-sheet__gate-btn--warn {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  border-color: transparent;
}

.comment-sheet__reply-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
  padding: var(--fi-space-8) var(--fi-space-14);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-page-soft);
}

.comment-sheet__reply-label {
  min-width: 0;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-22);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.comment-sheet__reply-cancel {
  flex-shrink: 0;
  padding: 0 var(--fi-space-8);
  background: transparent;
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: 1.4;
}

.comment-sheet__composer {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.comment-sheet__input {
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

.comment-sheet__send {
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

.comment-sheet__send[disabled] {
  opacity: 0.45;
}

.comment-sheet__pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
