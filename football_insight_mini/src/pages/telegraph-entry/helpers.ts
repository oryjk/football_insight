import type { Ref } from 'vue'
import { isSessionSupersededError } from '../../stores/appSession'
import type { CurrentUser } from '../../types/auth'
import type {
  DailyTopicEntry,
  DailyTopicItem,
  TelegraphComment,
  TelegraphCommentThreadPage,
  TelegraphCommentsPage,
} from '../../types/dailyTopic'
import { extractApiErrorCode, extractApiErrorMessage, isNotFoundError, isUnauthorizedError } from '../../utils/apiError'
import {
  mapTelegraphCommentLikeSummary,
  mapTelegraphCommentsPage,
  telegraphCommentUrl,
} from '../../utils/dailyTopics'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { isProfileComplete } from '../../utils/profileGate'
import { buildSharePath, buildShareQuery } from '../../utils/shareSource'
import {
  buildTelegraphEntrySharePayload,
  cleanTelegraphHeadline,
} from '../../utils/telegraphShareCard'

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
  ensureMiniReviewReady?: () => Promise<unknown>
}

export const VALID_TELEGRAPH_ID_RE = /^[a-zA-Z0-9_-]{1,128}$/
const MAX_LEGACY_PAGES = 5

export function defaultRedirectToLogin(returnUrl: string): void {
  rememberPostLoginRedirect({ type: 'navigateTo', url: returnUrl })
  uni.showToast({ title: '请登录后继续', icon: 'none' })
  // `/pages/user/index` 是 tabBar 页面，必须使用 switchTab。
  uni.switchTab({ url: '/pages/user/index' })
}

export function defaultToast(title: string, icon: 'none' | 'success' = 'none'): void {
  uni.showToast({ title, icon })
}

export function defaultNavigateBack(fallbackEntryUrl: string): void {
  const pages = typeof getCurrentPages === 'function' ? getCurrentPages() : []
  if (pages.length >= 2) {
    uni.navigateBack({ delta: 1 })
    return
  }
  uni.redirectTo({ url: fallbackEntryUrl })
}

export function confirmDeleteCommentModal(onConfirm: () => void): void {
  if (typeof uni === 'undefined' || typeof uni.showModal !== 'function') {
    onConfirm()
    return
  }
  uni.showModal({
    title: '删除评论',
    content: '确认删除这条评论吗？删除后不可恢复。',
    showCancel: true,
    success: (res) => {
      if (res.confirm) {
        onConfirm()
      }
    },
  })
}

export function mergeTelegraphComments(
  existing: TelegraphComment[],
  incoming: TelegraphComment[],
): TelegraphComment[] {
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

export function patchCommentNode(
  comment: TelegraphComment,
  targetId: string,
  updater: (current: TelegraphComment) => TelegraphComment,
): TelegraphComment {
  return comment.id === targetId ? updater(comment) : comment
}

export function updateTopicEntry(
  topic: Ref<DailyTopicItem | null>,
  targetEntryId: string,
  updater: (current: DailyTopicEntry) => DailyTopicEntry,
): void {
  const currentTopic = topic.value
  if (!currentTopic || !targetEntryId) {
    return
  }
  topic.value = {
    ...currentTopic,
    entries: currentTopic.entries.map(item => (item.id === targetEntryId ? updater(item) : item)),
  }
}

export function syncTopicEntryCommentsCount(
  topic: Ref<DailyTopicItem | null>,
  targetEntryId: string,
  nextCount: number,
): void {
  const safeCount = Math.max(0, Math.trunc(nextCount))
  updateTopicEntry(topic, targetEntryId, item => ({ ...item, commentsCount: safeCount }))
}

export function isRouteMissing404(error: unknown): boolean {
  if (!isNotFoundError(error)) {
    return false
  }
  const code = extractApiErrorCode(error)
  return code !== 'COMMENT_NOT_FOUND' && code !== 'NOT_FOUND'
}

export async function ensureTelegraphAuthAndProfile(options: {
  session: EntrySessionPort
  isDisposed: () => boolean
  profileSheetVisible: Ref<boolean>
  requestLogin: () => void
}): Promise<boolean> {
  const { session, isDisposed, profileSheetVisible, requestLogin } = options
  if (!session.hasAccessToken.value) {
    requestLogin()
    return false
  }
  if (!session.currentUser.value) {
    try {
      await session.ensureSessionReady()
    } catch {
      // Ignore transient fetch errors and evaluate login state below.
    }
  }
  if (isDisposed()) {
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

export async function runSaveProfileAndContinue(options: {
  payload: { display_name: string; avatar_data_url: string | null }
  session: EntrySessionPort
  savingProfile: Ref<boolean>
  profileSheetVisible: Ref<boolean>
  isDisposed: () => boolean
  canContinueWithDraft: () => boolean
  onContinueWithDraft: () => Promise<void>
  canFocusAfterSave: () => boolean
  inputFocusTrigger: Ref<number>
  toast: (title: string, icon?: 'none' | 'success') => void
}): Promise<void> {
  const {
    payload,
    session,
    savingProfile,
    profileSheetVisible,
    isDisposed,
    canContinueWithDraft,
    onContinueWithDraft,
    canFocusAfterSave,
    inputFocusTrigger,
    toast,
  } = options
  if (savingProfile.value) {
    return
  }
  const startRevision = session.revision.value
  savingProfile.value = true
  try {
    await session.updateProfile(payload)
    if (isDisposed() || session.revision.value !== startRevision) {
      return
    }
    profileSheetVisible.value = false
    if (canContinueWithDraft()) {
      await onContinueWithDraft()
    } else {
      toast('资料已完善', 'success')
      if (canFocusAfterSave()) {
        inputFocusTrigger.value += 1
      }
    }
  } catch (error) {
    if (isDisposed() || isSessionSupersededError(error) || session.revision.value !== startRevision) {
      return
    }
    toast(extractApiErrorMessage(error, '保存失败，请稍后重试'))
  } finally {
    if (!isDisposed() && session.revision.value === startRevision) {
      savingProfile.value = false
    }
  }
}

export async function runToggleCommentLike(options: {
  comment: TelegraphComment
  teamId: number
  topicId: string
  returnUrl: string
  session: EntrySessionPort
  pendingCommentLikeIds: Ref<string[]>
  isDisposed: () => boolean
  setCommentLike: (
    teamId: number,
    topicId: string,
    entryId: string,
    commentId: string,
    active: boolean,
  ) => Promise<unknown>
  applyLikeSummary: (commentId: string, likesCount: number, likedByMe: boolean) => void
  redirectToLogin: (returnUrl: string) => void
  toast: (title: string, icon?: 'none' | 'success') => void
}): Promise<void> {
  const {
    comment,
    teamId,
    topicId,
    returnUrl,
    session,
    pendingCommentLikeIds,
    isDisposed,
    setCommentLike,
    applyLikeSummary,
    redirectToLogin,
    toast,
  } = options
  if (pendingCommentLikeIds.value.includes(comment.id) || !topicId) {
    return
  }
  if (!session.hasAccessToken.value) {
    redirectToLogin(returnUrl)
    return
  }

  const startRevision = session.revision.value
  pendingCommentLikeIds.value = [...pendingCommentLikeIds.value, comment.id]
  try {
    const raw = await setCommentLike(
      teamId,
      topicId,
      comment.entryId,
      comment.id,
      !comment.likedByMe,
    )
    if (isDisposed() || session.revision.value !== startRevision) {
      return
    }
    const summary = mapTelegraphCommentLikeSummary(raw)
    applyLikeSummary(summary.commentId, summary.likesCount, summary.likedByMe)
  } catch (error) {
    if (isDisposed() || session.revision.value !== startRevision) {
      return
    }
    if (isUnauthorizedError(error)) {
      session.invalidateSession(startRevision)
      redirectToLogin(returnUrl)
      return
    }
    toast(extractApiErrorMessage(error, '点赞失败，请稍后重试'))
  } finally {
    if (!isDisposed()) {
      pendingCommentLikeIds.value = pendingCommentLikeIds.value.filter(id => id !== comment.id)
    }
  }
}

export async function buildThreadFromLegacyComments(options: {
  teamId: number
  topicId: string
  entryId: string
  commentId: string
  getComments: (
    teamId: number,
    topicId: string,
    entryId: string,
    cursor?: string | null,
    limit?: number,
  ) => Promise<unknown>
}): Promise<TelegraphCommentThreadPage | 'not_found' | 'disabled'> {
  const { teamId, topicId, entryId, commentId, getComments } = options
  const allItems: TelegraphComment[] = []
  let cursor: string | null = null
  let lastPage: TelegraphCommentsPage | null = null
  for (let pageIndex = 0; pageIndex < MAX_LEGACY_PAGES; pageIndex += 1) {
    const page = mapTelegraphCommentsPage(
      await getComments(teamId, topicId, entryId, cursor, 50),
    )
    lastPage = page
    if (!page.commentsEnabled) {
      return 'disabled'
    }
    for (let i = 0; i < page.items.length; i += 1) {
      const item = page.items[i]
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
  const focus = byId.get(commentId)
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
  const directChildren = allItems.filter(item => item.replyTo?.commentId === commentId)
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

export interface TelegraphCommentShareOptions {
  topicId: string
  entryId: string
  commentId: string
  entry: Pick<DailyTopicEntry, 'title' | 'text'> | null
  comment: Pick<TelegraphComment, 'author' | 'content'> | null
  imageUrl?: string
}

export function buildTelegraphCommentSharePayload({
  topicId,
  entryId,
  commentId,
  entry,
  comment,
  imageUrl,
}: TelegraphCommentShareOptions) {
  const valid =
    VALID_TELEGRAPH_ID_RE.test(topicId)
    && VALID_TELEGRAPH_ID_RE.test(entryId)
    && VALID_TELEGRAPH_ID_RE.test(commentId)
  if (!valid) {
    return buildTelegraphEntrySharePayload({ topicId, entryId, entry, imageUrl })
  }
  const headline = entry ? cleanTelegraphHeadline(entry.title) || entry.title : '每日电报'
  const title = comment
    ? `${comment.author.displayName}：${comment.content.slice(0, 26)}｜${headline}`
    : headline
  return {
    title,
    path: buildSharePath(telegraphCommentUrl(topicId, entryId, commentId), 'telegraph_comment'),
    ...(imageUrl ? { imageUrl } : {}),
  }
}

export function buildTelegraphCommentTimelinePayload(options: TelegraphCommentShareOptions) {
  const { topicId, entryId, commentId, imageUrl } = options
  const valid =
    VALID_TELEGRAPH_ID_RE.test(topicId)
    && VALID_TELEGRAPH_ID_RE.test(entryId)
    && VALID_TELEGRAPH_ID_RE.test(commentId)
  const share = buildTelegraphCommentSharePayload(options)
  const prefix = valid
    ? `topicId=${encodeURIComponent(topicId)}&entryId=${encodeURIComponent(entryId)}&commentId=${encodeURIComponent(commentId)}&`
    : ''
  return {
    title: share.title,
    query: `${prefix}${buildShareQuery('telegraph_comment')}`,
    ...(imageUrl ? { imageUrl } : {}),
  }
}
