import { ref } from 'vue'
import {
  getDirectorCircle,
  getDirectorPlan,
  postDirectorComment,
  setDirectorReaction,
} from '../../api/director'
import type { AppSession } from '../../stores/appSession'
import type {
  DirectorCircleResponse,
  DirectorPlanDetailResponse,
} from '../../types/director'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { buildDirectorViewUrl } from '../../utils/directorLinks'
import { createLatestGuard } from '../../utils/latestGuard'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { countUnicodeChars, DIRECTOR_COMMENT_MAX } from './helpers'

export function useDirectorView(session: AppSession) {
  const shareCode = ref('')
  const detail = ref<DirectorPlanDetailResponse | null>(null)
  const loading = ref(false)
  const error = ref('')

  const circle = ref<DirectorCircleResponse | null>(null)
  const circleLoading = ref(false)
  const circleError = ref('')

  const reactionPending = ref(false)
  const commentSubmitting = ref(false)

  const detailGuard = createLatestGuard(() => [shareCode.value, session.revision.value])
  const circleGuard = createLatestGuard(() => [detail.value?.plan.circle_code ?? '', session.revision.value])

  function redirectToLogin(toastText: string): void {
    const url = buildDirectorViewUrl(shareCode.value)
    if (url) {
      rememberPostLoginRedirect({ type: 'navigateTo', url })
    }
    uni.showToast({ title: toastText, icon: 'none' })
    uni.switchTab({ url: '/pages/user/index' })
  }

  async function loadCircle(circleCode: string): Promise<void> {
    if (!circleCode) return
    const token = circleGuard.begin()
    circleLoading.value = true
    circleError.value = ''
    try {
      const res = await getDirectorCircle(circleCode)
      if (!token.isCurrent()) return
      circle.value = res
    } catch (cause) {
      if (token.isCurrent()) {
        circleError.value = extractApiErrorMessage(cause, '好友圈共识加载失败，请重试')
      }
    } finally {
      if (token.owns()) {
        circleLoading.value = false
      }
    }
  }

  async function loadDetail(code?: string): Promise<void> {
    if (code !== undefined) {
      shareCode.value = code
    }
    if (!shareCode.value) {
      error.value = '分享链接无效'
      return
    }
    const token = detailGuard.begin()
    loading.value = true
    error.value = ''
    try {
      const res = await getDirectorPlan(shareCode.value)
      if (!token.isCurrent()) return
      detail.value = res
      void loadCircle(res.plan.circle_code)
    } catch (cause) {
      if (token.isCurrent()) {
        error.value = extractApiErrorMessage(cause, '该建队方案暂时无法打开，请稍后重试')
      }
    } finally {
      if (token.owns()) {
        loading.value = false
      }
    }
  }

  async function toggleReaction(target: 1 | -1): Promise<void> {
    if (!detail.value || reactionPending.value) return
    if (!session.hasAccessToken.value) {
      redirectToLogin('请先登录后表态')
      return
    }
    const current = detail.value.plan.my_reaction ?? 0
    const nextValue: 1 | -1 | 0 = current === target ? 0 : target
    reactionPending.value = true
    try {
      const res = await setDirectorReaction(detail.value.plan.share_code, nextValue)
      if (detail.value) {
        detail.value = {
          ...detail.value,
          plan: {
            ...detail.value.plan,
            likes_count: res.likes_count,
            dislikes_count: res.dislikes_count,
            my_reaction: res.my_reaction,
          },
        }
      }
    } catch (cause) {
      if (isUnauthorizedError(cause)) {
        redirectToLogin('登录已过期，请重新登录')
        return
      }
      uni.showToast({ title: extractApiErrorMessage(cause, '操作失败，请稍后重试'), icon: 'none' })
    } finally {
      reactionPending.value = false
    }
  }

  async function submitComment(rawText: string): Promise<boolean> {
    if (!detail.value || commentSubmitting.value) return false
    const content = rawText.trim()
    if (!content) {
      uni.showToast({ title: '写一句锐评再发送吧', icon: 'none' })
      return false
    }
    if (countUnicodeChars(content) > DIRECTOR_COMMENT_MAX) {
      uni.showToast({ title: `锐评最多 ${DIRECTOR_COMMENT_MAX} 字`, icon: 'none' })
      return false
    }
    if (!session.hasAccessToken.value) {
      redirectToLogin('请先登录后发表锐评')
      return false
    }
    commentSubmitting.value = true
    try {
      const comment = await postDirectorComment(detail.value.plan.share_code, content)
      if (detail.value) {
        detail.value = {
          ...detail.value,
          plan: {
            ...detail.value.plan,
            comments_count: detail.value.plan.comments_count + 1,
          },
          comments: [comment, ...detail.value.comments],
        }
      }
      uni.showToast({ title: '锐评已发送', icon: 'success' })
      return true
    } catch (cause) {
      if (isUnauthorizedError(cause)) {
        redirectToLogin('登录已过期，请重新登录')
        return false
      }
      uni.showToast({ title: extractApiErrorMessage(cause, '发送失败，请稍后重试'), icon: 'none' })
      return false
    } finally {
      commentSubmitting.value = false
    }
  }

  function dispose(): void {
    detailGuard.invalidate()
    circleGuard.invalidate()
  }

  return {
    shareCode,
    detail,
    loading,
    error,
    circle,
    circleLoading,
    circleError,
    reactionPending,
    commentSubmitting,
    loadDetail,
    loadCircle,
    toggleReaction,
    submitComment,
    dispose,
  }
}
