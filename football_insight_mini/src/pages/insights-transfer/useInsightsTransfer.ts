import { computed, ref, watch, type Ref } from 'vue'
import { getTransferSummary, setTransferVote } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps } from '../../composables/useTransferVotes'
import type { AppSession } from '../../stores/appSession'
import type { TransferCard, TransferSummary, TransferTally, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { applyTally, transferBlockedMessage, transferHint } from '../../utils/transferLinks'

const RETURN_URL = '/pages/insights/index'

export interface InsightsTransferOptions {
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'status' | 'revision' | 'invalidateSession'>
  selectedTeamId: Readonly<Ref<number | null>>
  active: Readonly<Ref<boolean>>
  deps?: {
    summary?: typeof getTransferSummary
    vote?: (code: string, value: 1 | -1 | 0) => Promise<TransferTally>
    login?: (returnUrl: string) => void
    notify?: (message: string) => void
  }
}

/** 洞察页"转会"胶囊：只在激活时加载，跟随页面顶部选中的球队。 */
export function useInsightsTransfer(options: InsightsTransferOptions) {
  const { session, selectedTeamId, active } = options
  const read = options.deps?.summary ?? getTransferSummary
  const notify = options.deps?.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const summary = ref<TransferSummary | null>(null)
  const loading = ref(false), error = ref('')
  const loggedIn = () => session.hasAccessToken.value && session.status.value !== 'loading'
  const votes = createTransferVotes(transferVoteDeps({
    vote: options.deps?.vote ?? setTransferVote,
    loggedIn,
    revision: () => session.revision.value,
    invalidate: revision => session.invalidateSession(revision),
    ...(options.deps?.login ? { login: options.deps.login } : {}),
  }))
  let generation = 0, disposed = false

  const teamId = () => {
    const id = selectedTeamId.value
    return id !== null && Number.isInteger(id) && id > 0 ? id : 0
  }
  const identity = () => `${session.revision.value}:${session.currentUser.value ? 'user' : 'guest'}`
  const enabled = () => !disposed && active.value && Boolean(teamId())

  const eligibility = () => {
    const value = summary.value
    return value ? { loggedIn: loggedIn(), canVote: value.can_vote, open: Boolean(value.window?.open) } : null
  }
  const hint = computed(() => {
    const input = eligibility()
    return input ? transferHint(input) : ''
  })
  /** 提名前的拦截：已登录却不能投时返回原因（不打开提名页），游客与能投的人返回空。 */
  function nominateBlocked(): string {
    const input = eligibility()
    return input ? transferBlockedMessage(input) : ''
  }

  async function refresh(): Promise<void> {
    if (!enabled()) return
    const owner = ++generation, id = teamId(), revision = session.revision.value
    const current = () => enabled() && owner === generation && teamId() === id
    votes.reset()
    loading.value = true; error.value = ''
    if (summary.value?.team.team_id !== id) summary.value = null
    try {
      const result = await read(id)
      if (current()) summary.value = result
    } catch (cause) {
      if (!current()) return
      if (isUnauthorizedError(cause)) session.invalidateSession(revision)
      error.value = extractApiErrorMessage(cause, '转会投票加载失败，请重试')
    } finally {
      if (current()) loading.value = false
    }
  }

  async function vote(card: TransferCard, value: TransferVoteValue): Promise<void> {
    const value_ = summary.value
    if (!value_ || !enabled()) return
    const blocked = nominateBlocked()
    if (blocked) {
      notify(blocked)
      return
    }
    const id = teamId()
    const message = await votes.toggle(card, value, {
      returnUrl: RETURN_URL,
      current: () => enabled() && teamId() === id,
      publish: tally => {
        const target = summary.value
        if (!target) return
        summary.value = { ...target, boards: target.boards.map(board => ({
          ...board, items: board.items.map(item => item.share_code === card.share_code ? applyTally(item, tally) : item),
        })) }
      },
    })
    if (message) notify(message)
  }

  const stop = watch(() => [teamId(), active.value, identity()] as const, () => {
    if (!enabled()) { generation++; loading.value = false; votes.reset(); return }
    void refresh()
  }, { immediate: true })

  function dispose(): void { disposed = true; generation++; stop(); votes.reset() }
  return { summary, loading, error, hint, pending: votes.pending, refresh, vote, nominateBlocked, dispose }
}
