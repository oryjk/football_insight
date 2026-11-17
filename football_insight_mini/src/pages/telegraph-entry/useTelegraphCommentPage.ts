import { computed, getCurrentScope, onScopeDispose, ref } from 'vue'
import {
  createTelegraphEntryComment,
  deleteTelegraphEntryComment,
  getTelegraphCommentThread,
  getTelegraphEntryComments,
  getTopicDetail,
  setTelegraphCommentLike,
} from '../../api/dailyTopics'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import type {
  DailyTopicEntry,
  DailyTopicItem,
  TelegraphComment,
  TelegraphCommentThreadPage,
} from '../../types/dailyTopic'
import { extractApiErrorCode, extractApiErrorMessage, isNotFoundError, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphComment,
  mapTelegraphCommentThreadPage,
  mapTopicDetail,
  telegraphCommentUrl,
  telegraphEntryUrl,
} from '../../utils/dailyTopics'
import { createLatestGuard, type LatestToken } from '../../utils/latestGuard'
import { isProfileComplete } from '../../utils/profileGate'
import {
  buildThreadFromLegacyComments,
  defaultNavigateBack,
  defaultRedirectToLogin,
  defaultToast,
  ensureTelegraphAuthAndProfile,
  isRouteMissing404,
  mergeTelegraphComments,
  patchCommentNode,
  runSaveProfileAndContinue,
  runToggleCommentLike,
  syncTopicEntryCommentsCount,
  VALID_TELEGRAPH_ID_RE,
  type EntryMiniReviewPort,
  type EntrySessionPort,
} from './helpers'
import { navigateInTelegraphStack } from './navigation'

export interface TelegraphCommentPageApi {
  getTopicDetail: typeof getTopicDetail
  getCommentThread: typeof getTelegraphCommentThread
  getComments: typeof getTelegraphEntryComments
  createComment: typeof createTelegraphEntryComment
  deleteComment: typeof deleteTelegraphEntryComment
  setCommentLike: typeof setTelegraphCommentLike
}

export interface TelegraphCommentPageUi {
  redirectToLogin: (returnUrl: string) => void
  toast: (title: string, icon?: 'none' | 'success') => void
  openComment: (url: string) => void
  openEntry: (url: string) => void
  navigateBack: (fallbackEntryUrl: string) => void
}

export interface UseTelegraphCommentPageOptions {
  topicId: () => string
  entryId: () => string
  commentId: () => string
  teamId?: number
  session?: EntrySessionPort
  miniReview?: EntryMiniReviewPort
  api?: Partial<TelegraphCommentPageApi>
  ui?: Partial<TelegraphCommentPageUi>
}

export function useTelegraphCommentPage(options: UseTelegraphCommentPageOptions) {
  const session = options.session ?? useAppSession()
  const miniReview = options.miniReview ?? useMiniReviewStatus()
  const teamId = options.teamId ?? DAILY_TOPICS_TEAM_ID
  const api: TelegraphCommentPageApi = {
    getTopicDetail,
    getCommentThread: getTelegraphCommentThread,
    getComments: getTelegraphEntryComments,
    createComment: createTelegraphEntryComment,
    deleteComment: deleteTelegraphEntryComment,
    setCommentLike: setTelegraphCommentLike,
    ...options.api,
  }
  const ui: TelegraphCommentPageUi = {
    redirectToLogin: defaultRedirectToLogin,
    toast: defaultToast,
    openComment: navigateInTelegraphStack,
    openEntry: navigateInTelegraphStack,
    navigateBack: defaultNavigateBack,
    ...options.ui,
  }

  const topic = ref<DailyTopicItem | null>(null)
  const ancestors = ref<TelegraphComment[]>([])
  const focalComment = ref<TelegraphComment | null>(null)
  const replies = ref<TelegraphComment[]>([])
  const repliesTotalCount = ref(0)
  const repliesNextCursor = ref<string | null>(null)
  const loading = ref(false)
  const errorMessage = ref('')
  const repliesLoading = ref(false)
  const loadingMoreReplies = ref(false)
  const repliesError = ref('')
  const commentsDisabled = ref(false)
  const pendingCommentLikeIds = ref<string[]>([])
  const draftContent = ref('')
  const inputFocusTrigger = ref(0)
  const submittingReply = ref(false)
  const deletingCommentId = ref('')
  const profileSheetVisible = ref(false)
  const savingProfile = ref(false)

  let disposed = false
  let usingLegacyFallback = false
  const loadGuard = createLatestGuard(() => [
    options.topicId(),
    options.entryId(),
    options.commentId(),
    session.revision.value,
  ])
  const threadGuard = createLatestGuard(() => [
    options.topicId(),
    options.entryId(),
    options.commentId(),
    session.revision.value,
  ])
  let activeThreadToken: LatestToken | null = null

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      loadGuard.invalidate()
      threadGuard.invalidate()
      activeThreadToken = null
    })
  }

  const entry = computed<DailyTopicEntry | null>(() => {
    const targetId = options.entryId()
    return topic.value?.entries.find(item => item.id === targetId) ?? null
  })
  const commentsVisible = computed(() =>
    !miniReview.shouldRestrictFeatures.value
    && Boolean(topic.value?.commentsEnabled)
    && !commentsDisabled.value,
  )
  const isLoggedIn = computed(() => session.hasAccessToken.value)
  const profileComplete = computed(() => isProfileComplete(session.currentUser.value))
  const repliesHasMore = computed(() => Boolean(repliesNextCursor.value))

  function currentCommentUrl(): string {
    return telegraphCommentUrl(options.topicId(), options.entryId(), options.commentId())
  }

  function currentEntryUrl(): string {
    return telegraphEntryUrl(options.topicId(), options.entryId())
  }

  function syncEntryCommentsCount(nextCount: number): void {
    syncTopicEntryCommentsCount(topic, options.entryId(), nextCount)
  }

  function resetThreadState(nextErrorMessage: string, disabled = false): void {
    if (disabled) {
      commentsDisabled.value = true
    }
    ancestors.value = []
    focalComment.value = null
    replies.value = []
    repliesTotalCount.value = 0
    repliesNextCursor.value = null
    errorMessage.value = nextErrorMessage
  }

  async function fetchThread(reset: boolean): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const currentCommentId = options.commentId()
    if (!currentTopicId || !currentEntryId || !currentCommentId || !commentsVisible.value) {
      return
    }
    if (!reset && (usingLegacyFallback || repliesLoading.value || loadingMoreReplies.value || !repliesNextCursor.value)) {
      return
    }

    const token = reset ? (activeThreadToken = threadGuard.begin()) : (activeThreadToken ?? threadGuard.begin())
    const cursor = reset ? null : repliesNextCursor.value
    if (reset) {
      repliesLoading.value = true
      repliesError.value = ''
    } else {
      loadingMoreReplies.value = true
    }

    try {
      let page: TelegraphCommentThreadPage
      try {
        const raw = await api.getCommentThread(
          teamId,
          currentTopicId,
          currentEntryId,
          currentCommentId,
          cursor,
          20,
        )
        if (disposed || !token.isCurrent()) {
          return
        }
        usingLegacyFallback = false
        page = mapTelegraphCommentThreadPage(raw)
      } catch (error) {
        if (disposed || !token.isCurrent()) {
          return
        }
        if (reset && isRouteMissing404(error)) {
          const fallback = await buildThreadFromLegacyComments({
            teamId,
            topicId: currentTopicId,
            entryId: currentEntryId,
            commentId: currentCommentId,
            getComments: api.getComments,
          })
          if (disposed || !token.isCurrent()) {
            return
          }
          if (fallback === 'disabled') {
            resetThreadState('评论功能已关闭', true)
            return
          }
          if (fallback === 'not_found') {
            resetThreadState('该评论不存在或已被删除')
            return
          }
          usingLegacyFallback = true
          page = fallback
        } else {
          throw error
        }
      }

      if (!page.commentsEnabled) {
        resetThreadState('评论功能已关闭', true)
        return
      }

      if (!reset && page.nextCursor && page.nextCursor === cursor) {
        repliesNextCursor.value = null
        return
      }

      ancestors.value = page.ancestors
      focalComment.value = page.comment
      repliesTotalCount.value = page.repliesTotalCount
      replies.value = reset ? page.items : mergeTelegraphComments(replies.value, page.items)
      repliesNextCursor.value = page.hasMore ? page.nextCursor : null
      syncEntryCommentsCount(page.entryCommentsCount)
    } catch (error) {
      if (disposed || !token.isCurrent()) {
        return
      }
      const code = extractApiErrorCode(error)
      if (code === 'COMMENTS_DISABLED') {
        resetThreadState('评论功能已关闭', true)
        return
      }
      if (code === 'COMMENT_NOT_FOUND' || code === 'NOT_FOUND') {
        resetThreadState('该评论不存在或已被删除')
        return
      }
      const msg = extractApiErrorMessage(error, '评论详情加载失败，请稍后重试')
      if (reset && !focalComment.value) {
        errorMessage.value = msg
      } else if (reset) {
        repliesError.value = msg
      } else {
        ui.toast(msg)
      }
    } finally {
      if (!disposed && token.owns()) {
        repliesLoading.value = false
        loadingMoreReplies.value = false
      }
    }
  }

  async function loadInitial(focusOnLoad = false): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const currentCommentId = options.commentId()
    const token = loadGuard.begin()
    if (
      !VALID_TELEGRAPH_ID_RE.test(currentTopicId)
      || !VALID_TELEGRAPH_ID_RE.test(currentEntryId)
      || !VALID_TELEGRAPH_ID_RE.test(currentCommentId)
    ) {
      topic.value = null
      focalComment.value = null
      errorMessage.value = '该评论不存在或已被删除'
      return
    }

    loading.value = true
    errorMessage.value = ''
    commentsDisabled.value = false
    try {
      const [rawTopic] = await Promise.all([
        api.getTopicDetail(teamId, currentTopicId),
        miniReview.ensureMiniReviewReady?.(),
      ])
      const mapped = mapTopicDetail(rawTopic)
      if (disposed || !token.isCurrent()) {
        return
      }
      if (!mapped.entries.some(item => item.id === currentEntryId)) {
        topic.value = null
        focalComment.value = null
        errorMessage.value = '电报条目不存在或已下线'
        return
      }
      topic.value = mapped
      if (miniReview.shouldRestrictFeatures.value || !mapped.commentsEnabled) {
        commentsDisabled.value = true
        focalComment.value = null
        errorMessage.value = '评论功能已关闭'
        return
      }

      await fetchThread(true)
      if (disposed || !token.isCurrent()) {
        return
      }
      if (
        focusOnLoad
        && focalComment.value
        && commentsVisible.value
        && session.hasAccessToken.value
        && isProfileComplete(session.currentUser.value)
      ) {
        inputFocusTrigger.value += 1
      }
    } catch (error) {
      if (disposed || !token.isCurrent()) {
        return
      }
      errorMessage.value = isNotFoundError(error)
        ? '电报条目不存在或已下线'
        : extractApiErrorMessage(error, '评论详情加载失败，请稍后重试')
    } finally {
      if (!disposed && token.owns()) {
        loading.value = false
      }
    }
  }

  async function refreshAll(): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const currentCommentId = options.commentId()
    if (
      !VALID_TELEGRAPH_ID_RE.test(currentTopicId)
      || !VALID_TELEGRAPH_ID_RE.test(currentEntryId)
      || !VALID_TELEGRAPH_ID_RE.test(currentCommentId)
      || loading.value
    ) {
      return
    }
    const token = loadGuard.begin()
    try {
      const [rawTopic] = await Promise.all([
        api.getTopicDetail(teamId, currentTopicId),
        miniReview.ensureMiniReviewReady?.(),
      ])
      const mapped = mapTopicDetail(rawTopic)
      if (disposed || !token.isCurrent()) {
        return
      }
      if (!mapped.entries.some(item => item.id === currentEntryId)) {
        return
      }
      topic.value = mapped
      if (miniReview.shouldRestrictFeatures.value || !mapped.commentsEnabled) {
        commentsDisabled.value = true
        focalComment.value = null
        errorMessage.value = '评论功能已关闭'
        return
      }
      await fetchThread(true)
    } catch {
      // Silent refresh keeps existing UI visible.
    }
  }

  function requestLogin(): void {
    if (options.topicId() && options.entryId() && options.commentId()) {
      ui.redirectToLogin(currentCommentUrl())
    }
  }

  function ensureAuthAndProfile(): Promise<boolean> {
    return ensureTelegraphAuthAndProfile({
      session,
      isDisposed: () => disposed,
      profileSheetVisible,
      requestLogin,
    })
  }

  async function openProfileSetup(): Promise<void> {
    await ensureAuthAndProfile()
  }

  async function focusReplyComposer(): Promise<void> {
    if (!commentsVisible.value || !focalComment.value) {
      return
    }
    if (!(await ensureAuthAndProfile())) {
      return
    }
    inputFocusTrigger.value += 1
  }

  function applyLikeToVisibleNodes(commentId: string, likesCount: number, likedByMe: boolean): void {
    const updater = (node: TelegraphComment) => ({ ...node, likesCount, likedByMe })
    ancestors.value = ancestors.value.map(node => patchCommentNode(node, commentId, updater))
    if (focalComment.value) {
      focalComment.value = patchCommentNode(focalComment.value, commentId, updater)
    }
    replies.value = replies.value.map(node => patchCommentNode(node, commentId, updater))
  }

  function toggleCommentLike(comment: TelegraphComment): Promise<void> {
    return runToggleCommentLike({
      comment,
      teamId,
      topicId: options.topicId(),
      returnUrl: currentCommentUrl(),
      session,
      pendingCommentLikeIds,
      isDisposed: () => disposed,
      setCommentLike: api.setCommentLike,
      applyLikeSummary: applyLikeToVisibleNodes,
      redirectToLogin: ui.redirectToLogin,
      toast: ui.toast,
    })
  }

  async function submitReply(): Promise<void> {
    if (submittingReply.value || !entry.value || !focalComment.value || !commentsVisible.value) {
      return
    }
    if (!(await ensureAuthAndProfile())) {
      return
    }

    const content = draftContent.value.trim()
    if (!content) {
      ui.toast('请输入回复内容')
      return
    }
    if (Array.from(content).length > 200) {
      ui.toast('评论内容最多 200 字')
      return
    }

    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const parentCommentId = focalComment.value.id
    const startRevision = session.revision.value
    submittingReply.value = true
    try {
      const response = await api.createComment(teamId, currentTopicId, currentEntryId, {
        content,
        reply_to_comment_id: parentCommentId,
      })
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const created = mapTelegraphComment(response.comment)
      if (created) {
        replies.value = [created, ...replies.value.filter(item => item.id !== created.id)]
      }
      if (focalComment.value && focalComment.value.id === parentCommentId) {
        focalComment.value = {
          ...focalComment.value,
          repliesCount: focalComment.value.repliesCount + 1,
        }
      }
      repliesTotalCount.value += 1
      syncEntryCommentsCount(response.comments_count)
      draftContent.value = ''
      ui.toast('回复已发布', 'success')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentCommentUrl())
        return
      }
      const code = extractApiErrorCode(error)
      if (code === 'PROFILE_INCOMPLETE') {
        profileSheetVisible.value = true
        ui.toast('请先上传头像并设置昵称')
        return
      }
      if (code === 'COMMENTS_DISABLED') {
        commentsDisabled.value = true
        focalComment.value = null
        replies.value = []
        errorMessage.value = '评论功能已关闭'
        ui.toast('评论功能已关闭')
        return
      }
      ui.toast(extractApiErrorMessage(error, '发表回复失败，请稍后重试'))
    } finally {
      if (!disposed && session.revision.value === startRevision) {
        submittingReply.value = false
      }
    }
  }

  function saveProfileAndContinue(payload: { display_name: string; avatar_data_url: string | null }): Promise<void> {
    return runSaveProfileAndContinue({
      payload,
      session,
      savingProfile,
      profileSheetVisible,
      isDisposed: () => disposed,
      canContinueWithDraft: () => Boolean(commentsVisible.value && draftContent.value.trim()),
      onContinueWithDraft: submitReply,
      canFocusAfterSave: () => Boolean(commentsVisible.value && focalComment.value),
      inputFocusTrigger,
      toast: ui.toast,
    })
  }

  async function deleteOwnComment(comment: TelegraphComment): Promise<void> {
    if (!comment.isMine || deletingCommentId.value) {
      return
    }
    const currentTopicId = options.topicId()
    const startRevision = session.revision.value
    deletingCommentId.value = comment.id
    try {
      const response = await api.deleteComment(teamId, currentTopicId, comment.entryId, comment.id)
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      syncEntryCommentsCount(response.comments_count)

      if (focalComment.value?.id === comment.id) {
        ui.toast('评论已删除')
        ui.navigateBack(currentEntryUrl())
        return
      }

      const existedInReplies = replies.value.some(item => item.id === comment.id)
      if (existedInReplies) {
        replies.value = replies.value.filter(item => item.id !== comment.id)
        repliesTotalCount.value = Math.max(0, repliesTotalCount.value - 1)
        if (focalComment.value) {
          focalComment.value = {
            ...focalComment.value,
            repliesCount: Math.max(0, focalComment.value.repliesCount - 1),
          }
        }
      }
      if (ancestors.value.some(item => item.id === comment.id)) {
        await refreshAll()
      }
      ui.toast('评论已删除')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentCommentUrl())
        return
      }
      ui.toast(extractApiErrorMessage(error, '删除评论失败，请稍后重试'))
    } finally {
      if (!disposed && session.revision.value === startRevision) {
        deletingCommentId.value = ''
      }
    }
  }

  function openEntry(): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_TELEGRAPH_ID_RE.test(currentTopicId) || !VALID_TELEGRAPH_ID_RE.test(currentEntryId)) {
      return
    }
    ui.openEntry(telegraphEntryUrl(currentTopicId, currentEntryId))
  }

  function openComment(comment: TelegraphComment, focusReply = false): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_TELEGRAPH_ID_RE.test(currentTopicId) || !VALID_TELEGRAPH_ID_RE.test(currentEntryId) || !comment.id) {
      return
    }
    if (comment.id === options.commentId()) {
      if (focusReply) {
        void focusReplyComposer()
      }
      return
    }
    ui.openComment(telegraphCommentUrl(currentTopicId, currentEntryId, comment.id, focusReply))
  }

  return {
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
    refreshAll,
    reloadReplies: () => fetchThread(true),
    loadMoreReplies: () => fetchThread(false),
    requestLogin,
    openProfileSetup,
    focusReplyComposer,
    toggleCommentLike,
    submitReply,
    saveProfileAndContinue,
    deleteOwnComment,
    openEntry,
    openComment,
  }
}
