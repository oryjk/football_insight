<template>
  <view class="x-thread">
    <!-- 上游节点 1：原电报摘要节点（与后续祖先/焦点评论通过左侧轨道竖线连接） -->
    <view
      class="x-node x-node--ancestor"
      role="button"
      aria-label="查看原电报详情"
      hover-class="thread__pressed"
      @tap="emit('open-entry')"
    >
      <view class="x-node__rail">
        <view class="x-node__avatar x-node__avatar--brand">
          <image class="x-node__avatar-img" src="/static/app-logo-mark.png" mode="aspectFill" />
        </view>
        <view class="x-node__line" />
      </view>

      <view class="x-node__main">
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

        <text class="x-node__entry-snippet">
          <text v-if="entry.title" class="x-node__entry-title">【{{ entry.title }}】</text>{{ entry.text }}
        </text>
      </view>
    </view>

    <!-- 上游节点 2：所有可见祖先评论（按从根到直接父评论顺序排列） -->
    <TelegraphCommentNode
      v-for="ancestor in ancestors"
      :key="ancestor.id"
      :comment="ancestor"
      show-line
      ancestor
      :pending-like="pendingLikeIds.includes(ancestor.id)"
      :deleting="deletingCommentId === ancestor.id"
      @open="(target, focusReply) => emit('open-comment', target, focusReply)"
      @like="emit('like-comment', $event)"
      @delete="emit('delete-comment', $event)"
    />

    <!-- 中心焦点评论主卡（X 展开态） -->
    <view class="x-focus">
      <view class="x-focus__header">
        <view class="x-node__avatar">
          <image
            v-if="focalComment.author.avatarUrl"
            :src="focalComment.author.avatarUrl"
            mode="aspectFill"
            class="x-node__avatar-img"
          />
          <text v-else class="x-node__avatar-fallback">{{ authorInitial(focalComment.author.displayName) }}</text>
        </view>

        <view class="x-focus__identity">
          <view class="x-node__author-group">
            <text class="x-focus__author">{{ focalComment.author.displayName }}</text>
            <text v-if="focalComment.isMine" class="x-node__mine">我</text>
          </view>
          <view v-if="focalComment.replyTo" class="x-focus__replying">
            <text class="x-node__replying-label">回复</text>
            <text class="x-node__replying-handle">@{{ focalComment.replyTo.authorDisplayName }}</text>
          </view>
        </view>

        <button
          v-if="focalComment.isMine"
          class="x-node__delete"
          :disabled="deletingCommentId === focalComment.id"
          aria-label="删除评论"
          hover-class="thread__pressed"
          @tap.stop="deletingCommentId === focalComment.id ? undefined : emit('delete-comment', focalComment)"
        >
          <image class="x-node__delete-icon" src="/static/topics/trash-muted.svg" mode="aspectFit" />
        </button>
      </view>

      <text class="x-focus__content">{{ focalComment.content }}</text>

      <view class="x-focus__meta">
        <text class="x-focus__time">{{ focalComment.createdLabel }}</text>
      </view>

      <view class="x-focus__stats">
        <view class="x-focus__stat">
          <text class="x-focus__stat-num">{{ formatCount(repliesTotalCount) }}</text>
          <text class="x-focus__stat-label">条回复</text>
        </view>
        <view class="x-focus__stat">
          <text class="x-focus__stat-num">{{ formatCount(focalComment.likesCount) }}</text>
          <text class="x-focus__stat-label">赞</text>
        </view>
      </view>

      <view class="x-focus__controls">
        <button
          class="x-focus__action"
          aria-label="回复这条评论"
          hover-class="thread__pressed"
          @tap="emit('focus-reply')"
        >
          <image class="x-focus__action-icon" src="/static/topics/comment-bubble.svg" mode="aspectFit" aria-hidden="true" />
          <text class="x-focus__action-label">回复</text>
        </button>

        <button
          class="x-focus__action"
          :class="{
            'x-focus__action--liked': focalComment.likedByMe,
            'x-focus__action--pending': pendingLikeIds.includes(focalComment.id),
          }"
          :aria-label="focalComment.likedByMe ? '取消评论点赞' : '点赞评论'"
          hover-class="thread__pressed"
          @tap="emit('like-comment', focalComment)"
        >
          <image
            class="x-focus__action-icon"
            :src="focalComment.likedByMe ? '/static/topics/heart-active.svg' : '/static/topics/heart-idle.svg'"
            mode="aspectFit"
            aria-hidden="true"
          />
          <text class="x-focus__action-label">{{ focalComment.likesCount > 0 ? formatCount(focalComment.likesCount) : '赞' }}</text>
        </button>

        <!-- #ifdef MP-WEIXIN -->
        <button
          class="x-focus__action"
          aria-label="分享评论详情"
          open-type="share"
          hover-class="thread__pressed"
        >
          <image class="x-focus__action-icon" src="/static/lineup/share.png" mode="aspectFit" aria-hidden="true" />
        </button>
        <!-- #endif -->
        <!-- #ifndef MP-WEIXIN -->
        <button
          class="x-focus__action"
          aria-label="分享评论详情"
          hover-class="thread__pressed"
          @tap="emit('share')"
        >
          <image class="x-focus__action-icon" src="/static/lineup/share.png" mode="aspectFit" aria-hidden="true" />
        </button>
        <!-- #endif -->
      </view>
    </view>

    <!-- 下方时间线：只展示在此评论下继续发表的直接子回复 -->
    <view class="x-replies">
      <view v-if="repliesLoading && !replies.length" class="x-timeline__state">
        <text>正在加载评论回复…</text>
      </view>

      <view v-else-if="repliesError && !replies.length" class="x-timeline__state">
        <text class="x-timeline__error">{{ repliesError }}</text>
        <button class="x-timeline__retry" hover-class="thread__pressed" @tap="emit('retry-replies')">
          重新加载
        </button>
      </view>

      <view v-else-if="!replies.length" class="x-timeline__state">
        <text>还没有人继续回复，在下方写下你的回应</text>
      </view>

      <template v-else>
        <TelegraphCommentNode
          v-for="(reply, index) in replies"
          :key="reply.id"
          :comment="reply"
          :show-line="index < replies.length - 1"
          :pending-like="pendingLikeIds.includes(reply.id)"
          :deleting="deletingCommentId === reply.id"
          @open="(target, focusReply) => emit('open-comment', target, focusReply)"
          @like="emit('like-comment', $event)"
          @delete="emit('delete-comment', $event)"
        />

        <view class="x-timeline__footer">
          <button
            v-if="hasMore"
            class="x-timeline__more"
            :disabled="loadingMore"
            hover-class="thread__pressed"
            @tap="emit('load-more')"
          >
            {{ loadingMore ? '正在加载更多回复…' : '加载更多回复' }}
          </button>
          <text v-else class="x-timeline__end">已显示全部 {{ replies.length }} 条回复</text>
        </view>
      </template>
    </view>
  </view>
</template>

<script setup lang="ts">
import type { DailyTopicEntry, TelegraphComment } from '../../../types/dailyTopic'
import { formatTopicHeat, telegraphCategoryLabel } from '../../../utils/dailyTopics'
import TelegraphCommentNode from './TelegraphCommentNode.vue'

defineProps<{
  entry: DailyTopicEntry
  dayLabel: string
  ancestors: TelegraphComment[]
  focalComment: TelegraphComment
  replies: TelegraphComment[]
  repliesTotalCount: number
  repliesLoading: boolean
  loadingMore: boolean
  hasMore: boolean
  repliesError: string
  pendingLikeIds: string[]
  deletingCommentId: string
}>()

const emit = defineEmits<{
  (e: 'open-entry'): void
  (e: 'open-comment', comment: TelegraphComment, focusReply: boolean): void
  (e: 'focus-reply'): void
  (e: 'like-comment', comment: TelegraphComment): void
  (e: 'delete-comment', comment: TelegraphComment): void
  (e: 'retry-replies'): void
  (e: 'load-more'): void
  (e: 'share'): void
}>()

function authorInitial(name: string): string {
  const trimmed = name.trim()
  return trimmed ? trimmed.slice(0, 1).toUpperCase() : '球'
}

function formatCount(count: number): string {
  return formatTopicHeat(count)
}
</script>

<style scoped>
.x-thread {
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
  padding-bottom: var(--fi-space-22);
  display: flex;
  flex-direction: column;
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
.x-node__time,
.x-node__day {
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
.x-focus__action::after,
.x-timeline__retry::after,
.x-timeline__more::after {
  border: 0;
}

.x-node__delete-icon {
  width: 28rpx;
  height: 28rpx;
}

.x-node__entry-snippet {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-top: var(--fi-space-8);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
  word-break: break-word;
}

.x-node__entry-title {
  color: var(--fi-color-text-primary);
  font-weight: var(--fi-weight-bold);
}

.x-node__replying-label {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.x-node__replying-handle {
  color: var(--fi-color-outsider);
  font-size: var(--fi-font-22);
}

/* 焦点评论主卡（X 展开风格） */
.x-focus {
  padding-top: var(--fi-space-4);
  padding-bottom: var(--fi-space-4);
}

.x-focus__header {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
}

.x-focus__identity {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-4);
}

.x-focus__author {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-snug);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.x-focus__replying {
  display: flex;
  align-items: center;
  gap: var(--fi-space-6);
}

.x-focus__content {
  display: block;
  margin-top: var(--fi-space-20);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-34);
  line-height: var(--fi-leading-relaxed);
  word-break: break-word;
  white-space: pre-wrap;
}

.x-focus__meta {
  margin-top: var(--fi-space-18);
  padding-bottom: var(--fi-space-16);
  border-bottom: var(--fi-border-divider);
}

.x-focus__time {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-24);
}

.x-focus__stats {
  display: flex;
  align-items: center;
  gap: var(--fi-space-28);
  padding: var(--fi-space-16) 0;
  border-bottom: var(--fi-border-divider);
}

.x-focus__stat {
  display: inline-flex;
  align-items: baseline;
  gap: var(--fi-space-6);
}

.x-focus__stat-num {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  font-variant-numeric: tabular-nums;
}

.x-focus__stat-label {
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-24);
}

.x-focus__controls {
  display: flex;
  align-items: center;
  justify-content: space-around;
  padding: var(--fi-space-12) 0;
  border-bottom: var(--fi-border-divider);
}

.x-focus__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-8);
  min-height: var(--fi-space-48);
  margin: 0;
  padding: 0 var(--fi-space-16);
  border: 0;
  background: transparent;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-none);
}

.x-focus__action--liked {
  color: var(--fi-color-heart-like);
  font-weight: var(--fi-weight-bold);
}

.x-focus__action--pending {
  opacity: var(--fi-opacity-pressed);
}

.x-focus__action-icon {
  width: var(--fi-space-32);
  height: var(--fi-space-32);
  flex-shrink: 0;
}

.x-focus__action-label {
  font-variant-numeric: tabular-nums;
}

.x-replies {
  margin-top: var(--fi-space-24);
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

.thread__pressed {
  opacity: var(--fi-opacity-pressed);
}
</style>
