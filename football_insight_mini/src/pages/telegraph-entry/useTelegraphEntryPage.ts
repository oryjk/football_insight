import { computed, getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'
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
import type { CurrentUser } from '../../types/auth'
import type { DailyTopicEntry, DailyTopicItem, TelegraphComment } from '../../types/dailyTopic'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphComment,
  mapTelegraphCommentLikeSummary,
  mapTelegraphCommentsPage,
  mapTelegraphReactionSummary,
  mapTopicDetail,
  telegraphCommentUrl,
  telegraphEntryUrl,
} from '../../utils/dailyTopics'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { extractApiErrorCode, isProfileComplete } from '../../utils/profileGate'
import { navigateInTelegraphStack } from './navigation'

export interface EntrySessionPort {
  currentUser: Ref<CurrentUser | null>
  hasAccessToken: Ref<boolean>
  revision: Ref<number>
  ensureSessionReady: () => Promise<CurrentUser | null>
  updateProfile: (payload: { display_name: string; avatar_data_url?: string | null }) => Promise<CurrentUser>
  invalidateSession: (revision: number) => void
}

export interface EntryMiniReviewPort {
  shouldRestrictFeatures: Ref<boolean>
}

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

const VALID_ID_RE = /^[a-zA-Z0-9_-]{1,128}$/

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
  let loadGeneration = 0
  let commentsGeneration = 0

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      loadGeneration += 1
      commentsGeneration += 1
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

  function updateEntry(updater: (current: DailyTopicEntry) => DailyTopicEntry): void {
    const currentTopic = topic.value
    const targetId = options.entryId()
    if (!currentTopic || !targetId) {
      return
    }
    const nextEntries = currentTopic.entries.map(item => (item.id === targetId ? updater(item) : item))
    topic.value = {
      ...currentTopic,
      entries: nextEntries,
    }
  }

  function syncEntryCommentsCount(nextCount: number): void {
    const safeCount = Math.max(0, Math.trunc(nextCount))
    updateEntry(item => ({ ...item, commentsCount: safeCount }))
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

    const generation = reset ? ++commentsGeneration : commentsGeneration
    const cursor = reset ? null : commentsNextCursor.value
    if (reset) {
      commentsLoading.value = true
      commentsError.value = ''
    } else {
      loadingMoreComments.value = true
    }

    try {
      const raw = await api.getComments(teamId, currentTopicId, currentEntryId, cursor, 20, { rootOnly: true })
      if (disposed || generation !== commentsGeneration) {
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
      comments.value = reset ? rootItems : mergeComments(comments.value, rootItems)
      commentsNextCursor.value = page.hasMore ? page.nextCursor : null
      syncEntryCommentsCount(page.totalCount)
    } catch (error) {
      if (disposed || generation !== commentsGeneration) {
        return
      }
      const msg = extractApiErrorMessage(error, '评论加载失败，请稍后重试')
      if (reset) {
        commentsError.value = msg
      } else {
        ui.toast(msg)
      }
    } finally {
      if (!disposed && generation === commentsGeneration) {
        commentsLoading.value = false
        loadingMoreComments.value = false
      }
    }
  }

  async function loadInitial(focusOnLoad = false): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const generation = ++loadGeneration
    if (!VALID_ID_RE.test(currentTopicId) || !VALID_ID_RE.test(currentEntryId)) {
      topic.value = null
      errorMessage.value = '电报条目不存在或已下线'
      return
    }

    loading.value = true
    errorMessage.value = ''
    try {
      const mapped = mapTopicDetail(await api.getTopicDetail(teamId, currentTopicId))
      if (disposed || generation !== loadGeneration) {
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
        if (disposed || generation !== loadGeneration) {
          return
        }
        if (focusOnLoad && session.hasAccessToken.value && isProfileComplete(session.currentUser.value)) {
          inputFocusTrigger.value += 1
        }
      }
    } catch (error) {
      if (disposed || generation !== loadGeneration) {
        return
      }
      const isNotFound = typeof error === 'object' && error !== null && 'statusCode' in error && (error as { statusCode?: unknown }).statusCode === 404
      errorMessage.value = isNotFound
        ? '电报条目不存在或已下线'
        : extractApiErrorMessage(error, '电报加载失败，请稍后重试')
    } finally {
      if (!disposed && generation === loadGeneration) {
        loading.value = false
      }
    }
  }

  async function refreshAll(): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_ID_RE.test(currentTopicId) || !VALID_ID_RE.test(currentEntryId) || loading.value) {
      return
    }
    const generation = ++loadGeneration
    try {
      const mapped = mapTopicDetail(await api.getTopicDetail(teamId, currentTopicId))
      if (disposed || generation !== loadGeneration) {
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
      updateEntry(item => ({
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
      reactionPending.value = false
    }
  }

  function likeEntry(): Promise<void> {
    return toggleEntryReaction('like')
  }

  function dislikeEntry(): Promise<void> {
    return toggleEntryReaction('dislike')
  }

  async function toggleCommentLike(comment: TelegraphComment): Promise<void> {
    if (pendingCommentLikeIds.value.includes(comment.id)) {
      return
    }
    const currentTopicId = options.topicId()
    if (!currentTopicId) {
      return
    }
    if (!session.hasAccessToken.value) {
      ui.redirectToLogin(currentPageUrl())
      return
    }

    const startRevision = session.revision.value
    pendingCommentLikeIds.value = [...pendingCommentLikeIds.value, comment.id]
    try {
      const raw = await api.setCommentLike(
        teamId,
        currentTopicId,
        comment.entryId,
        comment.id,
        !comment.likedByMe,
      )
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      const summary = mapTelegraphCommentLikeSummary(raw)
      comments.value = comments.value.map(item => (
        item.id === summary.commentId
          ? { ...item, likesCount: summary.likesCount, likedByMe: summary.likedByMe }
          : item
      ))
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentPageUrl())
        return
      }
      ui.toast(extractApiErrorMessage(error, '点赞失败，请稍后重试'))
    } finally {
      pendingCommentLikeIds.value = pendingCommentLikeIds.value.filter(id => id !== comment.id)
    }
  }

  function requestLogin(): void {
    if (options.topicId() && options.entryId()) {
      ui.redirectToLogin(currentPageUrl())
    }
  }

  async function ensureAuthAndProfile(): Promise<boolean> {
    if (!session.hasAccessToken.value) {
      requestLogin()
      return false
    }
    if (!session.currentUser.value) {
      try {
        await session.ensureSessionReady()
      } catch {
        // Ignore transient session fetch errors and check login state below.
      }
    }
    if (disposed) {
      return false
    }
    if (!session.hasAccessToken.value || !session.currentUser.value) {
      requestLogin()
      return false
    }
    if (!isProfileComplete(session.currentUser.value)) {
      profileSheetVisible.value = true
      return false
    }
    return true
  }

  async function openProfileSetup(): Promise<void> {
    await ensureAuthAndProfile()
  }

  function openComment(comment: TelegraphComment, focusReply = false): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_ID_RE.test(currentTopicId) || !VALID_ID_RE.test(currentEntryId) || !comment.id) {
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
      if (commentsVisible.value && draftContent.value.trim()) {
        await submitComment()
      } else {
        ui.toast('资料已完善', 'success')
        if (commentsVisible.value) {
          inputFocusTrigger.value += 1
        }
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
      deletingCommentId.value = ''
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
