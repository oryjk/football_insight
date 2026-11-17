<template>
  <page-meta :page-style="pageStyle" />
  <FiSecondaryNav :share-disabled="!entry" @back="goBack" @share="copyShareLink" />
  <view class="entry-page-offset">
    <view class="entry-page" :class="{ 'entry-page--with-composer': Boolean(entry && commentsVisible) }">
      <view v-if="loading && !entry" class="entry-page__state">
        <text>正在加载电报详情…</text>
      </view>

      <view v-else-if="errorMessage && !entry" class="entry-page__state">
        <text class="entry-page__error">{{ errorMessage }}</text>
        <button class="entry-page__retry" @tap="loadInitial(false)">重新加载</button>
      </view>

      <template v-else-if="entry">
        <TelegraphEntryTimeline
          :entry="entry"
          :day-label="dayLabel"
          :comments-visible="commentsVisible"
          :reaction-pending="reactionPending"
          :comments="comments"
          :comments-loading="commentsLoading"
          :loading-more="loadingMoreComments"
          :has-more="commentsHasMore"
          :comments-error="commentsError"
          :pending-like-ids="pendingCommentLikeIds"
          :deleting-comment-id="deletingCommentId"
          @copy-source="copyEntrySource"
          @share="copyShareLink"
          @like-entry="page.likeEntry"
          @dislike-entry="page.dislikeEntry"
          @focus-comment="page.startRootComment"
          @retry-comments="page.reloadComments"
          @load-more="page.loadMoreComments"
          @open-comment="page.openComment"
          @like-comment="page.toggleCommentLike"
          @delete-comment="page.deleteOwnComment"
        />
      </template>
    </view>
  </view>

  <TelegraphEntryComposer
    :visible="Boolean(entry && commentsVisible)"
    :logged-in="isLoggedIn"
    :profile-complete="profileComplete"
    :draft="draftContent"
    :reply-target="replyTarget"
    :submitting="submittingComment"
    :focus-trigger="inputFocusTrigger"
    @update:draft="draftContent = $event"
    @focus-change="composerFocused = $event"
    @login="page.requestLogin"
    @setup-profile="page.openProfileSetup"
    @cancel-reply="page.clearReplyTarget"
    @submit="page.submitComment"
  />

  <FiProfileSetupSheet
    :visible="profileSheetVisible"
    :submitting="savingProfile"
    :initial-nickname="editableNicknameInitialValue(session.currentUser.value)"
    :initial-avatar-url="session.currentUser.value?.avatar_url ?? null"
    @update:visible="profileSheetVisible = $event"
    @submit="page.saveProfileAndContinue"
  />
  <canvas :canvas-id="TELEGRAPH_ENTRY_SHARE_CANVAS_ID" class="entry-share-canvas" :style="shareCanvasStyle" />
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { onHide, onLoad, onPullDownRefresh, onReachBottom, onShareAppMessage, onShareTimeline, onShow } from '@dcloudio/uni-app'
import FiProfileSetupSheet from '../../components/FiProfileSetupSheet.vue'
import FiSecondaryNav from '../../components/FiSecondaryNav.vue'
import { useNavigationLayout } from '../../composables/useNavigationLayout'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import { isTopicSourceUrl, roundupDayLabel } from '../../utils/dailyTopics'
import { editableNicknameInitialValue } from '../../utils/profileGate'
import { copyPageShareLink, goBackFromSecondaryPage } from '../../utils/secondaryNavigation'
import {
  buildTelegraphEntrySharePayload,
  buildTelegraphEntryTimelinePayload,
  TELEGRAPH_SHARE_CARD,
} from '../../utils/telegraphShareCard'
import TelegraphEntryComposer from './components/TelegraphEntryComposer.vue'
import TelegraphEntryTimeline from './components/TelegraphEntryTimeline.vue'
import { useTelegraphEntryPage } from './useTelegraphEntryPage'
import { TELEGRAPH_ENTRY_SHARE_CANVAS_ID, useTelegraphEntryShareCard } from './useTelegraphEntryShareCard'

const { navigationPageStyle } = useNavigationLayout()
const session = useAppSession()
const miniReview = useMiniReviewStatus()
const shareCard = useTelegraphEntryShareCard()
// 分享图按实际像素绘制，与页面布局的 rpx 无关。
const shareCanvasStyle = { width: `${TELEGRAPH_SHARE_CARD.width}px`, height: `${TELEGRAPH_SHARE_CARD.height}px` }

let topicId = ''
let entryId = ''
let lastSessionRevision = session.revision.value
let hasLeftPage = false

const page = useTelegraphEntryPage({
  topicId: () => topicId,
  entryId: () => entryId,
  session,
  miniReview,
})

const {
  topic,
  entry,
  loading,
  errorMessage,
  commentsVisible,
  isLoggedIn,
  profileComplete,
  reactionPending,
  comments,
  commentsLoading,
  loadingMoreComments,
  commentsError,
  commentsHasMore,
  pendingCommentLikeIds,
  draftContent,
  replyTarget,
  inputFocusTrigger,
  submittingComment,
  deletingCommentId,
  profileSheetVisible,
  savingProfile,
  loadInitial,
} = page

const composerFocused = ref(false)
const dayLabel = computed(() => (topic.value ? roundupDayLabel(topic.value.publishedAt) : ''))
const pageStyle = computed(() =>
  profileSheetVisible.value || composerFocused.value
    ? `${navigationPageStyle.value};overflow:hidden;`
    : navigationPageStyle.value,
)

function copyEntrySource(url: string): void {
  if (!isTopicSourceUrl(url) || !entry.value?.sources.some(source => source.url === url)) {
    return
  }
  uni.setClipboardData({
    data: url,
    success: () => uni.showToast({ title: '原文链接已复制', icon: 'none' }),
  })
}

function goBack(): void {
  goBackFromSecondaryPage()
}

function copyShareLink(): void {
  copyPageShareLink(
    buildTelegraphEntrySharePayload({ topicId, entryId, entry: entry.value }).path,
  )
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
  if (topicId && entryId && topic.value && (hasLeftPage || session.revision.value !== lastSessionRevision)) {
    hasLeftPage = false
    lastSessionRevision = session.revision.value
    void page.refreshAll()
  }
})

onPullDownRefresh(() => {
  void page.refreshAll().finally(() => uni.stopPullDownRefresh())
})

onReachBottom(() => {
  if (commentsVisible.value && commentsHasMore.value) {
    page.loadMoreComments()
  }
})

onShareAppMessage(() => {
  const currentEntry = entry.value
  const options = { publishedAt: topic.value?.publishedAt, commentsVisible: commentsVisible.value }
  const fallback = buildTelegraphEntrySharePayload({
    topicId,
    entryId,
    entry: currentEntry,
    imageUrl: shareCard.imageUrl.value,
  })
  if (!currentEntry || shareCard.imageUrl.value) return fallback
  return {
    ...fallback,
    promise: shareCard.prepare(currentEntry, options).then(imageUrl =>
      buildTelegraphEntrySharePayload({
        topicId,
        entryId,
        entry: currentEntry,
        imageUrl,
      }),
    ),
  }
})

onShareTimeline(() => buildTelegraphEntryTimelinePayload({
  topicId,
  entryId,
  entry: entry.value,
  imageUrl: shareCard.imageUrl.value,
}))
</script>

<style scoped>
.entry-share-canvas {
  position: fixed;
  left: -9999rpx;
  top: 0;
  pointer-events: none;
}

.entry-page-offset {
  padding-top: var(--fi-brand-nav-height);
  min-height: 100vh;
  box-sizing: border-box;
  background: var(--fi-color-page);
}

.entry-page {
  padding: var(--fi-space-20) var(--fi-space-20) var(--fi-space-40);
}

.entry-page--with-composer {
  padding-bottom: calc(156rpx + env(safe-area-inset-bottom));
}

.entry-page__state {
  padding: var(--fi-space-40) 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--fi-space-20);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  text-align: center;
}

.entry-page__error {
  color: var(--fi-color-primary);
}

.entry-page__retry {
  margin: 0;
  padding: var(--fi-space-12) var(--fi-space-28);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-bg);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-26);
}

.entry-page__retry::after {
  border: 0;
}
</style>
