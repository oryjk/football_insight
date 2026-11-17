import { computed, getCurrentScope, onScopeDispose, ref, type Ref } from 'vue'
import { setTelegraphEntryDislike, setTelegraphEntryLike } from '../../api/dailyTopics'
import { useAppSession } from '../../stores/appSession'
import { useMiniReviewStatus } from '../../stores/miniReview'
import type { DailyTopicEntry, DailyTopicItem } from '../../types/dailyTopic'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import {
  DAILY_TOPICS_TEAM_ID,
  mapTelegraphReactionSummary,
  telegraphEntryUrl,
  topicDetailUrl,
} from '../../utils/dailyTopics'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'

export interface TelegraphSessionPort {
  hasAccessToken: Ref<boolean>
  revision: Ref<number>
  invalidateSession: (revision: number) => void
}

export interface TelegraphMiniReviewPort {
  shouldRestrictFeatures: Ref<boolean>
}

export interface TelegraphInteractionsApi {
  setLike: typeof setTelegraphEntryLike
  setDislike: typeof setTelegraphEntryDislike
}

export interface TelegraphInteractionsUi {
  redirectToLogin: (returnUrl: string) => void
  navigateToEntry: (url: string) => void
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

function defaultNavigateToEntry(url: string): void {
  uni.navigateTo({ url })
}

function defaultToast(title: string, icon: 'none' | 'success' = 'none'): void {
  uni.showToast({ title, icon })
}

export function useTelegraphInteractions(options: UseTelegraphInteractionsOptions) {
  const session = options.session ?? useAppSession()
  const miniReview = options.miniReview ?? useMiniReviewStatus()
  const teamId = options.teamId ?? DAILY_TOPICS_TEAM_ID
  const api: TelegraphInteractionsApi = {
    setLike: setTelegraphEntryLike,
    setDislike: setTelegraphEntryDislike,
    ...options.api,
  }
  const ui: TelegraphInteractionsUi = {
    redirectToLogin: defaultRedirectToLogin,
    navigateToEntry: defaultNavigateToEntry,
    toast: defaultToast,
    ...options.ui,
  }

  const pendingReactionIds = ref<string[]>([])
  let disposed = false

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed = true
    })
  }

  const commentsVisible = computed(() =>
    !miniReview.shouldRestrictFeatures.value && Boolean(options.item.value?.commentsEnabled),
  )

  function updateEntry(entryId: string, updater: (entry: DailyTopicEntry) => DailyTopicEntry): void {
    const current = options.item.value
    if (!current) {
      return
    }
    const nextEntries = current.entries.map(entry => (entry.id === entryId ? updater(entry) : entry))
    options.item.value = {
      ...current,
      entries: nextEntries,
    }
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

  function openEntry(entry: DailyTopicEntry, focusComment = false): void {
    const currentTopicId = options.topicId()
    if (!currentTopicId) {
      return
    }
    ui.navigateToEntry(telegraphEntryUrl(currentTopicId, entry.id, focusComment && commentsVisible.value))
  }

  return {
    commentsVisible,
    pendingReactionIds,
    likeEntry,
    dislikeEntry,
    openEntry,
  }
}
