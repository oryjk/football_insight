import { computed, ref, watch, type Ref } from 'vue'
import { cancelSupportVote, castSupportVote, getMatchSupportDetail } from '../../api/support'
import type { AppSession } from '../../stores/appSession'
import type { CastSupportVotePayload, SupportMatchDetail } from '../../types/support'
import { extractApiErrorMessage } from '../../utils/apiError'
import { createLatestGuard } from '../../utils/latestGuard'
import { resolveSupportVoteState } from '../../utils/supportVoteState'
import { resolveSupportCountdownSeconds, resolveSupportedTeam } from './helpers'

export interface SupportPageDeps {
  session: Pick<AppSession, 'revision' | 'hasAccessToken'>
  /** 页面持有的比赛 ID，与海报等同页模块共享作为上下文快照。 */
  matchId: Ref<number | null>
  /** 页面数据被替换或重新加载时通知海报失效。 */
  onDetailInvalidated?: () => void
  now?: () => number
  /** 测试注入可控的读写；默认走真实 api 层。 */
  getMatchSupportDetail?: (matchId: number) => Promise<SupportMatchDetail>
  castSupportVote?: (matchId: number, payload: CastSupportVotePayload) => Promise<SupportMatchDetail>
  cancelSupportVote?: (matchId: number) => Promise<SupportMatchDetail>
}

/** 助力提交结果；toast / 登录引导由页面按结果展示。 */
export type SupportVoteOutcome =
  | { kind: 'login-required', matchId: number }
  | { kind: 'blocked' }
  | { kind: 'cast' }
  | { kind: 'canceled' }
  | { kind: 'failed', message: string }
  | { kind: 'stale' }
  | { kind: 'ignored' }

/**
 * 助力页的数据与提交编排：加载、助力/取消、开球倒计时。
 *
 * 加载与助力各自一个 latestGuard，快照 session revision + matchId：
 * 重叠的 onShow 加载只发布最新一次；换号或换比赛后，旧请求的成功/失败都不发布；
 * 重新加载会让在途助力失效并复位提交态。
 */
export function useSupportPage(deps: SupportPageDeps) {
  const { session, matchId } = deps
  const now = deps.now ?? Date.now
  const fetchDetail = deps.getMatchSupportDetail ?? getMatchSupportDetail
  const writeVote = deps.castSupportVote ?? castSupportVote
  const removeVote = deps.cancelSupportVote ?? cancelSupportVote

  const loading = ref(true)
  const errorMessage = ref('')
  const detail = ref<SupportMatchDetail | null>(null) as Ref<SupportMatchDetail | null>
  const voteSubmittingTeamId = ref<number | null>(null)
  const liveCountdownSeconds = ref(0)
  let countdownTimer: ReturnType<typeof setInterval> | null = null
  let pageActive = false

  const contextSnapshot = () => [session.revision.value, matchId.value]
  const loadGuard = createLatestGuard(contextSnapshot)
  const voteGuard = createLatestGuard(contextSnapshot)

  const supportedTeam = computed(() => resolveSupportedTeam(detail.value))

  // 冷启动静默登录也会替换会话；丢弃旧结果后必须为可见页面重新读取详情。
  watch(session.revision, () => {
    detail.value = null
    errorMessage.value = ''
    liveCountdownSeconds.value = 0
    if (pageActive) void loadPage()
    else deps.onDetailInvalidated?.()
  }, { flush: 'sync' })

  function setMatchId(raw: unknown): void {
    const parsed = typeof raw === 'string' ? Number(raw) : Number.NaN
    matchId.value = Number.isFinite(parsed) && parsed > 0 ? parsed : null
  }

  function voteState(teamId: number) {
    return resolveSupportVoteState(detail.value, teamId, voteSubmittingTeamId.value, now())
  }

  async function loadPage(): Promise<void> {
    pageActive = true
    stopCountdown()
    deps.onDetailInvalidated?.()
    voteGuard.invalidate()
    voteSubmittingTeamId.value = null
    if (!matchId.value) {
      loadGuard.invalidate()
      errorMessage.value = '缺少比赛参数，无法打开助力页。'
      loading.value = false
      return
    }

    const token = loadGuard.begin()
    loading.value = true
    errorMessage.value = ''

    try {
      const result = await fetchDetail(matchId.value)
      if (!token.isCurrent()) return
      detail.value = result
      syncCountdownState()
      startCountdown()
    } catch (error) {
      if (!token.isCurrent()) return
      errorMessage.value = extractApiErrorMessage(error, '助力页加载失败，请稍后重试。')
    } finally {
      if (token.owns()) loading.value = false
    }
  }

  async function submitVote(teamId: number): Promise<SupportVoteOutcome> {
    const selectedMatchId = matchId.value
    if (!detail.value || !selectedMatchId || voteSubmittingTeamId.value !== null) {
      return { kind: 'ignored' }
    }

    if (!session.hasAccessToken.value) {
      return { kind: 'login-required', matchId: selectedMatchId }
    }

    if (voteState(teamId).disabled) {
      return { kind: 'blocked' }
    }

    const canceling = detail.value.viewer.has_supported && detail.value.viewer.supported_team_id === teamId
    const token = voteGuard.begin()
    voteSubmittingTeamId.value = teamId
    try {
      const updated = canceling
        ? await removeVote(selectedMatchId)
        : await writeVote(selectedMatchId, { supported_team_id: teamId })
      if (!token.isCurrent()) return { kind: 'stale' }
      deps.onDetailInvalidated?.()
      detail.value = updated
      syncCountdownState()
      return { kind: canceling ? 'canceled' : 'cast' }
    } catch (error) {
      if (!token.isCurrent()) return { kind: 'stale' }
      return { kind: 'failed', message: extractApiErrorMessage(error, canceling ? '取消失败' : '助力失败') }
    } finally {
      if (token.owns()) voteSubmittingTeamId.value = null
    }
  }

  function closeSupportWindow(): void {
    if (!detail.value) return
    detail.value.support_window_status = 'closed'
    detail.value.viewer.can_support = false
  }

  function syncCountdownState(): void {
    if (!detail.value) {
      liveCountdownSeconds.value = 0
      return
    }

    liveCountdownSeconds.value = resolveSupportCountdownSeconds(detail.value.kickoff_at, now())
    if (liveCountdownSeconds.value <= 0) {
      closeSupportWindow()
    }
  }

  function stopCountdown(): void {
    if (!countdownTimer) return
    clearInterval(countdownTimer)
    countdownTimer = null
  }

  function pausePage(): void {
    pageActive = false
    stopCountdown()
    loadGuard.invalidate()
    voteGuard.invalidate()
    loading.value = false
    voteSubmittingTeamId.value = null
    deps.onDetailInvalidated?.()
  }

  function startCountdown(): void {
    stopCountdown()
    if (!detail.value || detail.value.support_window_status === 'closed') return

    countdownTimer = setInterval(() => {
      if (!detail.value) {
        stopCountdown()
        return
      }

      liveCountdownSeconds.value = Math.max(0, liveCountdownSeconds.value - 1)
      if (liveCountdownSeconds.value <= 0) {
        closeSupportWindow()
        stopCountdown()
      }
    }, 1000)
  }

  return {
    loading,
    errorMessage,
    detail,
    supportedTeam,
    voteSubmittingTeamId,
    liveCountdownSeconds,
    setMatchId,
    voteState,
    loadPage,
    submitVote,
    pausePage,
    stopCountdown,
  }
}

export type SupportPage = ReturnType<typeof useSupportPage>
