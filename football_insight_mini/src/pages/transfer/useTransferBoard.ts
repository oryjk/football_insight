import { computed, ref } from 'vue'
import { getTransferBoard, setTransferVote } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps, type TransferVoteDeps } from '../../composables/useTransferVotes'
import type { TransferBoard, TransferCard, TransferIntent, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage } from '../../utils/apiError'
import { applyTally, buildTransferBoardUrl, transferBlockedMessage } from '../../utils/transferLinks'

export interface TransferBoardDeps extends Partial<TransferVoteDeps> {
  board?: typeof getTransferBoard
  notify?: (message: string) => void
}

export function createTransferBoard(deps: TransferBoardDeps = {}) {
  const read = deps.board ?? getTransferBoard
  const notify = deps.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const { board: _board, notify: _notify, ...voteOverrides } = deps
  const voteDeps = transferVoteDeps({ vote: setTransferVote, ...voteOverrides })
  const votes = createTransferVotes(voteDeps)
  const teamId = ref(0), intent = ref<TransferIntent>('out')
  const board = ref<TransferBoard | null>(null), items = ref<TransferCard[]>([])
  const loading = ref(false), error = ref('')
  /** 切换意向时保留上一个榜的列表直到新数据回来（页面不塌），期间标记为过期，页面可淡化显示。 */
  const stale = ref(false)
  let generation = 0, disposed = false
  const hasMore = computed(() => !board.value || items.value.length < board.value.total)

  async function fetch(pageNo: number): Promise<void> {
    if (disposed || !teamId.value) return
    const owner = ++generation, id = teamId.value, which = intent.value
    loading.value = true; error.value = ''
    try {
      const result = await read(id, which, pageNo)
      if (owner !== generation || disposed) return
      board.value = result
      stale.value = false
      const known = new Set(pageNo === 1 ? [] : items.value.map(item => item.share_code))
      items.value = [...(pageNo === 1 ? [] : items.value), ...result.items.filter(item => !known.has(item.share_code))]
    } catch (cause) {
      if (owner === generation && !disposed) {
        error.value = extractApiErrorMessage(cause, '转会榜加载失败，请重试')
        // 过期列表属于上一个榜，失败时不能继续冒充当前榜
        if (stale.value) { items.value = []; stale.value = false }
      }
    } finally {
      if (owner === generation) loading.value = false
    }
  }

  /** 首次进入或换球队：清空后加载。 */
  function load(id: number, which: TransferIntent): Promise<void> {
    teamId.value = id; intent.value = which
    board.value = null; items.value = []; stale.value = false; votes.reset()
    return fetch(1)
  }
  /** 返回页面时刷新：保留当前列表，新数据回来再替换。 */
  function refresh(): Promise<void> {
    if (!board.value) return load(teamId.value, intent.value)
    return fetch(1)
  }
  function more(): Promise<void> {
    if (loading.value || stale.value || !hasMore.value || !board.value) return Promise.resolve()
    return fetch(board.value.page + 1)
  }
  function changeIntent(which: TransferIntent): Promise<void> {
    if (which === intent.value && board.value && !error.value) return Promise.resolve()
    if (!board.value) return load(teamId.value, which)
    // 保留标题、说明与旧列表，避免切换时页面先塌再撑开造成抖动。
    intent.value = which
    stale.value = items.value.length > 0
    votes.reset()
    return fetch(1)
  }
  /** 提名或赞踩前的拦截：已登录却不能投时返回原因，游客与能投的人返回空。 */
  function nominateBlocked(): string {
    const current = board.value
    if (!current) return ''
    return transferBlockedMessage({ loggedIn: voteDeps.loggedIn(), canVote: current.can_vote, open: Boolean(current.window?.open) })
  }
  async function vote(card: TransferCard, value: TransferVoteValue): Promise<void> {
    const current = board.value
    if (!current) return
    // 已登录但不能投（非主队或已截止）给提示；游客交给 toggle 去登录。
    const blocked = nominateBlocked()
    if (blocked) {
      notify(blocked)
      return
    }
    const owner = generation
    const message = await votes.toggle(card, value, {
      returnUrl: buildTransferBoardUrl(teamId.value, intent.value),
      current: () => owner === generation && !disposed,
      publish: tally => { items.value = items.value.map(item => item.share_code === card.share_code ? applyTally(item, tally) : item) },
    })
    if (message) notify(message)
  }
  function dispose(): void { disposed = true; generation++; votes.reset() }
  return { teamId, intent, board, items, loading, error, stale, hasMore, pending: votes.pending, load, refresh, more, changeIntent, vote, nominateBlocked, dispose }
}
