import { ref } from 'vue'
import { getTransferCandidates, nominateTransfer } from '../../api/transfer'
import { transferVoteDeps } from '../../composables/useTransferVotes'
import type { TransferCandidateGroup, TransferIntent, TransferNominateResult } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { buildTransferBoardUrl } from '../../utils/transferLinks'

export interface TransferNominateDeps {
  candidates?: typeof getTransferCandidates
  nominate?: typeof nominateTransfer
  loggedIn?: () => boolean
  login?: (returnUrl: string) => void
  notify?: (message: string) => void
}

export function createTransferNominate(deps: TransferNominateDeps = {}) {
  const session = transferVoteDeps({
    ...(deps.loggedIn ? { loggedIn: deps.loggedIn, revision: () => 0, invalidate: () => {} } : {}),
    ...(deps.login ? { login: deps.login } : {}),
  })
  const readCandidates = deps.candidates ?? getTransferCandidates
  const write = deps.nominate ?? nominateTransfer
  const notify = deps.notify ?? ((title: string) => uni.showToast({ title, icon: 'none' }))
  const visible = ref(false), intent = ref<TransferIntent>('out')
  const groups = ref<TransferCandidateGroup[]>([]), loading = ref(false), error = ref(''), submitting = ref(false)
  const cache = new Map<string, TransferCandidateGroup[]>()
  let teamId = 0, generation = 0

  async function loadGroups(): Promise<void> {
    const key = `${teamId}:${intent.value}`
    const cached = cache.get(key)
    if (cached) { groups.value = cached; error.value = ''; return }
    const owner = ++generation
    loading.value = true; error.value = ''; groups.value = []
    try {
      const result = await readCandidates(teamId, intent.value)
      cache.set(key, result.groups)
      if (owner === generation) groups.value = result.groups
    } catch (cause) {
      if (owner === generation) error.value = extractApiErrorMessage(cause, '候选球员加载失败，请重试')
    } finally {
      if (owner === generation) loading.value = false
    }
  }

  async function open(id: number, which: TransferIntent): Promise<void> {
    teamId = id; intent.value = which
    if (!session.loggedIn()) { session.login(buildTransferBoardUrl(id, which, { nominate: true })); return }
    visible.value = true
    await loadGroups()
  }
  function changeIntent(which: TransferIntent): Promise<void> {
    intent.value = which
    return loadGroups()
  }
  function retry(): Promise<void> { cache.delete(`${teamId}:${intent.value}`); return loadGroups() }

  async function submit(playerId: number): Promise<TransferNominateResult | null> {
    if (submitting.value) return null
    submitting.value = true
    try {
      const result = await write(teamId, playerId, intent.value)
      notify(result.created ? '提名成功' : '已有球迷提名，已为你加一票')
      visible.value = false
      return result
    } catch (cause) {
      if (isUnauthorizedError(cause)) { visible.value = false; session.login(buildTransferBoardUrl(teamId, intent.value, { nominate: true })); return null }
      notify(extractApiErrorMessage(cause, '提名失败，请重试'))
      return null
    } finally {
      submitting.value = false
    }
  }
  return { visible, intent, groups, loading, error, submitting, open, changeIntent, retry, submit }
}
