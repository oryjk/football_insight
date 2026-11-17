import { computed, getCurrentScope, onScopeDispose, ref } from 'vue'
import {
  createTelegraphEntryComment,
  deleteTelegraphEntryComment,
  getTelegraphEntryComments,
  getTopicDetail,
  setTelegraphCommentLike,
  setTelegraphEntryDislike,
  setTelegraphEntryLike,
} from '../../api/dailyTopics'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import type { DailyTopicEntry, DailyTopicItem, TelegraphComment } from '../../types/dailyTopic'
import { extractApiErrorCode, extractApiErrorMessage, isNotFoundError, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphComment,
  mapTelegraphCommentsPage,
  mapTelegraphReactionSummary,
  mapTopicDetail,
  telegraphCommentUrl,
  telegraphEntryUrl,
} from '../../utils/dailyTopics'
import { createLatestGuard, type LatestToken } from '../../utils/latestGuard'
import { isProfileComplete } from '../../utils/profileGate'
import {
  defaultRedirectToLogin,
  defaultToast,
  ensureTelegraphAuthAndProfile,
  mergeTelegraphComments,
  runSaveProfileAndContinue,
  runToggleCommentLike,
  syncTopicEntryCommentsCount,
  updateTopicEntry,
  VALID_TELEGRAPH_ID_RE,
  type EntryMiniReviewPort,
  type EntrySessionPort,
} from './helpers'
import { navigateInTelegraphStack } from './navigation'

export type { EntryMiniReviewPort, EntrySessionPort } from './helpers'

export interface TelegraphEntryPageApi {
  getTopicDetail: typeof getTopicDetail
  setLike: typeof setTelegraphEntryLike
  setDislike: typeof setTelegraphEntryDislike
  getComments: typeof getTelegraphEntryComments
  createComment: typeof createTelegraphEntryComment
  deleteComment: typeof deleteTelegraphEntryComment
  setCommentLike: typeof setTelegraphCommentLike
}

export interface TelegraphEntryPageUi {
  redirectToLogin: (returnUrl: string) => void
  toast: (title: string, icon?: 'none' | 'success') => void
  openComment: (url: string) => void
}

export interface UseTelegraphEntryPageOptions {
  topicId: () => string
  entryId: () => string
  teamId?: number
  session?: EntrySessionPort
  miniReview?: EntryMiniReviewPort
  api?: Partial<TelegraphEntryPageApi>
  ui?: Partial<TelegraphEntryPageUi>
}

export function useTelegraphEntryPage(options: UseTelegraphEntryPageOptions) {
  const session = options.session ?? useAppSession()
  const miniReview = options.miniReview ?? useMiniReviewStatus()
  const teamId = options.teamId ?? DAILY_TOPICS_TEAM_ID
  const api: TelegraphEntryPageApi = {
    getTopicDetail,
    setLike: setTelegraphEntryLike,
    setDislike: setTelegraphEntryDislike,
    getComments: getTelegraphEntryComments,
    createComment: createTelegraphEntryComment,
    deleteComment: deleteTelegraphEntryComment,
    setCommentLike: setTelegraphCommentLike,
    ...options.api,
  }
  const ui: TelegraphEntryPageUi = {
    redirectToLogin: defaultRedirectToLogin,
    toast: defaultToast,
    openComment: navigateInTelegraphStack,
    ...options.ui,
  }

  const topic = ref<DailyTopicItem | null>(null)
  const loading = ref(false)
  const errorMessage = ref('')
  const reactionPending = ref(false)
  const comments = ref<TelegraphComment[]>([])
  const commentsNextCursor = ref<string | null>(null)
  const commentsLoading = ref(false)
  const loadingMoreComments = ref(false)
  const commentsError = ref('')
  const pendingCommentLikeIds = ref<string[]>([])
  const draftContent = ref('')
  const replyTarget = ref<TelegraphComment | null>(null)
  const inputFocusTrigger = ref(0)
  const submittingComment = ref(false)
  const deletingCommentId = ref('')
  const profileSheetVisible = ref(false)
  const savingProfile = ref(false)

  let disposed = false
  const loadGuard = createLatestGuard(() => [
    options.topicId(),
    options.entryId(),
    session.revision.value,
  ])
  const commentsGuard = createLatestGuard(() => [
    options.topicId(),
    options.entryId(),
    session.revision.value,
  ])
  let activeCommentsToken: LatestToken | null = null

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      loadGuard.invalidate()
      commentsGuard.invalidate()
      activeCommentsToken = null
    })
  }

  const entry = computed<DailyTopicEntry | null>(() => {
    const targetId = options.entryId()
    return topic.value?.entries.find(item => item.id === targetId) ?? null
  })
  const commentsVisible = computed(() =>
    !miniReview.shouldRestrictFeatures.value && Boolean(topic.value?.commentsEnabled),
  )
  const isLoggedIn = computed(() => session.hasAccessToken.value)
  const profileComplete = computed(() => isProfileComplete(session.currentUser.value))
  const commentsHasMore = computed(() => Boolean(commentsNextCursor.value))

  function currentPageUrl(): string {
    return telegraphEntryUrl(options.topicId(), options.entryId())
  }

  function syncEntryCommentsCount(nextCount: number): void {
    syncTopicEntryCommentsCount(topic, options.entryId(), nextCount)
  }

  async function fetchComments(reset: boolean): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!currentTopicId || !currentEntryId || !commentsVisible.value) {
      return
    }
    if (!reset && (commentsLoading.value || loadingMoreComments.value || !commentsNextCursor.value)) {
      return
    }

    const token = reset ? (activeCommentsToken = commentsGuard.begin()) : (activeCommentsToken ?? commentsGuard.begin())
    const cursor = reset ? null : commentsNextCursor.value
    if (reset) {
      commentsLoading.value = true
      commentsError.value = ''
    } else {
      loadingMoreComments.value = true
    }

    try {
      const raw = await api.getComments(teamId, currentTopicId, currentEntryId, cursor, 20, { rootOnly: true })
      if (disposed || !token.isCurrent()) {
        return
      }
      const page = mapTelegraphCommentsPage(raw)
      if (!page.commentsEnabled && topic.value) {
        topic.value = { ...topic.value, commentsEnabled: false }
        comments.value = []
        commentsNextCursor.value = null
        return
      }
      if (!reset && page.nextCursor && page.nextCursor === cursor) {
        commentsNextCursor.value = null
        return
      }
      // 电报详情页时间线仅展示直接评论本电报的根评论（兼容未升级 root_only 参数的旧后端）。
      const rootItems = page.items.filter(item => item.replyTo === null)
      comments.value = reset ? rootItems : mergeTelegraphComments(comments.value, rootItems)
      commentsNextCursor.value = page.hasMore ? page.nextCursor : null
      syncEntryCommentsCount(page.totalCount)
    } catch (error) {
      if (disposed || !token.isCurrent()) {
        return
      }
      const msg = extractApiErrorMessage(error, '评论加载失败，请稍后重试')
      if (reset) {
        commentsError.value = msg
      } else {
        ui.toast(msg)
      }
    } finally {
      if (!disposed && token.owns()) {
        commentsLoading.value = false
        loadingMoreComments.value = false
      }
    }
  }

  async function loadInitial(focusOnLoad = false): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const token = loadGuard.begin()
    if (!VALID_TELEGRAPH_ID_RE.test(currentTopicId) || !VALID_TELEGRAPH_ID_RE.test(currentEntryId)) {
      topic.value = null
      errorMessage.value = '电报条目不存在或已下线'
      return
    }

    loading.value = true
    errorMessage.value = ''
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
        errorMessage.value = '电报条目不存在或已下线'
        return
      }
      topic.value = mapped
      loading.value = false
      if (!miniReview.shouldRestrictFeatures.value && mapped.commentsEnabled) {
        await fetchComments(true)
        if (disposed || !token.isCurrent()) {
          return
        }
        if (focusOnLoad && session.hasAccessToken.value && isProfileComplete(session.currentUser.value)) {
          inputFocusTrigger.value += 1
        }
      }
    } catch (error) {
      if (disposed || !token.isCurrent()) {
        return
      }
      errorMessage.value = isNotFoundError(error)
        ? '电报条目不存在或已下线'
        : extractApiErrorMessage(error, '电报加载失败，请稍后重试')
    } finally {
      if (!disposed && token.owns()) {
        loading.value = false
      }
    }
  }

  async function refreshAll(): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_TELEGRAPH_ID_RE.test(currentTopicId) || !VALID_TELEGRAPH_ID_RE.test(currentEntryId) || loading.value) {
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
      if (!miniReview.shouldRestrictFeatures.value && mapped.commentsEnabled) {
        await fetchComments(true)
      }
    } catch {
      // Pull-to-refresh or session sync failure keeps visible content intact.
    }
  }

  async function toggleEntryReaction(kind: 'like' | 'dislike'): Promise<void> {
    if (reactionPending.value || !entry.value) {
      return
    }
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!currentTopicId || !currentEntryId) {
      return
    }
    if (!session.hasAccessToken.value) {
      ui.redirectToLogin(currentPageUrl())
      return
    }

    const startRevision = session.revision.value
    reactionPending.value = true
    try {
      const nextActive = kind === 'like' ? !entry.value.likedByMe : !entry.value.dislikedByMe
      const raw = kind === 'like'
        ? await api.setLike(teamId, currentTopicId, currentEntryId, nextActive)
        : await api.setDislike(teamId, currentTopicId, currentEntryId, nextActive)
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const summary = mapTelegraphReactionSummary(raw)
      updateTopicEntry(topic, currentEntryId, item => ({
        ...item,
        likesCount: summary.likesCount,
        dislikesCount: summary.dislikesCount,
        likedByMe: summary.likedByMe,
        dislikedByMe: summary.dislikedByMe,
      }))
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentPageUrl())
        return
      }
      ui.toast(extractApiErrorMessage(error, kind === 'like' ? '点赞失败，请稍后重试' : '点踩失败，请稍后重试'))
    } finally {
      if (!disposed && session.revision.value === startRevision) {
        reactionPending.value = false
      }
    }
  }

  function likeEntry(): Promise<void> {
    return toggleEntryReaction('like')
  }

  function dislikeEntry(): Promise<void> {
    return toggleEntryReaction('dislike')
  }

  function toggleCommentLike(comment: TelegraphComment): Promise<void> {
    return runToggleCommentLike({
      comment,
      teamId,
      topicId: options.topicId(),
      returnUrl: currentPageUrl(),
      session,
      pendingCommentLikeIds,
      isDisposed: () => disposed,
      setCommentLike: api.setCommentLike,
      applyLikeSummary: (commentId, likesCount, likedByMe) => {
        comments.value = comments.value.map(item => (
          item.id === commentId
            ? { ...item, likesCount, likedByMe }
            : item
        ))
      },
      redirectToLogin: ui.redirectToLogin,
      toast: ui.toast,
    })
  }

  function requestLogin(): void {
    if (options.topicId() && options.entryId()) {
      ui.redirectToLogin(currentPageUrl())
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

  function openComment(comment: TelegraphComment, focusReply = false): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_TELEGRAPH_ID_RE.test(currentTopicId) || !VALID_TELEGRAPH_ID_RE.test(currentEntryId) || !comment.id) {
      return
    }
    ui.openComment(telegraphCommentUrl(currentTopicId, currentEntryId, comment.id, focusReply))
  }

  async function startReply(comment: TelegraphComment): Promise<void> {
    if (!commentsVisible.value) {
      return
    }
    openComment(comment, true)
  }

  async function startRootComment(): Promise<void> {
    if (!commentsVisible.value) {
      return
    }
    if (!(await ensureAuthAndProfile())) {
      return
    }
    replyTarget.value = null
    inputFocusTrigger.value += 1
  }

  function clearReplyTarget(): void {
    replyTarget.value = null
  }

  async function submitComment(): Promise<void> {
    if (submittingComment.value || !entry.value || !commentsVisible.value) {
      return
    }
    if (!(await ensureAuthAndProfile())) {
      return
    }

    const content = draftContent.value.trim()
    if (!content) {
      ui.toast('请输入评论内容')
      return
    }
    if (Array.from(content).length > 200) {
      ui.toast('评论内容最多 200 字')
      return
    }

    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const startRevision = session.revision.value
    const repliedParentId = replyTarget.value?.id ?? null
    submittingComment.value = true
    try {
      const response = await api.createComment(teamId, currentTopicId, currentEntryId, {
        content,
        reply_to_comment_id: repliedParentId,
      })
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const created = mapTelegraphComment(response.comment)
      const nextList = created
        ? [created, ...comments.value.filter(existing => existing.id !== created.id)]
        : comments.value
      comments.value = repliedParentId
        ? nextList.map(item => (item.id === repliedParentId ? { ...item, repliesCount: item.repliesCount + 1 } : item))
        : nextList
      syncEntryCommentsCount(response.comments_count)
      draftContent.value = ''
      replyTarget.value = null
      ui.toast('评论已发布', 'success')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentPageUrl())
        return
      }
      const code = extractApiErrorCode(error)
      if (code === 'PROFILE_INCOMPLETE') {
        profileSheetVisible.value = true
        ui.toast('请先上传头像并设置昵称')
        return
      }
      if (code === 'COMMENTS_DISABLED') {
        if (topic.value) {
          topic.value = { ...topic.value, commentsEnabled: false }
        }
        comments.value = []
        commentsNextCursor.value = null
        ui.toast('评论功能已关闭')
        return
      }
      ui.toast(extractApiErrorMessage(error, '发表评论失败，请稍后重试'))
    } finally {
      if (!disposed && session.revision.value === startRevision) {
        submittingComment.value = false
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
      onContinueWithDraft: submitComment,
      canFocusAfterSave: () => commentsVisible.value,
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
      const parentId = comment.replyTo?.commentId ?? null
      comments.value = comments.value
        .filter(existing => existing.id !== comment.id)
        .map(item => (parentId && item.id === parentId ? { ...item, repliesCount: Math.max(0, item.repliesCount - 1) } : item))
      if (replyTarget.value?.id === comment.id) {
        replyTarget.value = null
      }
      syncEntryCommentsCount(response.comments_count)
      ui.toast('评论已删除')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentPageUrl())
        return
      }
      ui.toast(extractApiErrorMessage(error, '删除评论失败，请稍后重试'))
    } finally {
      if (!disposed && session.revision.value === startRevision) {
        deletingCommentId.value = ''
      }
    }
  }

  return {
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
    refreshAll,
    reloadComments: () => fetchComments(true),
    loadMoreComments: () => fetchComments(false),
    likeEntry,
    dislikeEntry,
    toggleCommentLike,
    requestLogin,
    openProfileSetup,
    openComment,
    startReply,
    startRootComment,
    clearReplyTarget,
    submitComment,
    saveProfileAndContinue,
    deleteOwnComment,
  }
}
