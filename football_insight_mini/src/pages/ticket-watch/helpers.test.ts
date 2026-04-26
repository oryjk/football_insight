import { describe, expect, test } from 'bun:test'
import type {
  TicketWatchInventoryEntry,
  TicketWatchMatchSummary,
  TicketWatchRegion,
} from '../../types/ticketWatch'
import {
  applyBlockInterestToSections,
  applyBlockInterestsToSections,
  buildRecentRefluxBuckets,
  buildInventorySince,
  buildTrackedInterestSummary,
  formatTicketWatchMembershipBadgeTier,
  formatTicketWatchPollIntervalLabel,
  formatTrackedInterestTime,
  formatTrackedInterestWaitLabel,
  formatInventorySectionSummary,
  getInventoryEmptyStateLabel,
  isVipInventoryBlock,
  normalizeTicketWatchPollIntervalSeconds,
  prioritizeInventorySections,
  resolveInventoryPriceTone,
  resolveInventoryHeatLevel,
  resolveInventoryRefluxFreshness,
  isRecentRefluxBucketUnlocked,
  summarizeInventoryBoard,
  isTicketWatchSectionCollapsed,
  toggleTicketWatchSectionCollapsed,
  groupInventoryByPrice,
  resolveCurrentBoardLoadStrategy,
  resolveHistoryBoardLoadStrategy,
  resolveBlockInterestHeatLevel,
  resolveRecentRefluxPanelMode,
  selectCompletedMatches,
} from './helpers'

describe('ticket watch polling', () => {
  test('normalizes backend-provided polling intervals', () => {
    expect(normalizeTicketWatchPollIntervalSeconds(600)).toBe(600)
    expect(normalizeTicketWatchPollIntervalSeconds(45)).toBe(45)
    expect(normalizeTicketWatchPollIntervalSeconds(0)).toBe(600)
    expect(normalizeTicketWatchPollIntervalSeconds(undefined)).toBe(600)
  })

  test('formats a user-facing refresh label from backend polling seconds', () => {
    expect(formatTicketWatchPollIntervalLabel(300)).toBe('5 分钟刷新')
    expect(formatTicketWatchPollIntervalLabel(60)).toBe('1 分钟刷新')
    expect(formatTicketWatchPollIntervalLabel(45)).toBe('45 秒刷新')
    expect(formatTicketWatchPollIntervalLabel(1)).toBe('1 秒刷新')
  })

  test('formats the membership badge tier copy', () => {
    expect(formatTicketWatchMembershipBadgeTier('V3')).toBe('V3 会员')
    expect(formatTicketWatchMembershipBadgeTier(undefined)).toBe('V1 会员')
  })
})

describe('resolveCurrentBoardLoadStrategy', () => {
  test('uses blocking loading only for the first initial load', () => {
    expect(resolveCurrentBoardLoadStrategy(false, 'initial')).toEqual({
      showBlockingLoading: true,
      clearErrorBeforeLoad: true,
    })

    expect(resolveCurrentBoardLoadStrategy(true, 'initial')).toEqual({
      showBlockingLoading: false,
      clearErrorBeforeLoad: true,
    })
  })

  test('keeps polling silent so the page does not remount while scrolling', () => {
    expect(resolveCurrentBoardLoadStrategy(true, 'poll')).toEqual({
      showBlockingLoading: false,
      clearErrorBeforeLoad: false,
    })
  })
})

describe('resolveHistoryBoardLoadStrategy', () => {
  test('uses blocking loading only for the first history load', () => {
    expect(resolveHistoryBoardLoadStrategy(false, 'initial')).toEqual({
      showBlockingLoading: true,
      clearErrorBeforeLoad: true,
    })

    expect(resolveHistoryBoardLoadStrategy(true, 'initial')).toEqual({
      showBlockingLoading: false,
      clearErrorBeforeLoad: true,
    })
  })

  test('keeps selection switching silent so chips and cards stay mounted', () => {
    expect(resolveHistoryBoardLoadStrategy(true, 'selection')).toEqual({
      showBlockingLoading: false,
      clearErrorBeforeLoad: false,
    })
  })
})

describe('selectCompletedMatches', () => {
  test('filters to completed matches and sorts latest first', () => {
    const matches: TicketWatchMatchSummary[] = [
      {
        match_id: 288601,
        external_match_id: '288601',
        round_number: 7,
        match_date: '2026-04-12',
        match_time: '19:35',
        kickoff_at: '2026-04-12T19:35:00+08:00',
        home_team_name: '北京国安',
        away_team_name: '山东泰山',
        is_current: false,
      },
      {
        match_id: 288600,
        external_match_id: '288600',
        round_number: 6,
        match_date: '2026-04-01',
        match_time: '19:35',
        kickoff_at: '2026-04-01T19:35:00+08:00',
        home_team_name: '成都蓉城',
        away_team_name: '上海申花',
        is_current: false,
      },
    ]

    const result = selectCompletedMatches(matches, '2026-04-10T12:00:00+08:00')

    expect(result.map((item) => item.match_id)).toEqual([288600])
  })
})

describe('groupInventoryByPrice', () => {
  test('groups regions by price and marks zones with inventory', () => {
    const regions: TicketWatchRegion[] = [
      { block_name: 'A1', price: '380', usable_count: 0, estate: 0 },
      { block_name: 'A2', price: '380', usable_count: 0, estate: 0 },
      { block_name: 'B1', price: '580', usable_count: 0, estate: 0 },
    ]
    const inventory: TicketWatchInventoryEntry[] = [
      { block_name: 'A2', occurrences: 3, latest_time: '2026-04-10T11:59:00+08:00' },
      { block_name: 'B1', occurrences: 1, latest_time: '2026-04-10T11:58:00+08:00' },
    ]

    const result = groupInventoryByPrice(regions, inventory)

    expect(result.length).toBe(2)
    expect(result[0].price).toBe('380')
    expect(result[0].available_region_count).toBe(1)
    expect(result[0].total_occurrences).toBe(3)
    expect(result[0].items[1].has_inventory).toBe(true)
    expect(result[0].items[1].interested_user_count).toBe(0)
    expect(result[1].price).toBe('580')
  })
})

describe('applyBlockInterestsToSections', () => {
  test('merges block interest counts into the grouped inventory cards', () => {
    const sections = applyBlockInterestsToSections(
      [
        {
          price: '180',
          region_count: 2,
          available_region_count: 1,
          total_occurrences: 4,
          items: [
            {
              block_name: '102',
              price: '180',
              occurrences: 4,
              latest_time: '2026-04-10T20:00:00+08:00',
              has_inventory: true,
              interested_user_count: 0,
              viewer_interested: false,
            },
            {
              block_name: '103',
              price: '180',
              occurrences: 0,
              latest_time: '',
              has_inventory: false,
              interested_user_count: 0,
              viewer_interested: false,
            },
          ],
        },
      ],
      [
        {
          block_name: '102',
          interested_user_count: 3,
          viewer_interested: true,
        },
      ],
    )

    expect(sections[0].items[0].interested_user_count).toBe(3)
    expect(sections[0].items[0].viewer_interested).toBe(true)
    expect(sections[0].items[1].interested_user_count).toBe(0)
  })
})

describe('applyBlockInterestToSections', () => {
  test('updates one block card after toggling interest without rebuilding the whole board', () => {
    const sections = applyBlockInterestToSections(
      [
        {
          price: '180',
          region_count: 2,
          available_region_count: 1,
          total_occurrences: 4,
          items: [
            {
              block_name: '102',
              price: '180',
              occurrences: 4,
              latest_time: '2026-04-10T20:00:00+08:00',
              has_inventory: true,
              interested_user_count: 1,
              viewer_interested: false,
            },
            {
              block_name: '103',
              price: '180',
              occurrences: 0,
              latest_time: '',
              has_inventory: false,
              interested_user_count: 0,
              viewer_interested: false,
            },
          ],
        },
      ],
      {
        block_name: '102',
        interested_user_count: 2,
        viewer_interested: true,
      },
    )

    expect(sections[0].items[0].interested_user_count).toBe(2)
    expect(sections[0].items[0].viewer_interested).toBe(true)
    expect(sections[0].items[1].interested_user_count).toBe(0)
  })
})

describe('resolveBlockInterestHeatLevel', () => {
  test('maps interested users into four user-facing heat bars', () => {
    expect(resolveBlockInterestHeatLevel(0)).toBe(0)
    expect(resolveBlockInterestHeatLevel(1)).toBe(1)
    expect(resolveBlockInterestHeatLevel(13)).toBe(2)
    expect(resolveBlockInterestHeatLevel(26)).toBe(3)
    expect(resolveBlockInterestHeatLevel(50)).toBe(4)
    expect(resolveBlockInterestHeatLevel(80)).toBe(4)
  })
})

describe('tracked block interests', () => {
  test('builds a summary for the current viewer tracked blocks', () => {
    const summary = buildTrackedInterestSummary([
      {
        block_name: '104',
        started_at: '2026-04-19T19:35:00+08:00',
        first_inventory_at: '2026-04-19T19:43:00+08:00',
      },
      {
        block_name: '125',
        started_at: '2026-04-19T19:36:00+08:00',
        first_inventory_at: null,
      },
    ])

    expect(summary.total).toBe(2)
    expect(summary.hitCount).toBe(1)
    expect(summary.pendingCount).toBe(1)
  })

  test('formats the first reflux wait label from started and first inventory timestamps', () => {
    expect(
      formatTrackedInterestWaitLabel(
        '2026-04-19T19:35:00+08:00',
        '2026-04-19T19:43:00+08:00',
      ),
    ).toBe('等了 8 分钟')
    expect(formatTrackedInterestWaitLabel('2026-04-19T19:35:00+08:00', null)).toBe('暂未等到回流')
  })

  test('formats tracked block timestamps into card-friendly labels', () => {
    expect(formatTrackedInterestTime('2026-04-19T19:35:00+08:00')).toBe('04-19 19:35')
  })

  test('formats tracked block timestamps without relying on Intl', () => {
    const intlHolder = globalThis as { Intl?: typeof Intl }
    const originalIntl = intlHolder.Intl

    // Simulate real-device runtimes that do not expose Intl.
    intlHolder.Intl = undefined

    try {
      expect(formatTrackedInterestTime('2026-04-19T19:35:00+08:00')).toBe('04-19 19:35')
    } finally {
      intlHolder.Intl = originalIntl
    }
  })
})

describe('ticket watch section collapse state', () => {
  test('defaults every section to expanded until the user explicitly collapses it', () => {
    expect(isTicketWatchSectionCollapsed({}, 'current:573:focus')).toBe(false)
  })

  test('toggles one section without mutating other collapse entries', () => {
    const nextState = toggleTicketWatchSectionCollapsed(
      {
        'current:573:focus': true,
        'history:572:price:180': false,
      },
      'history:572:price:180',
    )

    expect(nextState).toEqual({
      'current:573:focus': true,
      'history:572:price:180': true,
    })
    expect(isTicketWatchSectionCollapsed(nextState, 'history:572:price:180')).toBe(true)
  })
})

describe('prioritizeInventorySections', () => {
  test('shows active and hotter price bands first so users see reflux areas earlier', () => {
    const result = prioritizeInventorySections([
      {
        price: '100',
        region_count: 10,
        available_region_count: 0,
        total_occurrences: 0,
        items: [],
      },
      {
        price: '180',
        region_count: 12,
        available_region_count: 3,
        total_occurrences: 90,
        items: [],
      },
      {
        price: '120',
        region_count: 8,
        available_region_count: 5,
        total_occurrences: 120,
        items: [],
      },
    ])

    expect(result.map((item) => item.price)).toEqual(['120', '180', '100'])
  })
})

describe('resolveInventoryPriceTone', () => {
  test('maps ticket prices to the stadium legend color bands', () => {
    expect(resolveInventoryPriceTone('1288')).toBe('vip')
    expect(resolveInventoryPriceTone('400')).toBe('s')
    expect(resolveInventoryPriceTone('220')).toBe('a')
    expect(resolveInventoryPriceTone('180')).toBe('b')
    expect(resolveInventoryPriceTone('150')).toBe('c')
    expect(resolveInventoryPriceTone('120')).toBe('d')
    expect(resolveInventoryPriceTone('100')).toBe('e')
  })

  test('falls back to neutral when the price is unknown', () => {
    expect(resolveInventoryPriceTone('')).toBe('neutral')
    expect(resolveInventoryPriceTone('abc')).toBe('neutral')
  })
})

describe('summarizeInventoryBoard', () => {
  test('derives supply insight metrics and top blocks from grouped sections', () => {
    const result = summarizeInventoryBoard([
      {
        price: '100',
        region_count: 12,
        available_region_count: 9,
        total_occurrences: 1364,
        items: [
          { block_name: '501', price: '100', occurrences: 0, latest_time: '', has_inventory: false, interested_user_count: 0, viewer_interested: false },
          { block_name: '504', price: '100', occurrences: 55, latest_time: '2026-03-15T11:39:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          { block_name: '505', price: '100', occurrences: 43, latest_time: '2026-03-15T11:31:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
      {
        price: '160',
        region_count: 8,
        available_region_count: 2,
        total_occurrences: 84,
        items: [
          { block_name: '602', price: '160', occurrences: 31, latest_time: '2026-03-15T11:20:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          { block_name: '603', price: '160', occurrences: 9, latest_time: '2026-03-15T11:18:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
    ])

    expect(result.priceBandCount).toBe(2)
    expect(result.totalRegionCount).toBe(20)
    expect(result.activeRegionCount).toBe(11)
    expect(Math.round(result.activeRegionRatio * 100)).toBe(55)
    expect(result.totalOccurrences).toBe(1448)
    expect(result.hottestPrice?.price).toBe('100')
    expect(result.hottestPrice?.total_occurrences).toBe(1364)
    expect(result.cheapestActivePrice?.price).toBe('100')
    expect(result.topBlocks.map((item) => item.block_name)).toEqual(['504', '505', '602'])
    expect(result.topInterestBlocks).toEqual([])
  })

  test('returns empty metrics when there is no inventory data', () => {
    const result = summarizeInventoryBoard([])

    expect(result.priceBandCount).toBe(0)
    expect(result.totalRegionCount).toBe(0)
    expect(result.activeRegionCount).toBe(0)
    expect(result.activeRegionRatio).toBe(0)
    expect(result.totalOccurrences).toBe(0)
    expect(result.hottestPrice).toBeNull()
    expect(result.cheapestActivePrice).toBeNull()
    expect(result.topBlocks).toEqual([])
    expect(result.topInterestBlocks).toEqual([])
  })

  test('excludes vip blocks from focus ranking even when vip has reflux data', () => {
    const result = summarizeInventoryBoard([
      {
        price: '1288',
        region_count: 2,
        available_region_count: 2,
        total_occurrences: 11207,
        items: [
          { block_name: 'VIP1', price: '1288', occurrences: 6405, latest_time: '2026-04-10T20:00:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          { block_name: 'VIP3', price: '1288', occurrences: 4802, latest_time: '2026-04-10T19:58:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
      {
        price: '220',
        region_count: 4,
        available_region_count: 1,
        total_occurrences: 78,
        items: [
          { block_name: '125', price: '220', occurrences: 78, latest_time: '2026-04-10T19:50:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
    ])

    expect(result.topBlocks.map((item) => item.block_name)).toEqual(['125'])
    expect(result.topInterestBlocks).toEqual([])
  })

  test('excludes vip sections from hottest price when non-vip reflux exists', () => {
    const result = summarizeInventoryBoard([
      {
        price: '1288',
        region_count: 2,
        available_region_count: 2,
        total_occurrences: 11207,
        items: [
          { block_name: 'VIP1', price: '1288', occurrences: 6405, latest_time: '2026-04-10T20:00:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          { block_name: 'VIP3', price: '1288', occurrences: 4802, latest_time: '2026-04-10T19:58:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
      {
        price: '220',
        region_count: 4,
        available_region_count: 2,
        total_occurrences: 177,
        items: [
          { block_name: '125', price: '220', occurrences: 177, latest_time: '2026-04-10T19:50:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          { block_name: '126', price: '220', occurrences: 0, latest_time: '', has_inventory: false, interested_user_count: 0, viewer_interested: false },
        ],
      },
      {
        price: '180',
        region_count: 4,
        available_region_count: 1,
        total_occurrences: 120,
        items: [
          { block_name: '532', price: '180', occurrences: 120, latest_time: '2026-04-10T19:40:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
    ])

    expect(result.hottestPrice?.price).toBe('220')
  })

  test('builds a separate top interest ranking and excludes vip blocks from it', () => {
    const result = summarizeInventoryBoard([
      {
        price: '1288',
        region_count: 2,
        available_region_count: 2,
        total_occurrences: 11207,
        items: [
          { block_name: 'VIP1', price: '1288', occurrences: 6405, latest_time: '2026-04-10T20:00:00+08:00', has_inventory: true, interested_user_count: 99, viewer_interested: false },
          { block_name: 'VIP3', price: '1288', occurrences: 4802, latest_time: '2026-04-10T19:58:00+08:00', has_inventory: true, interested_user_count: 66, viewer_interested: false },
        ],
      },
      {
        price: '220',
        region_count: 3,
        available_region_count: 2,
        total_occurrences: 177,
        items: [
          { block_name: '125', price: '220', occurrences: 177, latest_time: '2026-04-10T19:50:00+08:00', has_inventory: true, interested_user_count: 12, viewer_interested: false },
          { block_name: '126', price: '220', occurrences: 50, latest_time: '2026-04-10T19:42:00+08:00', has_inventory: true, interested_user_count: 5, viewer_interested: false },
          { block_name: '127', price: '220', occurrences: 20, latest_time: '2026-04-10T19:39:00+08:00', has_inventory: true, interested_user_count: 18, viewer_interested: false },
        ],
      },
      {
        price: '180',
        region_count: 2,
        available_region_count: 1,
        total_occurrences: 120,
        items: [
          { block_name: '532', price: '180', occurrences: 120, latest_time: '2026-04-10T19:40:00+08:00', has_inventory: true, interested_user_count: 15, viewer_interested: false },
          { block_name: '533', price: '180', occurrences: 10, latest_time: '2026-04-10T19:35:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
        ],
      },
    ])

    expect(result.topInterestBlocks).toEqual([
      { block_name: '127', price: '220', interested_user_count: 18 },
      { block_name: '532', price: '180', interested_user_count: 15 },
      { block_name: '125', price: '220', interested_user_count: 12 },
    ])
  })
})

describe('buildInventorySince', () => {
  test('returns 10 minutes after sale start', () => {
    expect(buildInventorySince('2026-04-10T14:00:00+08:00')).toBe('2026-04-10T14:10:00+08:00')
  })

  test('returns null when sale start is missing or invalid', () => {
    expect(buildInventorySince(null)).toBeNull()
    expect(buildInventorySince('not-a-date')).toBeNull()
  })
})

describe('resolveInventoryHeatLevel', () => {
  test('returns 0 when there is no valid inventory signal', () => {
    expect(resolveInventoryHeatLevel(0, 120)).toBe(0)
    expect(resolveInventoryHeatLevel(8, 0)).toBe(0)
  })

  test('splits active blocks into four visual heat bands', () => {
    expect(resolveInventoryHeatLevel(5, 100)).toBe(1)
    expect(resolveInventoryHeatLevel(20, 100)).toBe(2)
    expect(resolveInventoryHeatLevel(55, 100)).toBe(3)
    expect(resolveInventoryHeatLevel(92, 100)).toBe(4)
  })
})

describe('resolveInventoryRefluxFreshness', () => {
  test('maps recent reflux timestamps into strong and weak visual states', () => {
    const now = '2026-04-24T12:10:00+08:00'

    expect(resolveInventoryRefluxFreshness('2026-04-24T12:08:00+08:00', now)).toBe('strong')
    expect(resolveInventoryRefluxFreshness('2026-04-24T12:07:00+08:00', now)).toBe('strong')
    expect(resolveInventoryRefluxFreshness('2026-04-24T12:06:59+08:00', now)).toBe('weak')
    expect(resolveInventoryRefluxFreshness('2026-04-24T12:00:00+08:00', now)).toBe('weak')
    expect(resolveInventoryRefluxFreshness('2026-04-24T11:59:59+08:00', now)).toBe('none')
  })

  test('ignores missing invalid or future reflux timestamps', () => {
    const now = '2026-04-24T12:10:00+08:00'

    expect(resolveInventoryRefluxFreshness('', now)).toBe('none')
    expect(resolveInventoryRefluxFreshness('not-a-date', now)).toBe('none')
    expect(resolveInventoryRefluxFreshness('2026-04-24T12:11:00+08:00', now)).toBe('none')
  })
})

describe('buildRecentRefluxBuckets', () => {
  test('groups active reflux blocks into 3 10 and 30 minute windows', () => {
    const result = buildRecentRefluxBuckets(
      [
        {
          price: '150',
          region_count: 4,
          available_region_count: 4,
          total_occurrences: 20,
          items: [
            { block_name: '530', price: '150', occurrences: 2, latest_time: '2026-04-24T12:09:30+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '118', price: '120', occurrences: 8, latest_time: '2026-04-24T12:05:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '112', price: '220', occurrences: 6, latest_time: '2026-04-24T11:45:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '130', price: '220', occurrences: 9, latest_time: '2026-04-24T11:39:59+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
          ],
        },
      ],
      '2026-04-24T12:10:00+08:00',
    )

    expect(result.map((bucket) => bucket.key)).toEqual(['within3', 'within10', 'within30'])
    expect(result[0].items.map((item) => item.block_name)).toEqual(['530'])
    expect(result[1].items.map((item) => item.block_name)).toEqual(['118'])
    expect(result[2].items.map((item) => item.block_name)).toEqual(['112'])
  })

  test('ignores empty invalid and future timestamps', () => {
    const result = buildRecentRefluxBuckets(
      [
        {
          price: '150',
          region_count: 4,
          available_region_count: 4,
          total_occurrences: 20,
          items: [
            { block_name: '530', price: '150', occurrences: 2, latest_time: '', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '118', price: '120', occurrences: 8, latest_time: 'not-a-date', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '112', price: '220', occurrences: 6, latest_time: '2026-04-24T12:11:00+08:00', has_inventory: true, interested_user_count: 0, viewer_interested: false },
            { block_name: '130', price: '220', occurrences: 9, latest_time: '2026-04-24T12:08:00+08:00', has_inventory: false, interested_user_count: 0, viewer_interested: false },
          ],
        },
      ],
      '2026-04-24T12:10:00+08:00',
    )

    expect(result.every((bucket) => bucket.items.length === 0)).toBe(true)
  })
})

describe('resolveRecentRefluxPanelMode', () => {
  test('shows a locked upgrade prompt when recent reflux exists but membership is not enough', () => {
    expect(resolveRecentRefluxPanelMode(true, 'V5')).toBe('locked')
  })

  test('shows unlocked content when recent reflux exists and membership reaches V6', () => {
    expect(resolveRecentRefluxPanelMode(true, 'V6')).toBe('unlocked')
    expect(resolveRecentRefluxPanelMode(true, 'V9')).toBe('unlocked')
  })

  test('keeps the recent reflux panel visible for V6 members even when no recent blocks exist', () => {
    expect(resolveRecentRefluxPanelMode(false, 'V6')).toBe('unlocked')
    expect(resolveRecentRefluxPanelMode(false, 'V9')).toBe('unlocked')
  })

  test('keeps the recent reflux panel visible as a V6 benefit prompt below V6', () => {
    expect(resolveRecentRefluxPanelMode(false, 'V5')).toBe('locked')
    expect(resolveRecentRefluxPanelMode(false, null)).toBe('locked')
  })
})

describe('isRecentRefluxBucketUnlocked', () => {
  test('unlocks recent reflux windows by membership tier', () => {
    expect(isRecentRefluxBucketUnlocked('within30', 'V6')).toBe(true)
    expect(isRecentRefluxBucketUnlocked('within10', 'V6')).toBe(false)
    expect(isRecentRefluxBucketUnlocked('within10', 'V7')).toBe(true)
    expect(isRecentRefluxBucketUnlocked('within3', 'V7')).toBe(false)
    expect(isRecentRefluxBucketUnlocked('within3', 'V8')).toBe(true)
    expect(isRecentRefluxBucketUnlocked('within3', 'V9')).toBe(true)
  })
})

describe('vip inventory labels', () => {
  test('detects vip block names', () => {
    expect(isVipInventoryBlock('VIP1')).toBe(true)
    expect(isVipInventoryBlock('vip3')).toBe(true)
    expect(isVipInventoryBlock('504')).toBe(false)
  })

  test('uses vip fallback copy when vip area has no tracked reflux data', () => {
    expect(getInventoryEmptyStateLabel('VIP1')).toBe('VIP 暂未统计')
    expect(getInventoryEmptyStateLabel('504')).toBe('无回流')
  })

  test('formats vip section summary as not tracked when no vip data is available', () => {
    expect(
      formatInventorySectionSummary({
        price: '1288.00',
        region_count: 2,
        available_region_count: 0,
        total_occurrences: 0,
        items: [
          { block_name: 'VIP1', price: '1288.00', occurrences: 0, latest_time: '', has_inventory: false, interested_user_count: 0, viewer_interested: false },
          { block_name: 'VIP3', price: '1288.00', occurrences: 0, latest_time: '', has_inventory: false, interested_user_count: 0, viewer_interested: false },
        ],
      }),
    ).toBe('VIP 暂未统计')
  })
})
