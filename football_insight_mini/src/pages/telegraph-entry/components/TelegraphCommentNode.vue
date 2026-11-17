<template>
  <view
    class="x-node"
    :class="{ 'x-node--ancestor': ancestor }"
    role="button"
    :aria-label="`查看 ${comment.author.displayName} 的评论详情`"
    hover-class="node__pressed"
    @tap="emit('open', comment, false)"
  >
    <view class="x-node__rail">
      <view class="x-node__avatar">
        <image
          v-if="comment.author.avatarUrl"
          :src="comment.author.avatarUrl"
          mode="aspectFill"
          class="x-node__avatar-img"
        />
        <text v-else class="x-node__avatar-fallback">{{ authorInitial(comment.author.displayName) }}</text>
      </view>
      <view v-if="showLine" class="x-node__line" />
    </view>

    <view class="x-node__main" :class="{ 'x-node__main--ancestor': ancestor }">
      <view class="x-node__header">
        <view class="x-node__author-group">
          <text class="x-node__author">{{ comment.author.displayName }}</text>
          <text v-if="comment.isMine" class="x-node__mine">我</text>
          <text class="x-node__dot">·</text>
          <text class="x-node__time">{{ comment.relativeTimeLabel || comment.createdLabel }}</text>
        </view>

        <button
          v-if="comment.isMine"
          class="x-node__delete"
          :disabled="deleting"
          aria-label="删除评论"
          hover-class="node__pressed"
          @tap.stop="deleting ? undefined : emit('delete', comment)"
        >
          <image class="x-node__delete-icon" src="/static/topics/trash-muted.svg" mode="aspectFit" />
        </button>
      </view>

      <view v-if="showReplyTo && comment.replyTo" class="x-node__replying">
        <text class="x-node__replying-label">回复</text>
        <text class="x-node__replying-handle">@{{ comment.replyTo.authorDisplayName }}</text>
      </view>

      <text class="x-node__content">{{ comment.content }}</text>

      <view class="x-node__actions">
        <button
          class="x-action"
          aria-label="回复评论"
          hover-class="node__pressed"
          @tap.stop="emit('open', comment, true)"
        >
          <image class="x-action__icon" src="/static/topics/comment-bubble.svg" mode="aspectFit" />
          <text v-if="comment.repliesCount > 0" class="x-action__count">
            {{ formatTopicHeat(comment.repliesCount) }}
          </text>
        </button>

        <button
          class="x-action"
          :class="{
            'x-action--liked': comment.likedByMe,
            'x-action--pending': pendingLike,
          }"
          :aria-label="comment.likedByMe ? '取消评论点赞' : '点赞评论'"
          hover-class="node__pressed"
          @tap.stop="emit('like', comment)"
        >
          <image
            class="x-action__icon"
            :src="comment.likedByMe ? '/static/topics/heart-active.svg' : '/static/topics/heart-idle.svg'"
            mode="aspectFit"
          />
          <text
            v-if="comment.likesCount > 0"
            class="x-action__count"
            :class="{ 'x-action__count--liked': comment.likedByMe }"
          >
            {{ formatTopicHeat(comment.likesCount) }}
          </text>
        </button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { TelegraphComment } from '../../../types/dailyTopic'
import { formatTopicHeat } from '../../../utils/dailyTopics'

withDefaults(defineProps<{
  comment: TelegraphComment
  showLine?: boolean
  showReplyTo?: boolean
  ancestor?: boolean
  pendingLike?: boolean
  deleting?: boolean
}>(), {
  showLine: false,
  showReplyTo: false,
  ancestor: false,
  pendingLike: false,
  deleting: false,
})

const emit = defineEmits<{
  (e: 'open', comment: TelegraphComment, focusReply: boolean): void
  (e: 'like', comment: TelegraphComment): void
  (e: 'delete', comment: TelegraphComment): void
}>()

function authorInitial(name: string): string {
  const trimmed = name.trim()
  return trimmed ? trimmed.slice(0, 1).toUpperCase() : '球'
}
</script>

<style scoped>
.x-node {
  display: flex;
  align-items: stretch;
  gap: var(--fi-space-18);
}

.x-node__rail {
  width: 72rpx;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.x-node__avatar {
  width: 72rpx;
  height: 72rpx;
  flex-shrink: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  border: var(--fi-border-card);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.x-node__avatar-img {
  width: 100%;
  height: 100%;
}

.x-node__avatar-fallback {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
}

.x-node__line {
  width: 4rpx;
  flex: 1;
  min-height: 28rpx;
  margin-top: var(--fi-space-6);
  margin-bottom: var(--fi-space-6);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-timeline-rail);
}

.x-node__main {
  min-width: 0;
  flex: 1;
  padding-bottom: var(--fi-space-24);
  display: flex;
  flex-direction: column;
}

.x-node__main--ancestor {
  padding-bottom: var(--fi-space-22);
}

.x-node__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
  min-height: 40rpx;
}

.x-node__author-group {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
}

.x-node__author {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-snug);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.x-node__mine {
  flex-shrink: 0;
  padding: var(--fi-space-2) var(--fi-space-8);
  border-radius: var(--fi-radius-xs);
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-bold);
  line-height: 1.2;
}

.x-node__dot,
.x-node__time {
  flex-shrink: 0;
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
}

.x-node__delete {
  flex-shrink: 0;
  width: 48rpx;
  height: 48rpx;
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
}

.x-node__delete::after,
.x-action::after {
  border: 0;
}

.x-node__delete-icon {
  width: 28rpx;
  height: 28rpx;
}

.x-node__replying {
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
  margin-top: var(--fi-space-4);
}

.x-node__replying-label {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.x-node__replying-handle {
  color: var(--fi-color-outsider);
  font-size: var(--fi-font-22);
}

.x-node__content {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-normal);
  word-break: break-word;
  white-space: pre-wrap;
}

.x-node__actions {
  display: flex;
  align-items: center;
  gap: var(--fi-space-40);
  margin-top: var(--fi-space-12);
}

.x-action {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  margin: 0;
  padding: var(--fi-space-6) var(--fi-space-8) var(--fi-space-6) 0;
  border: 0;
  background: transparent;
  color: var(--fi-color-text-secondary);
  line-height: 1;
}

.x-action--pending {
  opacity: var(--fi-opacity-pressed);
}

.x-action__icon {
  width: 30rpx;
  height: 30rpx;
}

.x-action__count {
  font-size: var(--fi-font-22);
  color: var(--fi-color-text-secondary);
  font-variant-numeric: tabular-nums;
}

.x-action__count--liked {
  color: var(--fi-color-heart-like);
  font-weight: var(--fi-weight-bold);
}

.node__pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
