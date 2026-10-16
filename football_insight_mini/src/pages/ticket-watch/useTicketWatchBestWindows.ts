import { onScopeDispose, ref, watch, type Ref } from 'vue'
import { getTicketWatchBestWindows } from '../../api/ticketWatch'
import type { TicketWatchBestWindowsResponse } from '../../types/ticketWatch'
import { createLatestGuard } from '../../utils/latestGuard'

const SUCCESS_CACHE_MS = 10 * 60 * 1000
interface BestWindowsOptions {
  revision: Readonly<Ref<number>>
  underReview: Readonly<Ref<boolean>>
  fetch?: () => Promise<TicketWatchBestWindowsResponse>
  now?: () => number
}

/** Session-owned statistics: never coupled to match selection or inventory polling. */
export function useTicketWatchBestWindows(options: BestWindowsOptions) {
  const data = ref<TicketWatchBestWindowsResponse | null>(null)
  const loading = ref(false)
  const fetch = options.fetch ?? getTicketWatchBestWindows
  const now = options.now ?? Date.now
  const guard = createLatestGuard(() => [options.revision.value, options.underReview.value])
  let active = false
  let disposed = false
  let successfulAt: number | null = null
  let pending: Promise<void> | null = null

  function invalidate(clearData: boolean) {
    guard.invalidate()
    pending = null
    loading.value = false
    if (clearData) {
      data.value = null
      successfulAt = null
    }
  }

  function load(): Promise<void> {
    if (!active || disposed || options.underReview.value) return Promise.resolve()
    if (pending) return pending
    if (successfulAt !== null && now() - successfulAt < SUCCESS_CACHE_MS) return Promise.resolve()
    const owner = guard.begin()
    loading.value = true
    pending = (async () => {
      try {
        // Session revision is published before logout/invalidation clears token storage.
        // Clear private state synchronously, but let that credential mutation finish first.
        await Promise.resolve()
        if (!active || disposed || !owner.isCurrent()) return
        const result = await fetch()
        if (!active || disposed || !owner.isCurrent()) return
        data.value = result.status === 'ok' ? result : null
        successfulAt = now()
      } catch {
        if (active && !disposed && owner.isCurrent()) data.value = null
      } finally {
        if (owner.owns()) {
          loading.value = false
          pending = null
        }
      }
    })()
    return pending
  }

  watch(options.revision, () => {
    invalidate(true)
    void load()
  }, { flush: 'sync' })
  watch(options.underReview, (review) => {
    invalidate(true)
    if (!review) void load()
  }, { flush: 'sync' })
  onScopeDispose(() => {
    disposed = true
    active = false
    invalidate(true)
  })

  return {
    data,
    loading,
    show() { active = true; return load() },
    hide() { active = false; invalidate(false) },
  }
}
