import { computed, getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'
import {
  createTelegraphEntryComment,
  deleteTelegraphEntryComment,
  getTelegraphEntryComments,
  setTelegraphEntryDislike,
  setTelegraphEntryLike,
} from '../../api/dailyTopics'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import type { CurrentUser } from '../../types/auth'
import type { DailyTopicEntry, DailyTopicItem, TelegraphComment } from '../../types/dailyTopic'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphComment,
  mapTelegraphCommentsPage,
  mapTelegraphReactionSummary,
  topicDetailUrl,
} from '../../utils/dailyTopics'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { extractApiErrorCode, isProfileComplete } from '../../utils/profileGate'

export interface TelegraphSessionPort {
  currentUser: Ref<CurrentUser | null>
  hasAccessToken: Ref<boolean>
  revision: Ref<number>
  ensureSessionReady: () => Promise<CurrentUser | null>
  updateProfile: (payload: { display_name: string; avatar_data_url?: string | null }) => Promise<CurrentUser>
  invalidateSession: (revision: number) => void
}

export interface TelegraphMiniReviewPort {
  shouldRestrictFeatures: Ref<boolean>
}

export interface TelegraphInteractionsApi {
  setLike: typeof setTelegraphEntryLike
  setDislike: typeof setTelegraphEntryDislike
  getComments: typeof getTelegraphEntryComments
  createComment: typeof createTelegraphEntryComment
  deleteComment: typeof deleteTelegraphEntryComment
}

export interface TelegraphInteractionsUi {
  redirectToLogin: (returnUrl: string) => void
  toast: (title: string, icon?: 'none' | 'success') => void
}

export interface UseTelegraphInteractionsOptions {
  topicId: () => string
  item: Ref<DailyTopicItem | null>
  teamId?: number
  session?: TelegraphSessionPort
  miniReview?: TelegraphMiniReviewPort
  api?: Partial<TelegraphInteractionsApi>
  ui?: Partial<TelegraphInteractionsUi>
}

function defaultRedirectToLogin(returnUrl: string): void {
  rememberPostLoginRedirect({ type: 'navigateTo', url: returnUrl })
  uni.showToast({ title: '请登录后继续', icon: 'none' })
  // `/pages/user/index` 是 tabBar 页面，必须使用 switchTab。
  uni.switchTab({ url: '/pages/user/index' })
}

function defaultToast(title: string, icon: 'none' | 'success' = 'none'): void {
  uni.showToast({ title, icon })
}

function mergeComments(existing: TelegraphComment[], incoming: TelegraphComment[]): TelegraphComment[] {
  if (!existing.length) {
    return incoming
  }
  const seen = new Set(existing.map(comment => comment.id))
  const merged = [...existing]
  for (const comment of incoming) {
    if (!seen.has(comment.id)) {
      seen.add(comment.id)
      merged.push(comment)
    }
  }
  return merged
}

export function useTelegraphInteractions(options: UseTelegraphInteractionsOptions) {
  const session = options.session ?? useAppSession()
  const miniReview = options.miniReview ?? useMiniReviewStatus()
  const teamId = options.teamId ?? DAILY_TOPICS_TEAM_ID
  const api: TelegraphInteractionsApi = {
    setLike: setTelegraphEntryLike,
    setDislike: setTelegraphEntryDislike,
    getComments: getTelegraphEntryComments,
    createComment: createTelegraphEntryComment,
    deleteComment: deleteTelegraphEntryComment,
    ...options.api,
  }
  const ui: TelegraphInteractionsUi = {
    redirectToLogin: defaultRedirectToLogin,
    toast: defaultToast,
    ...options.ui,
  }

  const pendingReactionIds = ref<string[]>([])
  const commentSheetVisible = ref(false)
  const activeEntryId = ref('')
  const comments = ref<TelegraphComment[]>([])
  const commentsNextCursor = ref<string | null>(null)
  const commentsLoading = ref(false)
  const loadingMoreComments = ref(false)
  const commentsError = ref('')
  const draftContent = ref('')
  const replyTarget = ref<TelegraphComment | null>(null)
  const submittingComment = ref(false)
  const deletingCommentId = ref('')
  const profileSheetVisible = ref(false)
  const savingProfile = ref(false)

  let disposed = false
  let commentsLoadGeneration = 0

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      commentsLoadGeneration += 1
    })
  }

  const commentsVisible = computed(() =>
    !miniReview.shouldRestrictFeatures.value && Boolean(options.item.value?.commentsEnabled),
  )
  const isLoggedIn = computed(() => session.hasAccessToken.value)
  const profileComplete = computed(() => isProfileComplete(session.currentUser.value))
  const activeEntry = computed<DailyTopicEntry | null>(() =>
    options.item.value?.entries.find(entry => entry.id === activeEntryId.value) ?? null,
  )
  const commentsHasMore = computed(() => Boolean(commentsNextCursor.value))

  function updateEntry(entryId: string, updater: (entry: DailyTopicEntry) => DailyTopicEntry, topicCommentsDelta = 0): void {
    const current = options.item.value
    if (!current) {
      return
    }
    const nextEntries = current.entries.map(entry => (entry.id === entryId ? updater(entry) : entry))
    options.item.value = {
      ...current,
      commentsCount: Math.max(0, current.commentsCount + topicCommentsDelta),
      entries: nextEntries,
    }
  }

  function syncEntryCommentsCount(entryId: string, nextCount: number): void {
    const current = options.item.value
    const target = current?.entries.find(entry => entry.id === entryId)
    if (!target) {
      return
    }
    const safeCount = Math.max(0, Math.trunc(nextCount))
    const delta = safeCount - target.commentsCount
    updateEntry(entryId, entry => ({ ...entry, commentsCount: safeCount }), delta)
  }

  async function toggleReaction(entry: DailyTopicEntry, kind: 'like' | 'dislike'): Promise<void> {
    if (pendingReactionIds.value.includes(entry.id)) {
      return
    }
    const currentTopicId = options.topicId()
    if (!currentTopicId) {
      return
    }
    if (!session.hasAccessToken.value) {
      ui.redirectToLogin(topicDetailUrl(currentTopicId))
      return
    }

    const startRevision = session.revision.value
    pendingReactionIds.value = [...pendingReactionIds.value, entry.id]
    try {
      const nextActive = kind === 'like' ? !entry.likedByMe : !entry.dislikedByMe
      const raw = kind === 'like'
        ? await api.setLike(teamId, currentTopicId, entry.id, nextActive)
        : await api.setDislike(teamId, currentTopicId, entry.id, nextActive)
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const summary = mapTelegraphReactionSummary(raw)
      updateEntry(entry.id, target => ({
        ...target,
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
        ui.redirectToLogin(topicDetailUrl(currentTopicId))
        return
      }
      ui.toast(extractApiErrorMessage(error, kind === 'like' ? '点赞失败，请稍后重试' : '点踩失败，请稍后重试'))
    } finally {
      pendingReactionIds.value = pendingReactionIds.value.filter(id => id !== entry.id)
    }
  }

  function likeEntry(entry: DailyTopicEntry): Promise<void> {
    return toggleReaction(entry, 'like')
  }

  function dislikeEntry(entry: DailyTopicEntry): Promise<void> {
    return toggleReaction(entry, 'dislike')
  }

  async function fetchComments(reset: boolean): Promise<void> {
    const entryId = activeEntryId.value
    const currentTopicId = options.topicId()
    if (!entryId || !currentTopicId || !commentsVisible.value) {
      return
    }
    if (!reset && (commentsLoading.value || loadingMoreComments.value || !commentsNextCursor.value)) {
      return
    }

    const generation = reset ? ++commentsLoadGeneration : commentsLoadGeneration
    const cursor = reset ? null : commentsNextCursor.value
    if (reset) {
      commentsLoading.value = true
      commentsError.value = ''
    } else {
      loadingMoreComments.value = true
    }

    try {
      const raw = await api.getComments(teamId, currentTopicId, entryId, cursor)
      if (disposed || generation !== commentsLoadGeneration || activeEntryId.value !== entryId) {
        return
      }
      const page = mapTelegraphCommentsPage(raw)
      if (!page.commentsEnabled && options.item.value) {
        options.item.value = { ...options.item.value, commentsEnabled: false }
        commentSheetVisible.value = false
        return
      }
      if (!reset && page.nextCursor && page.nextCursor === cursor) {
        commentsNextCursor.value = null
        return
      }
      comments.value = reset ? page.items : mergeComments(comments.value, page.items)
      commentsNextCursor.value = page.hasMore ? page.nextCursor : null
      syncEntryCommentsCount(entryId, page.totalCount)
    } catch (error) {
      if (disposed || generation !== commentsLoadGeneration || activeEntryId.value !== entryId) {
        return
      }
      const msg = extractApiErrorMessage(error, '评论加载失败，请稍后重试')
      if (reset) {
        commentsError.value = msg
      } else {
        ui.toast(msg)
      }
    } finally {
      if (!disposed && generation === commentsLoadGeneration) {
        commentsLoading.value = false
        loadingMoreComments.value = false
      }
    }
  }

  function openComments(entry: DailyTopicEntry): void {
    if (!commentsVisible.value) {
      return
    }
    if (activeEntryId.value !== entry.id) {
      draftContent.value = ''
      replyTarget.value = null
    }
    activeEntryId.value = entry.id
    commentSheetVisible.value = true
    comments.value = []
    commentsNextCursor.value = null
    commentsError.value = ''
    void fetchComments(true)
  }

  function reloadComments(): void {
    void fetchComments(true)
  }

  function loadMoreComments(): void {
    void fetchComments(false)
  }

  function closeComments(): void {
    commentSheetVisible.value = false
    replyTarget.value = null
  }

  function requestLoginFromComments(): void {
    const currentTopicId = options.topicId()
    closeComments()
    if (currentTopicId) {
      ui.redirectToLogin(topicDetailUrl(currentTopicId))
    }
  }

  async function requestCommentAuthOrProfile(): Promise<boolean> {
    const currentTopicId = options.topicId()
    if (!currentTopicId) {
      return false
    }
    if (!session.hasAccessToken.value) {
      requestLoginFromComments()
      return false
    }
    if (!session.currentUser.value) {
      try {
        await session.ensureSessionReady()
      } catch {
        // Ignore and let fallback redirect handle stale tokens.
      }
    }
    if (disposed) {
      return false
    }
    if (!session.hasAccessToken.value || !session.currentUser.value) {
      requestLoginFromComments()
      return false
    }
    if (!isProfileComplete(session.currentUser.value)) {
      profileSheetVisible.value = true
      return false
    }
    return true
  }

  async function openProfileSetup(): Promise<void> {
    await requestCommentAuthOrProfile()
  }

  async function setReplyTarget(comment: TelegraphComment): Promise<void> {
    if (!(await requestCommentAuthOrProfile())) {
      return
    }
    replyTarget.value = comment
  }

  function clearReplyTarget(): void {
    replyTarget.value = null
  }

  async function submitComment(): Promise<void> {
    if (submittingComment.value || !activeEntry.value || !commentsVisible.value) {
      return
    }
    if (!(await requestCommentAuthOrProfile())) {
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

    const entryId = activeEntry.value.id
    const currentTopicId = options.topicId()
    const startRevision = session.revision.value
    submittingComment.value = true
    try {
      const response = await api.createComment(teamId, currentTopicId, entryId, {
        content,
        reply_to_comment_id: replyTarget.value?.id ?? null,
      })
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const created = mapTelegraphComment(response.comment)
      if (created && activeEntryId.value === entryId) {
        comments.value = [created, ...comments.value.filter(existing => existing.id !== created.id)]
      }
      syncEntryCommentsCount(entryId, response.comments_count)
      draftContent.value = ''
      replyTarget.value = null
      ui.toast('评论已发布', 'success')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        commentSheetVisible.value = false
        ui.redirectToLogin(topicDetailUrl(currentTopicId))
        return
      }
      const code = extractApiErrorCode(error)
      if (code === 'PROFILE_INCOMPLETE') {
        profileSheetVisible.value = true
        ui.toast('请先上传头像并设置昵称')
        return
      }
      if (code === 'COMMENTS_DISABLED') {
        if (options.item.value) {
          options.item.value = { ...options.item.value, commentsEnabled: false }
        }
        commentSheetVisible.value = false
        ui.toast('评论功能已关闭')
        return
      }
      ui.toast(extractApiErrorMessage(error, '发表评论失败，请稍后重试'))
    } finally {
      submittingComment.value = false
    }
  }

  async function saveProfileAndContinue(payload: { display_name: string; avatar_data_url: string | null }): Promise<void> {
    if (savingProfile.value) {
      return
    }
    savingProfile.value = true
    try {
      await session.updateProfile(payload)
      if (disposed) {
        return
      }
      profileSheetVisible.value = false
      if (commentSheetVisible.value && draftContent.value.trim()) {
        await submitComment()
      } else {
        ui.toast('资料已保存', 'success')
      }
    } catch (error) {
      if (!disposed) {
        ui.toast(extractApiErrorMessage(error, '保存失败，请稍后重试'))
      }
    } finally {
      savingProfile.value = false
    }
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
      comments.value = comments.value.filter(existing => existing.id !== comment.id)
      if (replyTarget.value?.id === comment.id) {
        replyTarget.value = null
      }
      syncEntryCommentsCount(comment.entryId, response.comments_count)
      ui.toast('评论已删除')
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        commentSheetVisible.value = false
        ui.redirectToLogin(topicDetailUrl(currentTopicId))
        return
      }
      ui.toast(extractApiErrorMessage(error, '删除评论失败，请稍后重试'))
    } finally {
      deletingCommentId.value = ''
    }
  }

  return {
    commentsVisible,
    isLoggedIn,
    profileComplete,
    pendingReactionIds,
    commentSheetVisible,
    activeEntry,
    comments,
    commentsHasMore,
    commentsLoading,
    loadingMoreComments,
    commentsError,
    draftContent,
    replyTarget,
    submittingComment,
    deletingCommentId,
    profileSheetVisible,
    savingProfile,
    likeEntry,
    dislikeEntry,
    openComments,
    reloadComments,
    loadMoreComments,
    closeComments,
    requestLoginFromComments,
    openProfileSetup,
    setReplyTarget,
    clearReplyTarget,
    submitComment,
    saveProfileAndContinue,
    deleteOwnComment,
  }
}
