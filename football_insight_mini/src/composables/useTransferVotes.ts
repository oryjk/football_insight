import { ref } from 'vue'
import { setTransferVote } from '../api/transfer'
import { useAppSession } from '../stores/appSession'
import type { TransferCard, TransferTally, TransferVoteValue } from '../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../utils/apiError'
import { rememberPostLoginRedirect } from '../utils/postLoginRedirect'

export interface TransferVoteDeps {
  vote: (code: string, value: 1 | -1 | 0) => Promise<TransferTally>
  loggedIn: () => boolean
  revision: () => number
  invalidate: (revision: number) => void
  login: (returnUrl: string) => void
}

export function transferVoteDeps(overrides: Partial<TransferVoteDeps> = {}): TransferVoteDeps {
  const session = overrides.revision ? null : useAppSession()
  return {
    vote: setTransferVote,
    // 登录过程中旧 token 可能还在，此时不发写请求。
    loggedIn: () => Boolean(session?.hasAccessToken.value) && session?.status.value !== 'loading',
    revision: () => session?.revision.value ?? 0,
    invalidate: revision => session?.invalidateSession(revision),
    login: returnUrl => {
      rememberPostLoginRedirect({ type: returnUrl === '/pages/insights/index' ? 'switchTab' : 'navigateTo', url: returnUrl })
      uni.showToast({ title: '请登录后继续', icon: 'none' })
      uni.switchTab({ url: '/pages/user/index' })
    },
    ...overrides,
  }
}

export interface ToggleOptions {
  returnUrl: string
  /** 调用方上下文（球队、页面）是否仍有效 */
  current: () => boolean
  publish: (tally: TransferTally) => void
}

/** 赞踩：点同一侧取消，点另一侧改投；只发布服务端确认的结果，失败保留原状态。 */
export function createTransferVotes(deps: TransferVoteDeps) {
  const pending = ref<string[]>([])
  let generation = 0
  function reset(): void { generation++; pending.value = [] }
  async function toggle(card: TransferCard, value: TransferVoteValue, options: ToggleOptions): Promise<string> {
    const code = card.share_code
    if (pending.value.includes(code)) return ''
    if (!deps.loggedIn()) { deps.login(options.returnUrl); return '' }
    const owner = generation, revision = deps.revision()
    const valid = () => options.current() && generation === owner && deps.revision() === revision
    pending.value = [...pending.value, code]
    try {
      const tally = await deps.vote(code, card.my_vote === value ? 0 : value)
      if (valid()) options.publish(tally)
      return ''
    } catch (error) {
      if (!valid()) return ''
      if (isUnauthorizedError(error)) { deps.invalidate(revision); deps.login(options.returnUrl); return '' }
      return extractApiErrorMessage(error, '投票失败，请重试')
    } finally {
      if (owner === generation) pending.value = pending.value.filter(item => item !== code)
    }
  }
  return { pending, toggle, reset }
}
