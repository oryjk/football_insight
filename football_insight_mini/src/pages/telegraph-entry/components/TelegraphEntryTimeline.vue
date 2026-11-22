<template>
  <view class="x-timeline">
    <!-- 顶部原电报起点：左侧头像 + 向第一条评论延伸的时间线 -->
    <view class="x-node x-node--origin">
      <view class="x-node__rail">
        <view class="x-node__avatar x-node__avatar--brand">
          <image class="x-node__avatar-img" src="/static/app-logo-mark.png" mode="aspectFill" />
        </view>
        <view
          v-if="commentsVisible && (comments.length > 0 || commentsLoading)"
          class="x-node__line"
        />
      </view>

      <view class="x-node__main x-node__main--origin">
        <view class="x-node__header">
          <view class="x-node__author-group">
            <text class="x-node__author">每日电报</text>
            <text class="x-node__tag" :class="`x-node__tag--${entry.category}`">
              {{ telegraphCategoryLabel(entry.category) }}
            </text>
            <text class="x-node__dot">·</text>
            <text class="x-node__time">{{ entry.timeLabel }}</text>
          </view>
          <text v-if="dayLabel" class="x-node__day">{{ dayLabel }}</text>
        </view>

        <text class="x-node__entry-body">
          <text v-if="entry.title" class="x-node__entry-title">【{{ entry.title }}】</text>{{ entry.text }}
        </text>

        <view class="x-node__entry-controls">
          <button
            v-if="commentsVisible"
            class="x-entry-action"
            :aria-label="`发表评论，现有 ${entry.commentsCount} 条评论`"
            hover-class="timeline__pressed"
            @tap="emit('focus-comment')"
          >
            <image class="x-entry-action__icon" src="/static/topics/comment-bubble.svg" mode="aspectFit" aria-hidden="true" />
            <text class="x-entry-action__count">{{ formatCount(entry.commentsCount) }}</text>
          </button>
          <button
            class="x-entry-action"
            :class="{ 'x-entry-action--liked': entry.likedByMe, 'x-entry-action--pending': reactionPending }"
            :aria-label="entry.likedByMe ? '取消点赞' : '点赞'"
            :aria-pressed="entry.likedByMe"
            :aria-disabled="reactionPending"
            hover-class="timeline__pressed"
            @tap="reactionPending ? undefined : emit('like-entry')"
          >
            <image class="x-entry-action__icon" :src="entry.likedByMe ? '/static/lineup/reaction-like-active.png' : '/static/lineup/reaction-like-idle.png'" mode="aspectFit" aria-hidden="true" />
            <text class="x-entry-action__count">{{ formatCount(entry.likesCount) }}</text>
          </button>
          <button
            class="x-entry-action"
            :class="{ 'x-entry-action--disliked': entry.dislikedByMe, 'x-entry-action--pending': reactionPending }"
            :aria-label="entry.dislikedByMe ? '取消点踩' : '点踩'"
            :aria-pressed="entry.dislikedByMe"
            :aria-disabled="reactionPending"
            hover-class="timeline__pressed"
            @tap="reactionPending ? undefined : emit('dislike-entry')"
          >
            <image class="x-entry-action__icon" :src="entry.dislikedByMe ? '/static/lineup/reaction-dislike-active.png' : '/static/lineup/reaction-dislike-idle.png'" mode="aspectFit" aria-hidden="true" />
            <text class="x-entry-action__count">{{ formatCount(entry.dislikesCount) }}</text>
          </button>
          <!-- #ifdef MP-WEIXIN -->
          <button
            class="x-entry-action"
            aria-label="分享电报"
            open-type="share"
            hover-class="timeline__pressed"
          >
            <image class="x-entry-action__icon" src="/static/lineup/share.png" mode="aspectFit" aria-hidden="true" />
          </button>
          <!-- #endif -->
          <!-- #ifndef MP-WEIXIN -->
          <button
            class="x-entry-action"
            aria-label="分享电报"
            hover-class="timeline__pressed"
            @tap="emit('share')"
          >
            <image class="x-entry-action__icon" src="/static/lineup/share.png" mode="aspectFit" aria-hidden="true" />
          </button>
          <!-- #endif -->
        </view>
      </view>
    </view>

    <!-- X 风格连续评论时间线 -->
    <template v-if="commentsVisible">
      <view v-if="commentsLoading && !comments.length" class="x-timeline__state">
        <text>正在加载评论时间线…</text>
      </view>

      <view v-else-if="commentsError && !comments.length" class="x-timeline__state">
        <text class="x-timeline__error">{{ commentsError }}</text>
        <button class="x-timeline__retry" hover-class="timeline__pressed" @tap="emit('retry-comments')">
          重新加载
        </button>
      </view>

      <view v-else-if="!comments.length" class="x-timeline__state">
        <text>还没有球迷评论，点击下方写下第一条回复</text>
      </view>

      <template v-else>
        <TelegraphCommentNode
          v-for="(comment, index) in comments"
          :key="comment.id"
          :comment="comment"
          :show-line="index < comments.length - 1"
          show-reply-to
          :pending-like="pendingLikeIds.includes(comment.id)"
          :deleting="deletingCommentId === comment.id"
          @open="(target, focusReply) => emit('open-comment', target, focusReply)"
          @like="emit('like-comment', $event)"
          @delete="emit('delete-comment', $event)"
        />

        <view class="x-timeline__footer">
          <button
            v-if="hasMore"
            class="x-timeline__more"
            :disabled="loadingMore"
            hover-class="timeline__pressed"
            @tap="emit('load-more')"
          >
            {{ loadingMore ? '正在加载更多评论…' : '加载更多评论' }}
          </button>
          <text v-else class="x-timeline__end">已显示全部评论</text>
        </view>
      </template>
    </template>
  </view>
</template>

<script setup lang="ts">
import type { DailyTopicEntry, TelegraphComment } from '../../../types/dailyTopic'
import { formatTopicHeat, telegraphCategoryLabel } from '../../../utils/dailyTopics'
import TelegraphCommentNode from './TelegraphCommentNode.vue'

defineProps<{
  entry: DailyTopicEntry
  dayLabel: string
  commentsVisible: boolean
  reactionPending: boolean
  comments: TelegraphComment[]
  commentsLoading: boolean
  loadingMore: boolean
  hasMore: boolean
  commentsError: string
  pendingLikeIds: string[]
  deletingCommentId: string
}>()

const emit = defineEmits<{
  (e: 'share'): void
  (e: 'like-entry'): void
  (e: 'dislike-entry'): void
  (e: 'focus-comment'): void
  (e: 'retry-comments'): void
  (e: 'load-more'): void
  (e: 'open-comment', comment: TelegraphComment, focusReply: boolean): void
  (e: 'like-comment', comment: TelegraphComment): void
  (e: 'delete-comment', comment: TelegraphComment): void
}>()

function formatCount(count: number): string {
  return formatTopicHeat(count)
}
</script>

<style scoped>
.x-timeline {
  padding: var(--fi-space-4) var(--fi-space-4) var(--fi-space-20);
}

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

.x-node__avatar--brand {
  background: var(--fi-color-surface);
}

.x-node__avatar-img {
  width: 100%;
  height: 100%;
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
  padding-bottom: var(--fi-space-28);
  display: flex;
  flex-direction: column;
}

.x-node__main--origin {
  padding-bottom: var(--fi-space-24);
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

.x-node__tag {
  flex-shrink: 0;
  padding: var(--fi-space-2) var(--fi-space-10);
  border-radius: var(--fi-radius-xs);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-bold);
  line-height: 1.3;
}

.x-node__tag--domestic {
  background: var(--fi-color-primary-soft);
  color: var(--fi-color-primary-deep);
}

.x-node__tag--world {
  background: var(--fi-color-outsider-soft);
  color: var(--fi-color-outsider);
}

.x-node__tag--abroad {
  background: var(--fi-color-hat-trick-bg);
  color: var(--fi-color-hat-trick-text);
}

.x-node__dot,
.x-node__time,
.x-node__day {
  flex-shrink: 0;
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-snug);
}

.x-entry-action::after,
.x-timeline__retry::after,
.x-timeline__more::after {
  border: 0;
}

.x-node__entry-body {
  display: block;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-30);
  line-height: var(--fi-leading-relaxed);
  word-break: break-word;
  white-space: pre-wrap;
}

.x-node__entry-title {
  font-weight: var(--fi-weight-bold);
}

.x-node__entry-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-18);
  padding-top: var(--fi-space-12);
  border-top: var(--fi-border-divider);
}

.x-entry-action {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  min-height: var(--fi-space-48);
  margin: 0;
  padding: 0 var(--fi-space-8);
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
  box-sizing: border-box;
}

.x-entry-action--liked {
  color: var(--fi-color-reaction-like);
}

.x-entry-action--disliked {
  color: var(--fi-color-reaction-dislike);
}

.x-entry-action--pending {
  opacity: var(--fi-opacity-pressed);
}

.x-entry-action__icon {
  flex-shrink: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
}

.x-entry-action__count {
  line-height: var(--fi-leading-none);
  font-variant-numeric: tabular-nums;
}

.x-timeline__state {
  padding: var(--fi-space-32) var(--fi-space-16);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-14);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-26);
  text-align: center;
}

.x-timeline__error {
  color: var(--fi-color-primary);
}

.x-timeline__retry,
.x-timeline__more {
  margin: 0;
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page-soft);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: 1.4;
}

.x-timeline__footer {
  padding: var(--fi-space-12) 0 var(--fi-space-4);
  display: flex;
  justify-content: center;
}

.x-timeline__end {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.timeline__pressed {
  opacity: var(--fi-opacity-pressed);
}

@media (prefers-reduced-motion: reduce) {
  .x-node__chevron {
    transition: none;
  }
}
</style>
