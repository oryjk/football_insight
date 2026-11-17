import { computed, getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'
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
import type { CurrentUser } from '../../types/auth'
import type {
  DailyTopicEntry,
  DailyTopicItem,
  TelegraphComment,
  TelegraphCommentThreadPage,
  TelegraphCommentsPage,
} from '../../types/dailyTopic'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphComment,
  mapTelegraphCommentLikeSummary,
  mapTelegraphCommentThreadPage,
  mapTelegraphCommentsPage,
  mapTopicDetail,
  telegraphCommentUrl,
  telegraphEntryUrl,
} from '../../utils/dailyTopics'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { extractApiErrorCode, isProfileComplete } from '../../utils/profileGate'
import { navigateInTelegraphStack } from './navigation'
import type { EntryMiniReviewPort, EntrySessionPort } from './useTelegraphEntryPage'

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

const VALID_ID_RE = /^[a-zA-Z0-9_-]{1,128}$/
const MAX_LEGACY_PAGES = 5

function defaultRedirectToLogin(returnUrl: string): void {
  rememberPostLoginRedirect({ type: 'navigateTo', url: returnUrl })
  uni.showToast({ title: '请登录后继续', icon: 'none' })
  // `/pages/user/index` 是 tabBar 页面，必须使用 switchTab。
  uni.switchTab({ url: '/pages/user/index' })
}

function defaultToast(title: string, icon: 'none' | 'success' = 'none'): void {
  uni.showToast({ title, icon })
}

function defaultNavigateBack(fallbackEntryUrl: string): void {
  const pages = typeof getCurrentPages === 'function' ? getCurrentPages() : []
  if (pages.length >= 2) {
    uni.navigateBack({ delta: 1 })
    return
  }
  uni.redirectTo({ url: fallbackEntryUrl })
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

function patchCommentNode(
  comment: TelegraphComment,
  targetId: string,
  updater: (current: TelegraphComment) => TelegraphComment,
): TelegraphComment {
  return comment.id === targetId ? updater(comment) : comment
}

function isRouteMissing404(error: unknown): boolean {
  if (typeof error !== 'object' || error === null || !('statusCode' in error)) {
    return false
  }
  if ((error as { statusCode?: unknown }).statusCode !== 404) {
    return false
  }
  const code = extractApiErrorCode(error)
  return code !== 'COMMENT_NOT_FOUND' && code !== 'NOT_FOUND'
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
  let loadGeneration = 0
  let threadGeneration = 0
  let usingLegacyFallback = false

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
      loadGeneration += 1
      threadGeneration += 1
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
    const currentTopic = topic.value
    const targetEntryId = options.entryId()
    if (!currentTopic || !targetEntryId) {
      return
    }
    const safeCount = Math.max(0, Math.trunc(nextCount))
    topic.value = {
      ...currentTopic,
      entries: currentTopic.entries.map(item => (
        item.id === targetEntryId ? { ...item, commentsCount: safeCount } : item
      )),
    }
  }

  async function buildThreadFromLegacyComments(
    currentTopicId: string,
    currentEntryId: string,
    currentCommentId: string,
  ): Promise<TelegraphCommentThreadPage | 'not_found' | 'disabled'> {
    const allItems: TelegraphComment[] = []
    let cursor: string | null = null
    let lastPage: TelegraphCommentsPage | null = null
    for (let pageIndex = 0; pageIndex < MAX_LEGACY_PAGES; pageIndex += 1) {
      const page = mapTelegraphCommentsPage(
        await api.getComments(teamId, currentTopicId, currentEntryId, cursor, 50),
      )
      lastPage = page
      if (!page.commentsEnabled) {
        return 'disabled'
      }
      for (const item of page.items) {
        if (!allItems.some(existing => existing.id === item.id)) {
          allItems.push(item)
        }
      }
      if (!page.hasMore || !page.nextCursor || page.nextCursor === cursor) {
        break
      }
      cursor = page.nextCursor
    }
    const byId = new Map(allItems.map(item => [item.id, item]))
    const focus = byId.get(currentCommentId)
    if (!focus) {
      return 'not_found'
    }
    const builtAncestors: TelegraphComment[] = []
    const visited = new Set<string>([focus.id])
    let parentId = focus.replyTo?.commentId ?? null
    while (parentId && builtAncestors.length < 20 && !visited.has(parentId)) {
      visited.add(parentId)
      const parent = byId.get(parentId)
      if (!parent) {
        break
      }
      builtAncestors.push(parent)
      parentId = parent.replyTo?.commentId ?? null
    }
    builtAncestors.reverse()
    const directChildren = allItems.filter(item => item.replyTo?.commentId === currentCommentId)
    return {
      commentsEnabled: true,
      entryCommentsCount: lastPage?.totalCount ?? allItems.length,
      ancestors: builtAncestors,
      comment: focus,
      repliesTotalCount: directChildren.length,
      items: directChildren,
      nextCursor: null,
      hasMore: false,
    }
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

    const generation = reset ? ++threadGeneration : threadGeneration
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
        if (disposed || generation !== threadGeneration) {
          return
        }
        usingLegacyFallback = false
        page = mapTelegraphCommentThreadPage(raw)
      } catch (error) {
        if (disposed || generation !== threadGeneration) {
          return
        }
        if (reset && isRouteMissing404(error)) {
          const fallback = await buildThreadFromLegacyComments(currentTopicId, currentEntryId, currentCommentId)
          if (disposed || generation !== threadGeneration) {
            return
          }
          if (fallback === 'disabled') {
            commentsDisabled.value = true
            ancestors.value = []
            focalComment.value = null
            replies.value = []
            repliesTotalCount.value = 0
            repliesNextCursor.value = null
            errorMessage.value = '评论功能已关闭'
            return
          }
          if (fallback === 'not_found') {
            ancestors.value = []
            focalComment.value = null
            replies.value = []
            repliesTotalCount.value = 0
            repliesNextCursor.value = null
            errorMessage.value = '该评论不存在或已被删除'
            return
          }
          usingLegacyFallback = true
          page = fallback
        } else {
          throw error
        }
      }

      if (!page.commentsEnabled) {
        commentsDisabled.value = true
        ancestors.value = []
        focalComment.value = null
        replies.value = []
        repliesTotalCount.value = 0
        repliesNextCursor.value = null
        errorMessage.value = '评论功能已关闭'
        return
      }

      if (!reset && page.nextCursor && page.nextCursor === cursor) {
        repliesNextCursor.value = null
        return
      }

      ancestors.value = page.ancestors
      focalComment.value = page.comment
      repliesTotalCount.value = page.repliesTotalCount
      replies.value = reset ? page.items : mergeComments(replies.value, page.items)
      repliesNextCursor.value = page.hasMore ? page.nextCursor : null
      syncEntryCommentsCount(page.entryCommentsCount)
    } catch (error) {
      if (disposed || generation !== threadGeneration) {
        return
      }
      const code = extractApiErrorCode(error)
      if (code === 'COMMENTS_DISABLED') {
        commentsDisabled.value = true
        ancestors.value = []
        focalComment.value = null
        replies.value = []
        repliesTotalCount.value = 0
        repliesNextCursor.value = null
        errorMessage.value = '评论功能已关闭'
        return
      }
      if (code === 'COMMENT_NOT_FOUND' || code === 'NOT_FOUND') {
        ancestors.value = []
        focalComment.value = null
        replies.value = []
        repliesTotalCount.value = 0
        repliesNextCursor.value = null
        errorMessage.value = '该评论不存在或已被删除'
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
      if (!disposed && generation === threadGeneration) {
        repliesLoading.value = false
        loadingMoreReplies.value = false
      }
    }
  }

  async function loadInitial(focusOnLoad = false): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const currentCommentId = options.commentId()
    const generation = ++loadGeneration
    if (
      !VALID_ID_RE.test(currentTopicId)
      || !VALID_ID_RE.test(currentEntryId)
      || !VALID_ID_RE.test(currentCommentId)
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
      const mapped = mapTopicDetail(await api.getTopicDetail(teamId, currentTopicId))
      if (disposed || generation !== loadGeneration) {
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
      if (disposed || generation !== loadGeneration) {
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
      if (disposed || generation !== loadGeneration) {
        return
      }
      const isNotFound = typeof error === 'object' && error !== null && 'statusCode' in error && (error as { statusCode?: unknown }).statusCode === 404
      errorMessage.value = isNotFound
        ? '电报条目不存在或已下线'
        : extractApiErrorMessage(error, '评论详情加载失败，请稍后重试')
    } finally {
      if (!disposed && generation === loadGeneration) {
        loading.value = false
      }
    }
  }

  async function refreshAll(): Promise<void> {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    const currentCommentId = options.commentId()
    if (
      !VALID_ID_RE.test(currentTopicId)
      || !VALID_ID_RE.test(currentEntryId)
      || !VALID_ID_RE.test(currentCommentId)
      || loading.value
    ) {
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

  async function ensureAuthAndProfile(): Promise<boolean> {
    if (!session.hasAccessToken.value) {
      requestLogin()
      return false
    }
    if (!session.currentUser.value) {
      try {
        await session.ensureSessionReady()
      } catch {
        // Ignore transient fetch errors and evaluate state below.
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

  async function toggleCommentLike(comment: TelegraphComment): Promise<void> {
    if (pendingCommentLikeIds.value.includes(comment.id)) {
      return
    }
    const currentTopicId = options.topicId()
    if (!currentTopicId) {
      return
    }
    if (!session.hasAccessToken.value) {
      ui.redirectToLogin(currentCommentUrl())
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
      applyLikeToVisibleNodes(summary.commentId, summary.likesCount, summary.likedByMe)
    } catch (error) {
      if (disposed || session.revision.value !== startRevision) {
        return
      }
      if (isUnauthorizedError(error)) {
        session.invalidateSession(startRevision)
        ui.redirectToLogin(currentCommentUrl())
        return
      }
      ui.toast(extractApiErrorMessage(error, '点赞失败，请稍后重试'))
    } finally {
      pendingCommentLikeIds.value = pendingCommentLikeIds.value.filter(id => id !== comment.id)
    }
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
      submittingReply.value = false
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
        await submitReply()
      } else {
        ui.toast('资料已完善', 'success')
        if (commentsVisible.value && focalComment.value) {
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
      deletingCommentId.value = ''
    }
  }

  function openEntry(): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_ID_RE.test(currentTopicId) || !VALID_ID_RE.test(currentEntryId)) {
      return
    }
    ui.openEntry(telegraphEntryUrl(currentTopicId, currentEntryId))
  }

  function openComment(comment: TelegraphComment, focusReply = false): void {
    const currentTopicId = options.topicId()
    const currentEntryId = options.entryId()
    if (!VALID_ID_RE.test(currentTopicId) || !VALID_ID_RE.test(currentEntryId) || !comment.id) {
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
