import { computed, reactive, ref } from 'vue'
import {
  addTeamBoardComment,
  createTeamBoardPost,
  getTeamBoard,
  toggleTeamBoardPostLike,
} from '../../api/teamBoard'
import type {
  TeamBoardComposerPreset,
  TeamBoardInsightKind,
  TeamBoardViewResponse,
} from '../../types/teamBoard'
import { extractApiErrorMessage } from '../../utils/apiError'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import { useAppSession } from '../../stores/appSession'
import { appendComment, applyLikeSummary, prependPost, resolveTeamBoardLoadFailure } from './helpers'

export function useTeamBoardPage() {
  const teamId = ref<number | null>(null)
  const loading = ref(true)
  const errorMessage = ref('')
  const composerOpen = ref(false)
  const selectedInsightKind = ref<TeamBoardInsightKind | null>(null)
  const postTitle = ref('')
  const postCommentary = ref('')
  const composerErrorMessage = ref('')
  const commentDrafts = reactive<Record<string, string>>({})
  const board = ref<TeamBoardViewResponse | null>(null)
  // 登录标记与 401 失效统一走 appSession。
  const session = useAppSession()

  const posts = computed(() => board.value?.posts ?? [])
  const composerPresets = computed(() => board.value?.composer_presets ?? [])
  const selectedPreset = computed<TeamBoardComposerPreset | null>(() =>
    composerPresets.value.find((item) => item.insight_kind === selectedInsightKind.value) ?? null,
  )

  async function loadBoard(): Promise<void> {
    if (!teamId.value) {
      errorMessage.value = '缺少球队 id'
      loading.value = false
      return
    }

    if (!session.hasAccessToken.value) {
      rememberPostLoginRedirect({
        type: 'navigateTo',
        url: `/pages/team-board/index?teamId=${teamId.value}`,
      })
      uni.switchTab({ url: '/pages/user/index' })
      return
    }

    loading.value = true
    errorMessage.value = ''
    const requestRevision = session.revision.value

    try {
      board.value = await getTeamBoard(teamId.value)
    } catch (error) {
      const failure = resolveTeamBoardLoadFailure(error, '战术板加载失败，请稍后重试。')
      if (failure.kind === 'unauthorized') {
        // 401：仅失效当前 revision 的会话，再走既有登录引导；不断改新账号状态。
        session.invalidateSession(requestRevision)
        rememberPostLoginRedirect({
          type: 'navigateTo',
          url: `/pages/team-board/index?teamId=${teamId.value}`,
        })
        uni.switchTab({ url: '/pages/user/index' })
        return
      }

      errorMessage.value = failure.message
    } finally {
      loading.value = false
    }
  }

  function openComposer(kind: TeamBoardInsightKind): void {
    selectedInsightKind.value = kind
    postTitle.value = ''
    postCommentary.value = ''
    composerErrorMessage.value = ''
    composerOpen.value = true
  }

  function closeComposer(): void {
    composerOpen.value = false
  }

  async function publishPost(): Promise<void> {
    if (!teamId.value || !selectedInsightKind.value) {
      return
    }

    try {
      const created = await createTeamBoardPost(teamId.value, {
        insight_kind: selectedInsightKind.value,
        title: postTitle.value.trim(),
        commentary: postCommentary.value.trim(),
      })

      if (board.value) {
        board.value = prependPost(board.value, created)
      }

      composerOpen.value = false
      selectedInsightKind.value = null
      postTitle.value = ''
      postCommentary.value = ''
      composerErrorMessage.value = ''
      uni.showToast({ title: '发布成功', icon: 'success' })
    } catch (error) {
      composerErrorMessage.value = extractApiErrorMessage(error, '发布失败，请稍后重试。')
    }
  }

  async function submitComment(postId: string): Promise<void> {
    const content = (commentDrafts[postId] ?? '').trim()
    if (!content) {
      return
    }

    try {
      const created = await addTeamBoardComment(postId, { content })

      if (board.value) {
        board.value = appendComment(board.value, postId, created)
      }

      commentDrafts[postId] = ''
      uni.showToast({ title: '评论成功', icon: 'success' })
    } catch (error) {
      uni.showToast({ title: extractApiErrorMessage(error, '评论失败'), icon: 'none' })
    }
  }

  async function toggleLike(postId: string): Promise<void> {
    try {
      const summary = await toggleTeamBoardPostLike(postId)

      if (board.value) {
        board.value = applyLikeSummary(board.value, postId, summary)
      }
    } catch (error) {
      uni.showToast({ title: extractApiErrorMessage(error, '支持失败'), icon: 'none' })
    }
  }

  return {
    teamId, loading, errorMessage, board, posts, composerPresets,
    composerOpen, selectedPreset, postTitle, postCommentary, composerErrorMessage, commentDrafts,
    loadBoard, openComposer, closeComposer, publishPost, submitComment, toggleLike,
  }
}
