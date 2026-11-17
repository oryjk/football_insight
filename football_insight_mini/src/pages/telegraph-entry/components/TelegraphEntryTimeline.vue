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

        <view v-if="entry.sources.length" class="x-node__source-wrap">
          <view
            class="x-node__source-toggle"
            role="button"
            :aria-expanded="sourcesExpanded"
            hover-class="timeline__pressed"
            @tap="sourcesExpanded = !sourcesExpanded"
          >
            <text>来源：{{ publishers }}</text>
            <text class="x-node__chevron" :class="{ 'x-node__chevron--open': sourcesExpanded }">›</text>
          </view>
          <view v-if="sourcesExpanded" class="x-node__sources">
            <view v-for="source in entry.sources" :key="source.url" class="x-node__source">
              <text class="x-node__source-title">{{ source.title }}</text>
              <view class="x-node__source-meta">
                <text>{{ source.publisher }} · {{ source.dateLabel }}</text>
                <text class="x-node__copy" role="button" hover-class="timeline__pressed" @tap="emit('copy-source', source.url)">
                  复制原文链接
                </text>
              </view>
            </view>
          </view>
        </view>

        <view class="x-node__entry-controls">
          <FiReactionBar
            size="md"
            :count="entry.likesCount"
            :dislike-count="entry.dislikesCount"
            :liked="entry.likedByMe"
            :disliked="entry.dislikedByMe"
            :pending="reactionPending"
            @like="emit('like-entry')"
            @dislike="emit('dislike-entry')"
          />
          <button
            v-if="commentsVisible"
            class="x-entry-comment-pill"
            hover-class="timeline__pressed"
            @tap="emit('focus-comment')"
          >
            <image class="x-entry-comment-pill__icon" src="/static/topics/comment-bubble.svg" mode="aspectFit" />
            <text class="x-entry-comment-pill__label">
              {{ entry.commentsCount > 0 ? `评论 ${formatCount(entry.commentsCount)}` : '写评论' }}
            </text>
          </button>
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
        <view v-for="(comment, index) in comments" :key="comment.id" class="x-node">
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
            <view v-if="index < comments.length - 1" class="x-node__line" />
          </view>

          <view class="x-node__main">
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
                :disabled="deletingCommentId === comment.id"
                aria-label="删除评论"
                hover-class="timeline__pressed"
                @tap.stop="confirmDelete(comment)"
              >
                <image class="x-node__delete-icon" src="/static/topics/trash-muted.svg" mode="aspectFit" />
              </button>
            </view>

            <view v-if="comment.replyTo" class="x-node__replying">
              <text class="x-node__replying-label">回复</text>
              <text class="x-node__replying-handle">@{{ comment.replyTo.authorDisplayName }}</text>
            </view>

            <text class="x-node__content">{{ comment.content }}</text>

            <view class="x-node__actions">
              <button
                class="x-action"
                aria-label="回复评论"
                hover-class="timeline__pressed"
                @tap.stop="emit('reply-comment', comment)"
              >
                <image class="x-action__icon" src="/static/topics/comment-bubble.svg" mode="aspectFit" />
                <text v-if="comment.repliesCount > 0" class="x-action__count">
                  {{ formatCount(comment.repliesCount) }}
                </text>
              </button>

              <button
                class="x-action"
                :class="{
                  'x-action--liked': comment.likedByMe,
                  'x-action--pending': pendingLikeIds.includes(comment.id),
                }"
                :aria-label="comment.likedByMe ? '取消评论点赞' : '点赞评论'"
                hover-class="timeline__pressed"
                @tap.stop="emit('like-comment', comment)"
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
                  {{ formatCount(comment.likesCount) }}
                </text>
              </button>
            </view>
          </view>
        </view>

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
          <text v-else class="x-timeline__end">已显示全部 {{ comments.length }} 条评论</text>
        </view>
      </template>
    </template>
  </view>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import FiReactionBar from '../../../components/FiReactionBar.vue'
import type { DailyTopicEntry, TelegraphComment } from '../../../types/dailyTopic'
import { formatTopicHeat, telegraphCategoryLabel } from '../../../utils/dailyTopics'

const props = defineProps<{
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
  (e: 'copy-source', url: string): void
  (e: 'like-entry'): void
  (e: 'dislike-entry'): void
  (e: 'focus-comment'): void
  (e: 'retry-comments'): void
  (e: 'load-more'): void
  (e: 'reply-comment', comment: TelegraphComment): void
  (e: 'like-comment', comment: TelegraphComment): void
  (e: 'delete-comment', comment: TelegraphComment): void
}>()

const sourcesExpanded = ref(false)
const publishers = computed(() =>
  Array.from(new Set(props.entry.sources.map(source => source.publisher))).join(' · '),
)

function authorInitial(name: string): string {
  const trimmed = name.trim()
  return trimmed ? trimmed.slice(0, 1).toUpperCase() : '球'
}

function formatCount(count: number): string {
  return formatTopicHeat(count)
}

function confirmDelete(comment: TelegraphComment): void {
  if (props.deletingCommentId) {
    return
  }
  uni.showModal({
    title: '删除评论',
    content: '确认删除这条评论吗？删除后不可恢复。',
    showCancel: true,
    success: (res) => {
      if (res.confirm) {
        emit('delete-comment', comment)
      }
    },
  })
}
</script>

<style scoped>
.x-timeline {
  background: var(--fi-color-surface);
  border-radius: var(--fi-radius-lg);
  padding: var(--fi-space-24) var(--fi-space-24) var(--fi-space-20);
  border: var(--fi-border-card);
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
.x-action::after,
.x-entry-comment-pill::after,
.x-timeline__retry::after,
.x-timeline__more::after {
  border: 0;
}

.x-node__delete-icon {
  width: 28rpx;
  height: 28rpx;
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

.x-node__source-wrap {
  margin-top: var(--fi-space-10);
}

.x-node__source-toggle {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  padding: var(--fi-space-6) 0;
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.x-node__chevron {
  display: inline-block;
  transform: rotate(90deg);
  transition: transform 0.2s;
}

.x-node__chevron--open {
  transform: rotate(-90deg);
}

.x-node__sources {
  margin-top: var(--fi-space-8);
  padding: var(--fi-space-4) var(--fi-space-16);
  border-radius: var(--fi-radius-md);
  background: var(--fi-color-page-soft);
}

.x-node__source {
  padding: var(--fi-space-12) 0;
  border-bottom: var(--fi-border-divider);
}

.x-node__source:last-child {
  border-bottom: 0;
}

.x-node__source-title {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.x-node__source-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-12);
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-22);
}

.x-node__copy {
  flex-shrink: 0;
  color: var(--fi-color-primary);
}

.x-node__entry-controls {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--fi-space-16);
  margin-top: var(--fi-space-18);
}

.x-entry-comment-pill {
  display: inline-flex;
  align-items: center;
  gap: var(--fi-space-8);
  height: var(--fi-component-reaction-height-md);
  margin: 0;
  padding: 0 var(--fi-space-24);
  border: var(--fi-border-width) solid var(--fi-color-reaction-border);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-reaction-bg);
  color: var(--fi-color-reaction-text);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: 1;
  box-sizing: border-box;
}

.x-entry-comment-pill__icon {
  width: 28rpx;
  height: 28rpx;
}

.x-entry-comment-pill__label {
  line-height: 1;
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
</style>
