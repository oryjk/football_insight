import { reportPaywallEvents } from '../api/paywall'
import { MINI_PROGRAM_VERSION } from '../api/system'
import { useAppSession } from '../stores/appSession'
import { getAccessToken } from '../utils/authStorage'
import type { PaywallContext, PaywallEventPayload, PaywallSurface } from '../types/paywall'

const FLUSH_DELAY_MS = 500
const TAP_MERGE_WINDOW_MS = 2_000
const MAX_EVENTS_PER_REQUEST = 10

interface TrackerOptions {
  send: (events: PaywallEventPayload[]) => Promise<unknown> | void
  getContext: () => PaywallContext
  isLoggedIn: () => boolean
  now?: () => number
  schedule?: (flush: () => void, delayMs: number) => void
}

export interface PaywallTracker {
  /** Call on each page show: a surface may report one impression per visit. */
  beginVisit(): void
  impression(surface: PaywallSurface, extra?: { matchId?: number }): void
  tap(surface: PaywallSurface, extra?: { matchId?: number }): void
}

/**
 * Paywall measurement is best-effort and must never affect the page: every failure is swallowed,
 * guests are skipped, impressions are deduplicated per visit and rapid repeat taps are merged.
 */
export function createPaywallTracker(options: TrackerOptions): PaywallTracker {
  const now = options.now ?? Date.now
  const schedule = options.schedule ?? ((flush, delayMs) => { setTimeout(flush, delayMs) })
  const seenImpressions = new Set<PaywallSurface>()
  const lastTapAt = new Map<PaywallSurface, number>()
  let queue: PaywallEventPayload[] = []
  let scheduled = false

  const flush = () => {
    scheduled = false
    const events = queue
    queue = []
    for (let start = 0; start < events.length; start += MAX_EVENTS_PER_REQUEST) {
      const batch = events.slice(start, start + MAX_EVENTS_PER_REQUEST)
      try {
        void Promise.resolve(options.send(batch)).catch(() => { /* measurement only */ })
      } catch { /* measurement only */ }
    }
  }

  const enqueue = (surface: PaywallSurface, action: PaywallEventPayload['action'], extra?: { matchId?: number }) => {
    try {
      if (!options.isLoggedIn()) return
      const context = options.getContext()
      const detail: Record<string, string | number> = {}
      if (context.miniVersion) detail.mini_version = context.miniVersion
      if (context.platform) detail.platform = context.platform
      if (typeof extra?.matchId === 'number' && extra.matchId > 0) detail.match_id = extra.matchId
      const event: PaywallEventPayload = { surface, action, detail }
      if (context.membershipTier) event.membership_tier = context.membershipTier
      queue.push(event)
      if (!scheduled) {
        scheduled = true
        schedule(flush, FLUSH_DELAY_MS)
      }
    } catch { /* measurement only */ }
  }

  return {
    beginVisit() {
      seenImpressions.clear()
    },
    impression(surface, extra) {
      if (seenImpressions.has(surface)) return
      seenImpressions.add(surface)
      enqueue(surface, 'impression', extra)
    },
    tap(surface, extra) {
      const at = now()
      const previous = lastTapAt.get(surface)
      if (previous !== undefined && at - previous < TAP_MERGE_WINDOW_MS) return
      lastTapAt.set(surface, at)
      enqueue(surface, 'tap', extra)
    },
  }
}

function readPlatform(): string | undefined {
  try {
    const platform = uni.getSystemInfoSync().platform
    return typeof platform === 'string' && platform ? platform : undefined
  } catch {
    return undefined
  }
}

function accessToken(): string | null {
  try { return getAccessToken() } catch { return null }
}

const sharedTracker = createPaywallTracker({
  send: (events) => reportPaywallEvents({ events }),
  getContext: () => ({
    membershipTier: useAppSession().currentUser.value?.membership_tier ?? null,
    miniVersion: MINI_PROGRAM_VERSION,
    platform: readPlatform(),
  }),
  isLoggedIn: () => Boolean(accessToken()),
})

export const beginPaywallVisit = (): void => sharedTracker.beginVisit()
export const reportPaywallImpression = (surface: PaywallSurface, extra?: { matchId?: number }): void => sharedTracker.impression(surface, extra)
export const reportPaywallTap = (surface: PaywallSurface, extra?: { matchId?: number }): void => sharedTracker.tap(surface, extra)
