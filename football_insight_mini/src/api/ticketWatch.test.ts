import { describe, expect, test } from 'bun:test'

import {
  buildCurrentTicketWatchBoardUrl,
  buildTicketWatchInterestsUrl,
  buildTicketWatchInventoryUrl,
} from './ticketWatch'

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
