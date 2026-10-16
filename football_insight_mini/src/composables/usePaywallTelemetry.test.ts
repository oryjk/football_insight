import { describe, expect, test } from 'bun:test'

import { createPaywallTracker } from './usePaywallTelemetry'
import type { PaywallEventPayload } from '../types/paywall'

function harness(overrides: { loggedIn?: boolean; throwOnSend?: boolean; rejectOnSend?: boolean } = {}) {
  const sent: PaywallEventPayload[][] = []
  const timers: Array<() => void> = []
  let clock = 1_000
  const tracker = createPaywallTracker({
    send: (events) => {
      if (overrides.throwOnSend) throw new Error('boom')
      sent.push(events)
      return overrides.rejectOnSend ? Promise.reject(new Error('network')) : Promise.resolve()
    },
    getContext: () => ({ membershipTier: 'V3', miniVersion: '1.0.78', platform: 'android' }),
    isLoggedIn: () => overrides.loggedIn ?? true,
    now: () => clock,
    schedule: (fn) => { timers.push(fn) },
  })
  return {
    tracker,
    sent,
    flush: () => { while (timers.length) timers.shift()!() },
    advance: (ms: number) => { clock += ms },
  }
}

describe('paywall tracker', () => {
  test('records an impression with tier and a small detail, flushed as one batch', () => {
    const { tracker, sent, flush } = harness()
    tracker.beginVisit()
    tracker.impression('ticket_watch_recent_reflux_lock')
    tracker.impression('insights_ticket_watch_entry')
    expect(sent.length).toBe(0)
    flush()
    expect(sent.length).toBe(1)
    expect(sent[0]).toEqual([
      { surface: 'ticket_watch_recent_reflux_lock', action: 'impression', membership_tier: 'V3', detail: { mini_version: '1.0.78', platform: 'android' } },
      { surface: 'insights_ticket_watch_entry', action: 'impression', membership_tier: 'V3', detail: { mini_version: '1.0.78', platform: 'android' } },
    ])
  })

  test('one impression per surface per visit; a new visit may report it again', () => {
    const { tracker, sent, flush } = harness()
    tracker.beginVisit()
    tracker.impression('user_membership_entry')
    tracker.impression('user_membership_entry')
    flush()
    expect(sent.flat().length).toBe(1)
    tracker.beginVisit()
    tracker.impression('user_membership_entry')
    flush()
    expect(sent.flat().length).toBe(2)
  })

  test('taps within two seconds on the same surface merge; other surfaces and later taps count', () => {
    const { tracker, sent, flush, advance } = harness()
    tracker.tap('user_membership_entry')
    advance(1_500)
    tracker.tap('user_membership_entry')
    tracker.tap('ticket_watch_match_id_sheet')
    advance(1_000)
    tracker.tap('user_membership_entry')
    flush()
    expect(sent.flat().map(e => e.surface)).toEqual(['user_membership_entry', 'ticket_watch_match_id_sheet', 'user_membership_entry'])
  })

  test('guests are never reported', () => {
    const { tracker, sent, flush } = harness({ loggedIn: false })
    tracker.beginVisit()
    tracker.impression('user_membership_entry')
    tracker.tap('user_membership_entry')
    flush()
    expect(sent.length).toBe(0)
  })

  test('a send that throws or rejects never surfaces to the page', async () => {
    for (const overrides of [{ throwOnSend: true }, { rejectOnSend: true }]) {
      const { tracker, flush } = harness(overrides)
      tracker.beginVisit()
      let threw = false
      try {
        tracker.impression('user_membership_entry')
        flush()
        await Promise.resolve()
      } catch {
        threw = true
      }
      expect(threw).toBe(false)
    }
  })

  test('extra match id is included when provided', () => {
    const { tracker, sent, flush } = harness()
    tracker.tap('ticket_watch_match_id_sheet', { matchId: 583 })
    flush()
    expect(sent[0][0].detail).toEqual({ mini_version: '1.0.78', platform: 'android', match_id: 583 })
  })

  test('batches larger than ten are split into several sends', () => {
    const { tracker, sent, flush, advance } = harness()
    for (let i = 0; i < 12; i += 1) {
      advance(3_000)
      tracker.tap('user_membership_entry')
    }
    flush()
    expect(sent.map(batch => batch.length)).toEqual([10, 2])
  })
})
