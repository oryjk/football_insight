import { describe, expect, test } from 'bun:test'

import {
  buildCurrentTicketWatchBoardUrl,
  buildYukunTicketWatchInventoryUrl,
  buildYukunTicketWatchRegionsUrl,
  buildTicketWatchInterestsUrl,
  buildTicketWatchInventoryUrl,
} from './ticketWatch'

function captureErrorMessage(action: () => unknown): string {
  try {
    action()
  } catch (error) {
    return error instanceof Error ? error.message : String(error)
  }

  return ''
}

describe('buildTicketWatchInventoryUrl', () => {
  test('appends since and fallback match id for legacy history lookups', () => {
    expect(buildTicketWatchInventoryUrl(570, '2026-03-06T14:10:00+08:00', 72)).toBe(
      '/ticket-watch/matches/570/inventory?since=2026-03-06T14%3A10%3A00%2B08%3A00&fallback_match_id=72',
    )
  })

  test('omits fallback match id when it matches the primary id', () => {
    expect(buildTicketWatchInventoryUrl(572, '2026-04-03T14:10:00+08:00', 572)).toBe(
      '/ticket-watch/matches/572/inventory?since=2026-04-03T14%3A10%3A00%2B08%3A00',
    )
  })

  test('rejects missing since so inventory never falls back to all records', () => {
    expect(captureErrorMessage(() => buildTicketWatchInventoryUrl(572, null, 572))).toBe(
      'sale_start_at is required to build inventory since; refusing full inventory query',
    )
    expect(captureErrorMessage(() => buildTicketWatchInventoryUrl(572, '   ', 572))).toBe(
      'sale_start_at is required to build inventory since; refusing full inventory query',
    )
  })
})

describe('buildTicketWatchInterestsUrl', () => {
  test('builds the block interest endpoint for the selected match', () => {
    expect(buildTicketWatchInterestsUrl(572)).toBe('/ticket-watch/matches/572/interests')
  })
})

describe('buildCurrentTicketWatchBoardUrl', () => {
  test('builds the aggregated current board endpoint', () => {
    expect(buildCurrentTicketWatchBoardUrl()).toBe('/ticket-watch/current-board')
  })
})

describe('buildYukunTicketWatchInventoryUrl', () => {
  test('requires sale-start based since for yukun inventory and regions', () => {
    expect(buildYukunTicketWatchInventoryUrl(288651, '2026-05-04T14:10:00+08:00')).toBe(
      '/ticket-watch/yukun/matches/288651/inventory?since=2026-05-04T14%3A10%3A00%2B08%3A00',
    )
    expect(buildYukunTicketWatchRegionsUrl(288651, '2026-05-04T14:10:00+08:00')).toBe(
      '/ticket-watch/yukun/matches/288651/regions?since=2026-05-04T14%3A10%3A00%2B08%3A00',
    )
    expect(captureErrorMessage(() => buildYukunTicketWatchInventoryUrl(288651, null))).toBe(
      'sale_start_at is required to build inventory since; refusing full inventory query',
    )
  })
})
