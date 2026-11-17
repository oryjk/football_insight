<template>
  <page-meta :page-style="pageStyle" />
  <FiSecondaryNav :share-disabled="!focalComment || !entry" @back="goBack" @share="copyShareLink" />
  <view class="comment-page-offset">
    <view class="comment-page" :class="{ 'comment-page--with-composer': Boolean(entry && focalComment && commentsVisible) }">
      <view v-if="loading && !focalComment" class="comment-page__state">
        <text>正在加载评论详情…</text>
      </view>

      <view v-else-if="errorMessage && !focalComment" class="comment-page__state">
        <text class="comment-page__error">{{ errorMessage }}</text>
        <button class="comment-page__retry" @tap="loadInitial(false)">重新加载</button>
      </view>

      <template v-else-if="entry && focalComment">
        <TelegraphCommentThread
          :entry="entry"
          :day-label="dayLabel"
          :ancestors="ancestors"
          :focal-comment="focalComment"
          :replies="replies"
          :replies-total-count="repliesTotalCount"
          :replies-loading="repliesLoading"
          :loading-more="loadingMoreReplies"
          :has-more="repliesHasMore"
          :replies-error="repliesError"
          :pending-like-ids="pendingCommentLikeIds"
          :deleting-comment-id="deletingCommentId"
          @open-entry="page.openEntry"
          @open-comment="page.openComment"
          @focus-reply="page.focusReplyComposer"
          @like-comment="page.toggleCommentLike"
          @delete-comment="page.deleteOwnComment"
          @retry-replies="page.reloadReplies"
          @load-more="page.loadMoreReplies"
          @share="copyShareLink"
        />
      </template>
    </view>
  </view>

  <TelegraphEntryComposer
    :visible="Boolean(entry && focalComment && commentsVisible)"
    :logged-in="isLoggedIn"
    :profile-complete="profileComplete"
    :draft="draftContent"
    :default-placeholder="composerPlaceholder"
    :submitting="submittingReply"
    :focus-trigger="inputFocusTrigger"
    @update:draft="draftContent = $event"
    @focus-change="composerFocused = $event"
    @login="page.requestLogin"
    @setup-profile="page.openProfileSetup"
    @submit="page.submitReply"
  />

  <FiProfileSetupSheet
    :visible="profileSheetVisible"
    :submitting="savingProfile"
    :initial-nickname="editableNicknameInitialValue(session.currentUser.value)"
    :initial-avatar-url="session.currentUser.value?.avatar_url ?? null"
    @update:visible="profileSheetVisible = $event"
    @submit="page.saveProfileAndContinue"
  />
  <canvas :canvas-id="TELEGRAPH_ENTRY_SHARE_CANVAS_ID" class="comment-share-canvas" :style="shareCanvasStyle" />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { onHide, onLoad, onPullDownRefresh, onReachBottom, onShareAppMessage, onShareTimeline, onShow } from '@dcloudio/uni-app'
import FiProfileSetupSheet from '../../components/FiProfileSetupSheet.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import { roundupDayLabel, telegraphCommentUrl } from '../../utils/dailyTopics'
import { editableNicknameInitialValue } from '../../utils/profileGate'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import { buildSharePath, buildShareQuery } from '../../utils/shareSource'
import {
  buildTelegraphEntrySharePayload,
  cleanTelegraphHeadline,
  TELEGRAPH_SHARE_CARD,
} from '../../utils/telegraphShareCard'
import TelegraphCommentThread from './components/TelegraphCommentThread.vue'
import TelegraphEntryComposer from './components/TelegraphEntryComposer.vue'
import { useTelegraphCommentPage } from './useTelegraphCommentPage'
import { TELEGRAPH_ENTRY_SHARE_CANVAS_ID, useTelegraphEntryShareCard } from './useTelegraphEntryShareCard'

const VALID_ID_RE = /^[a-zA-Z0-9_-]{1,128}$/

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const miniReview = useMiniReviewStatus()
const shareCard = useTelegraphEntryShareCard()
// 分享图按实际像素绘制，与页面布局的 rpx 无关。
const shareCanvasStyle = { width: `${TELEGRAPH_SHARE_CARD.width}px`, height: `${TELEGRAPH_SHARE_CARD.height}px` }

let topicId = ''
let entryId = ''
let commentId = ''
let lastSessionRevision = session.revision.value
let hasLeftPage = false

const page = useTelegraphCommentPage({
  topicId: () => topicId,
  entryId: () => entryId,
  commentId: () => commentId,
  session,
  miniReview,
})

const {
  topic,
  entry,
  ancestors,
  focalComment,
  replies,
  repliesTotalCount,
  loading,
  errorMessage,
  commentsVisible,
  isLoggedIn,
  profileComplete,
  repliesLoading,
  loadingMoreReplies,
  repliesError,
  repliesHasMore,
  pendingCommentLikeIds,
  draftContent,
  inputFocusTrigger,
  submittingReply,
  deletingCommentId,
  profileSheetVisible,
  savingProfile,
  loadInitial,
} = page

const composerFocused = ref(false)
const dayLabel = computed(() => (topic.value ? roundupDayLabel(topic.value.publishedAt) : ''))
const composerPlaceholder = computed(() => (
  focalComment.value
    ? `回复 @${focalComment.value.author.displayName}…（最多 200 字）`
    : '发布你的回复（最多 200 字）'
))
const pageStyle = computed(() =>
  profileSheetVisible.value || composerFocused.value
    ? `${navigationPageStyle.value};overflow:hidden;`
    : navigationPageStyle.value,
)

function goBack(): void {
  goBackFromSecondaryPage()
}

function buildCommentSharePayload(imageUrl?: string) {
  const valid = VALID_ID_RE.test(topicId) && VALID_ID_RE.test(entryId) && VALID_ID_RE.test(commentId)
  if (!valid) {
    return buildTelegraphEntrySharePayload({ topicId, entryId, entry: entry.value, imageUrl })
  }
  const comment = focalComment.value
  const currentEntry = entry.value
  const headline = currentEntry ? cleanTelegraphHeadline(currentEntry.title) || currentEntry.title : '每日电报'
  const title = comment
    ? `${comment.author.displayName}：${comment.content.slice(0, 26)}｜${headline}`
    : headline
  return {
    title,
    path: buildSharePath(telegraphCommentUrl(topicId, entryId, commentId), 'telegraph_comment'),
    ...(imageUrl ? { imageUrl } : {}),
  }
}

function copyShareLink(): void {
  copyPageShareLink(buildCommentSharePayload().path)
}

watch(
  [entry, () => topic.value?.publishedAt, commentsVisible],
  ([currentEntry, publishedAt, visible]) => {
    if (currentEntry) {
      void shareCard.prepare(currentEntry, { publishedAt, commentsVisible: visible })
    } else {
      shareCard.reset()
    }
  },
)

onLoad((query) => {
  topicId = typeof query?.topicId === 'string' ? decodeURIComponent(query.topicId) : ''
  entryId = typeof query?.entryId === 'string' ? decodeURIComponent(query.entryId) : ''
  commentId = typeof query?.commentId === 'string' ? decodeURIComponent(query.commentId) : ''
  shareCard.reset()
  const focus = query?.focus === '1'
  lastSessionRevision = session.revision.value
  void miniReview.ensureMiniReviewReady()
  void session.ensureSessionReady()
  void page.loadInitial(focus)
})

onHide(() => {
  hasLeftPage = true
})

onShow(() => {
  void miniReview.ensureMiniReviewReady()
  void session.ensureSessionReady()
  if (topicId && entryId && commentId && topic.value && (hasLeftPage || session.revision.value !== lastSessionRevision)) {
    hasLeftPage = false
    lastSessionRevision = session.revision.value
    void page.refreshAll()
  }
})

onPullDownRefresh(() => {
  void page.refreshAll().finally(() => uni.stopPullDownRefresh())
})

onReachBottom(() => {
  if (commentsVisible.value && repliesHasMore.value) {
    void page.loadMoreReplies()
  }
})

onShareAppMessage(() => {
  const currentEntry = entry.value
  const options = { publishedAt: topic.value?.publishedAt, commentsVisible: commentsVisible.value }
  const fallback = buildCommentSharePayload(shareCard.imageUrl.value)
  if (!currentEntry || shareCard.imageUrl.value) return fallback
  return {
    ...fallback,
    promise: shareCard.prepare(currentEntry, options).then(imageUrl => buildCommentSharePayload(imageUrl)),
  }
})

onShareTimeline(() => {
  const valid = VALID_ID_RE.test(topicId) && VALID_ID_RE.test(entryId) && VALID_ID_RE.test(commentId)
  const share = buildCommentSharePayload(shareCard.imageUrl.value)
  const prefix = valid
    ? `topicId=${encodeURIComponent(topicId)}&entryId=${encodeURIComponent(entryId)}&commentId=${encodeURIComponent(commentId)}&`
    : ''
  return {
    title: share.title,
    query: `${prefix}${buildShareQuery('telegraph_comment')}`,
    ...(shareCard.imageUrl.value ? { imageUrl: shareCard.imageUrl.value } : {}),
  }
})
</script>

<style scoped>
.comment-share-canvas {
  position: fixed;
  left: -9999rpx;
  top: 0;
  pointer-events: none;
}

.comment-page-offset {
  padding-top: var(--fi-brand-nav-height);
  min-height: 100vh;
  box-sizing: border-box;
  background: var(--fi-color-page);
}

.comment-page {
  padding: var(--fi-space-20) var(--fi-space-20) var(--fi-space-40);
}

.comment-page--with-composer {
  padding-bottom: calc(156rpx + env(safe-area-inset-bottom));
}

.comment-page__state {
  padding: var(--fi-space-40) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-20);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  text-align: center;
}

.comment-page__error {
  color: var(--fi-color-primary);
}

.comment-page__retry {
  margin: 0;
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-bg);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
}

.comment-page__retry::after {
  border: 0;
}
</style>
