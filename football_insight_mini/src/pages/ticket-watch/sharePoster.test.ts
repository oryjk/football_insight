import { expect, test } from 'bun:test'
import type { TicketWatchGroupedInventorySection, TicketWatchMatchSummary } from '../../types/ticketWatch'
import { buildTicketPosterContent, formatTicketPosterCutoff } from './sharePoster'

export const posterMatch: TicketWatchMatchSummary = {
  match_id: 99, external_match_id: 'private-match-id', round_number: 19,
  sale_start_at: '2026-10-19T10:00:00+08:00', match_date: '2026-10-21', match_time: '19:35',
  kickoff_at: '2026-10-21T19:35:00+08:00', home_team_name: '成都蓉城', away_team_name: '云南玉昆', is_current: true,
}
export const posterSections: TicketWatchGroupedInventorySection[] = [{
  price: '180', region_count: 3, available_region_count: 3, total_occurrences: 999999,
  items: [
    { block_name: 'A1', price: '180', occurrences: 23, latest_time: '2026-10-19T10:12:00+08:00', has_inventory: true, interested_user_count: 19, viewer_interested: true },
    { block_name: 'A2', price: '180', occurrences: 7, latest_time: '2026-10-19T10:11:00+08:00', has_inventory: true, interested_user_count: 20, viewer_interested: false },
    { block_name: '开售峰值', price: '180', occurrences: 10000, latest_time: '2026-10-19T10:09:59+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
  ],
}]

for (const mode of ['current', 'history'] as const) {
  for (const tier of [null, 'V1', 'V2', 'V3', 'V4', 'V5', 'V6', 'V7', 'V8', 'V9']) {
    test(`${mode} poster is identical for ${tier ?? 'guest'} and contains only public grid fields`, () => {
      const input = { mode, match: posterMatch, sections: posterSections, loadedAtMs: Date.parse('2026-10-19T10:13:24+08:00') }
      const content = buildTicketPosterContent({ ...input, viewer: tier ? { membership_tier: tier } : null })!
      expect(content).toEqual(buildTicketPosterContent({ ...input, viewer: null }))
      expect(content.totalOccurrences).toBe(30)
      expect(content.regions).toEqual([{ name: 'A1', price: '180', occurrences: 23 }, { name: 'A2', price: '180', occurrences: 7 }])
      expect(Object.keys(content).sort()).toEqual(['mode', 'teams', 'matchLabel', 'totalOccurrences', 'regionCount', 'regions', 'cutoffLabel', 'disclaimer'].sort())
      const serialized = JSON.stringify(content)
      for (const privateField of ['membership', 'user_id', 'phone', 'nickname', 'latest_time', 'interested', 'minutes_ago', 'private-match-id']) expect(serialized.includes(privateField)).toBe(false)
      expect(content.cutoffLabel).toBe(mode === 'current' ? '2026-10-19 10:13:24' : null)
    })
  }
}

test('poster refuses empty data, unknown sale start and unknown current data time', () => {
  expect(buildTicketPosterContent({ mode: 'history', match: null, sections: posterSections })).toBeNull()
  for (const sale_start_at of [null, '', 'invalid']) {
    expect(buildTicketPosterContent({ mode: 'history', match: { ...posterMatch, sale_start_at }, sections: posterSections })).toBeNull()
  }
  expect(buildTicketPosterContent({ mode: 'history', match: posterMatch, sections: [] })).toBeNull()
  expect(buildTicketPosterContent({ mode: 'current', match: posterMatch, sections: posterSections })).toBeNull()
})

test('current data includes second precision Beijing cutoff and explicit expiry warning', () => {
  const content = buildTicketPosterContent({ mode: 'current', match: posterMatch, sections: posterSections, loadedAtMs: Date.parse('2026-10-19T23:59:59Z') })!
  expect(formatTicketPosterCutoff(Date.parse('2026-10-19T23:59:59Z'))).toBe('2026-10-20 07:59:59')
  expect(content.disclaimer).toBe('实时库存变化很快，请以小程序为准')
})
