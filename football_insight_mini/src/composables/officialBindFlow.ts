import { computed, ref, watch, type Ref } from 'vue'
import { createWechatOfficialBindCode } from '../api/auth'
import type { AppSession } from '../stores/appSession'
import {
  canShowOfficialWechatBindEntry,
  formatOfficialWechatBindCode,
  getOfficialWechatBindRemainingSeconds,
  resolveOfficialWechatBindError,
} from '../utils/officialAccountBinding'

interface OfficialBindDeps {
  createCode(): Promise<{ code: string; expires_in_seconds: number }>
  now(): number
  copy(text: string): void
}

export function createUserOfficialBindFlow(
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'revision' | 'ensureSessionReady'>,
  underReview: Readonly<Ref<boolean>>,
  deps: OfficialBindDeps = {
    createCode: createWechatOfficialBindCode,
    now: () => Date.now(),
    copy: (text) => uni.setClipboardData({ data: text }),
  },
) {
  const visible = ref(false)
  const code = ref('')
  const expiresAt = ref<number | null>(null)
  const remainingSeconds = ref(0)
  const loading = ref(false)
  const error = ref('')
  const canOpen = computed(() => canShowOfficialWechatBindEntry(
    session.hasAccessToken.value && Boolean(session.currentUser.value),
    underReview.value,
    session.currentUser.value?.official_wechat_bound,
  ))
  const actor = computed(() => `${session.revision.value}:${session.currentUser.value?.id || ''}`)
  let owner = ''
  let requestId = 0
  let issuedForActor = ''
  let pageHidden = false
  let refreshOnReturn = false

  function resetSheet(): void {
    requestId += 1
    visible.value = false
    code.value = ''
    expiresAt.value = null
    remainingSeconds.value = 0
    loading.value = false
    error.value = ''
    owner = ''
  }

  // Synchronous invalidation prevents render/copy between the actor change and Vue's next tick.
  const stopWatching = watch([actor, canOpen], () => {
    resetSheet()
    issuedForActor = ''
    refreshOnReturn = false
  }, { flush: 'sync' })

  function isCurrent(): boolean {
    return visible.value && canOpen.value && owner === actor.value
  }

  async function refreshUser(): Promise<void> {
    try {
      // appSession owns request deduplication, publication and session/token guards.
      await session.ensureSessionReady({ force: true })
    } catch {
      // Keep existing account data on transient errors; 401 remains appSession's responsibility.
    }
  }

  async function close(): Promise<void> {
    const shouldRefresh = isCurrent()
    resetSheet()
    if (shouldRefresh) await refreshUser()
  }

  function open(): void {
    resetSheet()
    if (!canOpen.value) return
    owner = actor.value
    visible.value = true
  }

  function refreshCountdown(): void {
    remainingSeconds.value = getOfficialWechatBindRemainingSeconds(expiresAt.value, deps.now())
  }

  async function getCode(): Promise<void> {
    if (!isCurrent() || loading.value) return
    const id = ++requestId
    const startedAt = deps.now()
    const stillCurrent = () => id === requestId && isCurrent()
    loading.value = true
    error.value = ''
    code.value = ''
    expiresAt.value = null
    remainingSeconds.value = 0
    try {
      const result = await deps.createCode()
      if (!stillCurrent()) return
      const normalizedCode = formatOfficialWechatBindCode(result.code)
      if (!normalizedCode || !Number.isFinite(result.expires_in_seconds) || result.expires_in_seconds <= 0) {
        throw new Error('Invalid binding code response')
      }
      code.value = normalizedCode
      // Count from request start, conservatively including network latency in the expiry.
      expiresAt.value = startedAt + result.expires_in_seconds * 1000
      refreshCountdown()
      // Closing the sheet preserves this marker: the copied command can still be used.
      issuedForActor = actor.value
      refreshOnReturn = pageHidden
    } catch (cause) {
      if (stillCurrent()) error.value = resolveOfficialWechatBindError(cause)
    } finally {
      if (stillCurrent()) loading.value = false
    }
  }

  function copyCode(): void {
    if (!isCurrent()) return
    refreshCountdown()
    if (!code.value || remainingSeconds.value <= 0) return
    deps.copy(`绑定 ${code.value}`)
  }

  function dispose(): void {
    stopWatching()
    resetSheet()
    issuedForActor = ''
    refreshOnReturn = false
  }

  function handlePageHide(): void {
    pageHidden = true
    refreshOnReturn = canOpen.value && issuedForActor === actor.value
  }

  async function handlePageShow(): Promise<void> {
    pageHidden = false
    if (!refreshOnReturn) return
    const shouldRefresh = canOpen.value && issuedForActor === actor.value
    // Consume before awaiting: overlapping onShow events cannot repeat the refresh.
    refreshOnReturn = false
    issuedForActor = ''
    if (!shouldRefresh) return
    await refreshUser()
  }

  return {
    visible, code, expiresAt, remainingSeconds, loading, error, canOpen,
    open, close, getCode, copyCode, refreshCountdown, dispose,
    handlePageHide, handlePageShow,
  }
}
