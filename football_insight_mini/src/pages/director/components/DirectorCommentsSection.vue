<template>
  <view class="interact-card">
    <view class="reaction-row">
      <view class="reaction-copy">
        <text class="reaction-title">这份方案你服不服？</text>
        <text class="reaction-sub">认同 {{ likesCount }} · 不服 {{ dislikesCount }}</text>
      </view>
      <FiReactionBar
        size="md"
        :count="likesCount"
        :dislike-count="dislikesCount"
        :liked="myReaction === 1"
        :disliked="myReaction === -1"
        :pending="reactionPending"
        @like="emit('react', 1)"
        @dislike="emit('react', -1)"
      />
    </view>

    <view class="comment-composer">
      <input
        v-model="draftText"
        class="comment-input"
        :maxlength="DIRECTOR_COMMENT_MAX"
        confirm-type="send"
        placeholder="一句话锐评这份方案（最多 50 字）"
        @confirm="handleSubmit"
      />
      <button
        class="comment-send"
        :class="{ 'comment-send--disabled': submitting || !draftText.trim() }"
        hover-class="comment-pressed"
        @tap="handleSubmit"
      >
        {{ submitting ? '发送中' : '锐评' }}
      </button>
    </view>

    <view class="comment-block">
      <text class="comment-heading">球迷锐评（{{ commentsCount }}）</text>
      <view v-if="comments.length" class="comment-list">
        <view v-for="item in comments" :key="item.id" class="comment-item">
          <view class="comment-avatar">
            <FiAvatar :src="item.author.avatar_url" :size="96" :name="item.author.nickname" />
          </view>
          <view class="comment-body">
            <view class="comment-top">
              <text class="comment-author">{{ item.author.nickname }}</text>
              <text class="comment-time">{{ formatShortDateTime(item.created_at) }}</text>
            </view>
            <text class="comment-text">{{ item.content }}</text>
          </view>
        </view>
      </view>
      <text v-else class="comment-empty">还没有人锐评，来抢沙发吐槽一句吧！</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import FiAvatar from '../../../components/FiAvatar.vue'
import FiReactionBar from '../../../components/FiReactionBar.vue'
import type { DirectorCommentDto } from '../../../types/director'
import { DIRECTOR_COMMENT_MAX, formatShortDateTime } from '../helpers'

const props = defineProps<{
  likesCount: number
  dislikesCount: number
  myReaction: 1 | -1 | 0 | null
  reactionPending: boolean
  commentsCount: number
  comments: DirectorCommentDto[]
  submitting: boolean
}>()

const emit = defineEmits<{
  (e: 'react', value: 1 | -1): void
  (e: 'submit-comment', text: string, onSuccess: () => void): void
}>()

const draftText = ref('')

function handleSubmit(): void {
  if (props.submitting || !draftText.value.trim()) return
  emit('submit-comment', draftText.value, () => {
    draftText.value = ''
  })
}
</script>

<style scoped lang="css">
.interact-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-surface);
  border: var(--fi-border-card);
  box-shadow: var(--fi-shadow-card);
}

.reaction-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.reaction-copy {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.reaction-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.reaction-sub {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.comment-composer {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  padding-top: var(--fi-space-14);
  border-top: var(--fi-border-divider);
}

.comment-input {
  flex: 1;
  min-width: 0;
  height: var(--fi-space-68);
  padding: 0 var(--fi-space-18);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-width) solid var(--fi-color-divider);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
}

.comment-send {
  flex-shrink: 0;
  margin: 0;
  padding: var(--fi-space-16) var(--fi-space-24);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.comment-send::after {
  border: 0;
}

.comment-send--disabled {
  opacity: var(--fi-opacity-pressed);
}

.comment-block {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
}

.comment-heading {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
}

.comment-list {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
}

.comment-item {
  display: flex;
  align-items: flex-start;
  gap: var(--fi-space-12);
}

.comment-avatar {
  flex-shrink: 0;
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  border-radius: var(--fi-radius-round);
  overflow: hidden;
  background: var(--fi-color-page-soft);
  font-size: var(--fi-font-22);
}

.comment-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-6);
}

.comment-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-8);
}

.comment-author {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.comment-time {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
}

.comment-text {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.comment-empty {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.comment-pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
