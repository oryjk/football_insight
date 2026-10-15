<template>
  <view class="panel team-board-post">
    <view class="team-board-post__header">
      <view class="team-board-post__author">
        <image :src="post.author.avatar_url || ''" mode="aspectFill" class="team-board-post__author-avatar" />
        <view>
          <text class="team-board-post__author-name">{{ post.author.display_name }}</text>
          <text class="team-board-post__author-time">{{ formatPublishedAt(post.created_at) }}</text>
        </view>
      </view>
      <text class="meta-pill">{{ post.insight_label }}</text>
    </view>

    <view class="team-board-post__copy">
      <text class="team-board-post__title">{{ post.title }}</text>
      <text class="team-board-post__body">{{ post.commentary }}</text>
    </view>

    <TeamBoardSnapshot :snapshot="post.snapshot" />

    <view class="team-board-post__actions">
      <button
        class="team-board-post__action"
        :class="{ active: post.liked_by_viewer }"
        @click="emit('toggle-like', post.post_id)"
      >
        支持 {{ post.like_count }}
      </button>
      <text>{{ post.comment_count }} 条评论</text>
    </view>

    <view class="team-board-comments">
      <view
        v-for="comment in post.comments"
        :key="comment.comment_id"
        class="team-board-comment"
      >
        <view class="team-board-comment__head">
          <text class="team-board-comment__author">{{ comment.author.display_name }}</text>
          <text class="team-board-comment__time">{{ formatPublishedAt(comment.created_at) }}</text>
        </view>
        <text class="team-board-comment__body">{{ comment.content }}</text>
      </view>

      <view class="team-board-comment-form">
        <textarea
          v-model="draft"
          class="team-board-comment-form__input"
          maxlength="280"
          placeholder="补一句你的判断，不用写长。"
        />
        <button class="team-board-comment-form__submit" @click="emit('submit-comment', post.post_id)">评论</button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TeamBoardPost } from '../../../types/teamBoard'
import { formatPublishedAt } from '../helpers'
import TeamBoardSnapshot from './TeamBoardSnapshot.vue'

const props = defineProps<{ post: TeamBoardPost; commentDraft: string | undefined }>()
const emit = defineEmits<{
  (e: 'toggle-like', postId: string): void
  (e: 'submit-comment', postId: string): void
  (e: 'update:commentDraft', value: string): void
}>()
const draft = computed({
  get: () => props.commentDraft,
  set: (value: string) => emit('update:commentDraft', value),
})
</script>

<style scoped lang="css">
@import '../team-board-shared.css';

.team-board-post__author {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}
.team-board-post__body { margin-top: var(--fi-space-12); }
.team-board-post__actions {
  margin-top: var(--fi-space-18);
}
.team-board-post__actions text {
  color: #8f9198;
  font-size: 24rpx;
}
.team-board-post__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: var(--fi-space-12) var(--fi-space-20);
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.team-board-post__action.active {
  background: rgba(220, 38, 38, 0.12);
  color: #dc2626;
}
.team-board-comments {
  margin-top: var(--fi-space-18);
}
.team-board-comment {
  padding: var(--fi-space-18) 0;
  border-bottom: 2rpx solid #eff1f5;
}
.team-board-comment__body {
  display: block;
  margin-top: var(--fi-space-8);
  color: #565b67;
  font-size: 26rpx;
  line-height: 1.65;
}
.team-board-comment-form {
  display: grid;
  gap: var(--fi-space-14);
  margin-top: var(--fi-space-10);
}
.team-board-comment-form__input {
  min-height: 120rpx;
}
</style>
