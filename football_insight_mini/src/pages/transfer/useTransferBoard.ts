import { computed, ref } from 'vue'
import { getTransferBoard, setTransferVote } from '../../api/transfer'
import { createTransferVotes, transferVoteDeps, type TransferVoteDeps } from '../../composables/useTransferVotes'
import type { TransferBoard, TransferCard, TransferIntent, TransferVoteValue } from '../../types/transfer'
import { extractApiErrorMessage } from '../../utils/apiError'
import { applyTally, buildTransferBoardUrl } from '../../utils/transferLinks'

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
      const known = new Set(pageNo === 1 ? [] : items.value.map(item => item.share_code))
      items.value = [...(pageNo === 1 ? [] : items.value), ...result.items.filter(item => !known.has(item.share_code))]
    } catch (cause) {
      if (owner === generation && !disposed) error.value = extractApiErrorMessage(cause, '转会榜加载失败，请重试')
    } finally {
      if (owner === generation) loading.value = false
    }
  }

  function load(id: number, which: TransferIntent): Promise<void> {
    teamId.value = id; intent.value = which
    board.value = null; items.value = []; votes.reset()
    return fetch(1)
  }
  function more(): Promise<void> {
    if (loading.value || !hasMore.value || !board.value) return Promise.resolve()
    return fetch(board.value.page + 1)
  }
  function changeIntent(which: TransferIntent): Promise<void> {
    if (which === intent.value && board.value && !error.value) return Promise.resolve()
    return load(teamId.value, which)
  }
  async function vote(card: TransferCard, value: TransferVoteValue): Promise<void> {
    const current = board.value
    if (!current) return
    // 已登录但不能投（非主队或已截止）给提示；游客交给 toggle 去登录。
    if (voteDeps.loggedIn() && !current.can_vote) {
      notify(current.window?.open ? '只能为你的主队投票' : '本期投票已结束')
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
  return { teamId, intent, board, items, loading, error, hasMore, pending: votes.pending, load, more, changeIntent, vote, dispose }
}
