<template>
  <scroll-view scroll-y class="page-scroll">
    <view class="page">
      <FiLoading
        v-if="loading"
        title="战术板加载中"
        caption="正在整理这个球队的观点帖和洞察快照。"
      />

      <view v-else-if="errorMessage" class="state-card state-card--error">
        <text>{{ errorMessage }}</text>
      </view>

      <template v-else-if="board">
        <view class="hero-card team-board-hero">
          <view class="team-board-hero__back-row">
            <button class="team-board-hero__back" @click="goBackToInsights">返回洞察</button>
          </view>

          <view class="hero-card__top">
            <view>
              <text class="eyebrow">Team Board</text>
              <text class="hero-card__title">{{ board.team.team_name }}战术板</text>
            </view>
            <text class="hero-card__badge">第 {{ board.team.rank_no }} 名</text>
          </view>

          <text class="hero-card__summary">
            不发普通灌水帖，只围绕当前球队洞察卡发布观点。让讨论先站在数据上，再延伸到判断。
          </text>
        </view>

        <view class="panel team-board-launcher">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">发布观点</text>
              <text class="section-title">先选一张洞察卡</text>
            </view>
            <text class="meta-pill">{{ posts.length }} 篇观点</text>
          </view>

          <view class="team-board-launcher__grid">
            <button
              v-for="preset in composerPresets"
              :key="preset.insight_kind"
              class="team-board-launcher__card"
              @click="openComposer(preset.insight_kind)"
            >
              <view class="team-board-launcher__meta">
                <text>{{ preset.label }}</text>
                <text class="team-board-launcher__value">{{ preset.snapshot.summary_value }}</text>
              </view>
              <text class="team-board-launcher__summary">{{ preset.snapshot.summary_label }}</text>
              <text class="team-board-launcher__note">基于当前洞察卡发布观点</text>
            </button>
          </view>
        </view>

        <view v-if="!posts.length" class="panel state-card state-card--empty">
          <text>这个球队还没有观点帖。先从一张洞察卡开始，发布第一条数据观点。</text>
        </view>

        <view
          v-for="post in posts"
          :key="post.post_id"
          class="panel team-board-post"
        >
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

          <view class="team-board-snapshot">
            <view class="team-board-snapshot__summary">
              <view>
                <text class="team-board-snapshot__label">{{ post.snapshot.insight_label }}</text>
                <text class="team-board-snapshot__headline">{{ post.snapshot.summary_label }}</text>
              </view>
              <text class="team-board-snapshot__summary-value">{{ post.snapshot.summary_value }}</text>
            </view>

            <view class="team-board-snapshot__sections">
              <view
                v-for="section in post.snapshot.sections"
                :key="section.title"
                class="team-board-snapshot__section"
              >
                <view class="team-board-snapshot__section-head">
                  <text>{{ section.title }}</text>
                  <text>{{ section.metric_label }}</text>
                </view>

                <view class="team-board-snapshot__rows">
                  <view
                    v-for="item in section.items.slice(0, 3)"
                    :key="`${section.title}-${item.item_id ?? item.name}`"
                    class="team-board-snapshot__row"
                  >
                    <view class="team-board-snapshot__identity">
                      <image :src="item.avatar_storage_url || ''" mode="aspectFit" class="team-board-snapshot__avatar" />
                      <view>
                        <text class="team-board-snapshot__name">{{ item.name }}</text>
                        <text class="team-board-snapshot__metric">{{ metricSuffix(section.metric_label, item.value) }}</text>
                      </view>
                    </view>
                    <text class="team-board-snapshot__share">{{ formatShare(item.share) }}</text>
                  </view>
                </view>
              </view>
            </view>
          </view>

          <view class="team-board-post__actions">
            <button
              class="team-board-post__action"
              :class="{ active: post.liked_by_viewer }"
              @click="toggleLike(post.post_id)"
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
                v-model="commentDrafts[post.post_id]"
                class="team-board-comment-form__input"
                maxlength="280"
                placeholder="补一句你的判断，不用写长。"
              />
              <button class="team-board-comment-form__submit" @click="submitComment(post.post_id)">评论</button>
            </view>
          </view>
        </view>
      </template>

      <view v-if="composerOpen && selectedPreset" class="team-board-composer" @click.self="closeComposer">
        <view class="team-board-composer__dialog">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">发布观点</text>
              <text class="section-title">{{ selectedPreset.label }}</text>
            </view>
            <button class="team-board-composer__close" @click="closeComposer">关闭</button>
          </view>

          <view class="team-board-snapshot team-board-snapshot--composer">
            <view class="team-board-snapshot__summary">
              <view>
                <text class="team-board-snapshot__label">{{ selectedPreset.snapshot.insight_label }}</text>
                <text class="team-board-snapshot__headline">{{ selectedPreset.snapshot.summary_label }}</text>
              </view>
              <text class="team-board-snapshot__summary-value">{{ selectedPreset.snapshot.summary_value }}</text>
            </view>

            <view class="team-board-snapshot__sections">
              <view
                v-for="section in selectedPreset.snapshot.sections"
                :key="section.title"
                class="team-board-snapshot__section"
              >
                <view class="team-board-snapshot__section-head">
                  <text>{{ section.title }}</text>
                  <text>{{ section.metric_label }}</text>
                </view>

                <view class="team-board-snapshot__rows">
                  <view
                    v-for="item in section.items.slice(0, 3)"
                    :key="`${section.title}-${item.item_id ?? item.name}`"
                    class="team-board-snapshot__row"
                  >
                    <view class="team-board-snapshot__identity">
                      <image :src="item.avatar_storage_url || ''" mode="aspectFit" class="team-board-snapshot__avatar" />
                      <view>
                        <text class="team-board-snapshot__name">{{ item.name }}</text>
                        <text class="team-board-snapshot__metric">{{ metricSuffix(section.metric_label, item.value) }}</text>
                      </view>
                    </view>
                    <text class="team-board-snapshot__share">{{ formatShare(item.share) }}</text>
                  </view>
                </view>
              </view>
            </view>
          </view>

          <input
            v-model="postTitle"
            class="team-board-composer__input"
            maxlength="120"
            placeholder="例如：头部火力还是太集中"
          />

          <textarea
            v-model="postCommentary"
            class="team-board-composer__textarea"
            maxlength="280"
            placeholder="把你真正想说的判断写出来，不要复述数据。"
          />

          <text v-if="composerErrorMessage" class="team-board-composer__error">{{ composerErrorMessage }}</text>

          <button class="team-board-composer__submit" @click="publishPost">发布观点</button>
        </view>
      </view>
    </view>
  </scroll-view>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { onLoad, onShareAppMessage, onShow } from '@dcloudio/uni-app'
import FiLoading from '../../components/FiLoading.vue'
import {
  addTeamBoardComment,
  createTeamBoardPost,
  getTeamBoard,
  toggleTeamBoardPostLike,
} from '../../api/teamBoard'
import type {
  TeamBoardComposerPreset,
  TeamBoardInsightKind,
  TeamBoardViewResponse,
} from '../../types/teamBoard'
import { extractApiErrorMessage } from '../../utils/apiError'
import { getAccessToken } from '../../utils/authStorage'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'

const teamId = ref<number | null>(null)
const loading = ref(true)
const errorMessage = ref('')
const composerOpen = ref(false)
const selectedInsightKind = ref<TeamBoardInsightKind | null>(null)
const postTitle = ref('')
const postCommentary = ref('')
const composerErrorMessage = ref('')
const commentDrafts = reactive<Record<string, string>>({})
const board = ref<TeamBoardViewResponse | null>(null)

const posts = computed(() => board.value?.posts ?? [])
const composerPresets = computed(() => board.value?.composer_presets ?? [])
const selectedPreset = computed<TeamBoardComposerPreset | null>(() =>
  composerPresets.value.find((item) => item.insight_kind === selectedInsightKind.value) ?? null,
)

onShareAppMessage(() => ({
  title: board.value
    ? `${board.value.team.team_name}战术板，来看看这支队的数据观点`
    : '球队战术板：把洞察卡变成观点',
  path: teamId.value ? `/pages/team-board/index?teamId=${teamId.value}` : '/pages/insights/index',
}))

onLoad((query) => {
  const raw = query?.teamId
  teamId.value = typeof raw === 'string' ? Number(raw) : null
})

function formatShare(share: number): string {
  return `${(share * 100).toFixed(1)}%`
}

function metricSuffix(metricLabel: string, value: number): string {
  return `${value} ${metricLabel}`
}

function formatPublishedAt(value: string): string {
  const date = new Date(value)
  return `${date.getMonth() + 1}/${date.getDate()} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

async function loadBoard(): Promise<void> {
  if (!teamId.value) {
    errorMessage.value = '缺少球队 id'
    loading.value = false
    return
  }

  if (!getAccessToken()) {
    rememberPostLoginRedirect({
      type: 'navigateTo',
      url: `/pages/team-board/index?teamId=${teamId.value}`,
    })
    uni.switchTab({ url: '/pages/user/index' })
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    board.value = await getTeamBoard(teamId.value)
  } catch (error) {
    const message = extractApiErrorMessage(error, '战术板加载失败，请稍后重试。')
    if (message.includes('未登录') || message.includes('Unauthorized') || message.includes('not logged in')) {
      rememberPostLoginRedirect({
        type: 'navigateTo',
        url: `/pages/team-board/index?teamId=${teamId.value}`,
      })
      uni.switchTab({ url: '/pages/user/index' })
      return
    }

    errorMessage.value = message
  } finally {
    loading.value = false
  }
}

function openComposer(kind: TeamBoardInsightKind): void {
  selectedInsightKind.value = kind
  postTitle.value = ''
  postCommentary.value = ''
  composerErrorMessage.value = ''
  composerOpen.value = true
}

function closeComposer(): void {
  composerOpen.value = false
}

async function publishPost(): Promise<void> {
  if (!teamId.value || !selectedInsightKind.value) {
    return
  }

  try {
    const created = await createTeamBoardPost(teamId.value, {
      insight_kind: selectedInsightKind.value,
      title: postTitle.value.trim(),
      commentary: postCommentary.value.trim(),
    })

    if (board.value) {
      board.value = {
        ...board.value,
        posts: [created, ...board.value.posts],
      }
    }

    composerOpen.value = false
    selectedInsightKind.value = null
    postTitle.value = ''
    postCommentary.value = ''
    composerErrorMessage.value = ''
    uni.showToast({ title: '发布成功', icon: 'success' })
  } catch (error) {
    composerErrorMessage.value = extractApiErrorMessage(error, '发布失败，请稍后重试。')
  }
}

async function submitComment(postId: string): Promise<void> {
  const content = (commentDrafts[postId] ?? '').trim()
  if (!content) {
    return
  }

  try {
    const created = await addTeamBoardComment(postId, { content })

    if (board.value) {
      board.value = {
        ...board.value,
        posts: board.value.posts.map((post) => {
          if (post.post_id !== postId) {
            return post
          }

          return {
            ...post,
            comment_count: post.comment_count + 1,
            comments: [...post.comments, created],
          }
        }),
      }
    }

    commentDrafts[postId] = ''
    uni.showToast({ title: '评论成功', icon: 'success' })
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '评论失败'), icon: 'none' })
  }
}

async function toggleLike(postId: string): Promise<void> {
  try {
    const summary = await toggleTeamBoardPostLike(postId)

    if (board.value) {
      board.value = {
        ...board.value,
        posts: board.value.posts.map((post) =>
          post.post_id === postId
            ? {
                ...post,
                liked_by_viewer: summary.liked_by_viewer,
                like_count: summary.like_count,
              }
            : post,
        ),
      }
    }
  } catch (error) {
    uni.showToast({ title: extractApiErrorMessage(error, '支持失败'), icon: 'none' })
  }
}

function goBackToInsights(): void {
  uni.navigateBack({
    fail: () => {
      uni.switchTab({ url: '/pages/insights/index' })
    },
  })
}

onShow(() => {
  void loadBoard()
})
</script>

<style scoped lang="css">
.page-scroll { height: 100vh; }
.page { padding: 28rpx 24rpx 40rpx; display: flex; flex-direction: column; gap: 24rpx; }
.hero-card, .panel, .state-card, .team-board-composer__dialog {
  background: rgba(255,255,255,0.94);
  border-radius: 36rpx;
  padding: 28rpx;
  border: 2rpx solid rgba(236, 236, 241, 0.95);
  box-shadow: 0 28rpx 60rpx rgba(26,28,36,0.08);
}
.hero-card__top, .section-heading, .team-board-post__header, .team-board-post__actions, .team-board-snapshot__summary, .team-board-snapshot__section-head, .team-board-comment__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.hero-card__top, .section-heading, .team-board-post__header, .team-board-comment__head { align-items: flex-start; gap: 16rpx; }
.eyebrow, .section-kicker {
  margin: 0;
  color: #131313;
  font-size: 24rpx;
  font-weight: 700;
  letter-spacing: 2rpx;
}
.hero-card__title, .section-title, .team-board-post__title, .team-board-snapshot__headline {
  display: block;
  margin-top: 10rpx;
  color: #121212;
  font-size: 56rpx;
  line-height: 1.08;
  font-weight: 800;
}
.section-title, .team-board-post__title, .team-board-snapshot__headline { font-size: 44rpx; }
.hero-card__summary, .team-board-post__body, .team-board-launcher__summary {
  display: block;
  margin-top: 18rpx;
  color: #6b707b;
  font-size: 28rpx;
  line-height: 1.7;
}
.hero-card__badge, .meta-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: 1;
  box-sizing: border-box;
  padding: 14rpx 24rpx;
  border-radius: 999rpx;
  border: 2rpx solid rgba(229, 223, 205, 0.92);
  background: linear-gradient(180deg, rgba(252, 250, 245, 0.98), rgba(247, 243, 232, 0.94));
  color: #93876a;
  font-size: 24rpx;
}
.team-board-hero__back-row { margin-bottom: 16rpx; }
.team-board-hero__back {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: 1;
  padding: 10rpx 18rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.team-board-launcher__grid, .team-board-snapshot__sections, .team-board-comments, .post-list {
  display: grid;
  gap: 14rpx;
}
.team-board-launcher__grid {
  margin-top: 18rpx;
}
.team-board-launcher__card {
  display: grid;
  gap: 10rpx;
  padding: 22rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(230, 232, 239, 0.9);
  background: linear-gradient(180deg, rgba(252, 252, 255, 0.98), rgba(248, 249, 252, 0.96));
  text-align: left;
}
.team-board-launcher__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12rpx;
}
.team-board-launcher__meta text { font-size: 24rpx; color: #121212; font-weight: 700; }
.team-board-launcher__value { font-size: 34rpx !important; }
.team-board-launcher__note { color: #8f9198; font-size: 22rpx; }
.team-board-post__author {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.team-board-post__author-avatar, .team-board-snapshot__avatar {
  width: 48rpx;
  height: 48rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
}
.team-board-post__author-name, .team-board-snapshot__name, .team-board-comment__author {
  display: block;
  color: #121212;
  font-size: 26rpx;
  font-weight: 700;
}
.team-board-post__author-time, .team-board-snapshot__label, .team-board-snapshot__metric, .team-board-snapshot__share, .team-board-comment__time {
  color: #8f9198;
  font-size: 22rpx;
}
.team-board-post__body { margin-top: 12rpx; }
.team-board-snapshot {
  margin-top: 18rpx;
  padding: 22rpx;
  border-radius: 28rpx;
  background: #f7f8fb;
}
.team-board-snapshot__summary-value {
  color: #121212;
  font-size: 48rpx;
  font-weight: 800;
}
.team-board-snapshot__section {
  display: grid;
  gap: 10rpx;
}
.team-board-snapshot__rows {
  display: grid;
  gap: 10rpx;
}
.team-board-snapshot__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}
.team-board-snapshot__identity {
  display: flex;
  align-items: center;
  gap: 12rpx;
}
.team-board-post__actions {
  margin-top: 18rpx;
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
  padding: 12rpx 20rpx;
  border-radius: 999rpx;
  background: #f6f7fb;
  color: #6d7280;
  font-size: 24rpx;
}
.team-board-post__action.active {
  background: rgba(255, 106, 0, 0.12);
  color: #ff6a00;
}
.team-board-comments {
  margin-top: 18rpx;
}
.team-board-comment {
  padding: 18rpx 0;
  border-bottom: 2rpx solid #eff1f5;
}
.team-board-comment__body {
  display: block;
  margin-top: 8rpx;
  color: #565b67;
  font-size: 26rpx;
  line-height: 1.65;
}
.team-board-comment-form {
  display: grid;
  gap: 14rpx;
  margin-top: 10rpx;
}
.team-board-comment-form__input, .team-board-composer__input, .team-board-composer__textarea {
  width: 100%;
  padding: 20rpx 24rpx;
  border-radius: 22rpx;
  background: #f7f8fb;
  font-size: 28rpx;
}
.team-board-comment-form__input {
  min-height: 120rpx;
}
.team-board-comment-form__submit, .team-board-composer__submit, .team-board-composer__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  align-self: flex-start;
  padding: 20rpx 30rpx;
  border-radius: 999rpx;
  background: #15161b;
  color: #ffffff;
  font-size: 28rpx;
  white-space: nowrap;
  line-height: 1;
}
.team-board-composer {
  position: fixed;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 28rpx;
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}
.team-board-composer__dialog {
  width: 100%;
  max-height: 84vh;
  overflow-y: auto;
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}
.team-board-composer__textarea {
  min-height: 180rpx;
}
.team-board-composer__error {
  display: block;
  margin-top: 12rpx;
  color: #c03a2b;
  font-size: 24rpx;
}
.state-card--error text { font-size: 28rpx; color: #c03a2b; }
.state-card--empty text { color: #767a84; font-size: 26rpx; }
</style>
