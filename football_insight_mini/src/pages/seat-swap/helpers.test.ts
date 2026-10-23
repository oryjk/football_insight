import { describe, expect, test } from 'bun:test'

import {
  buildSeatSwapRegionAnchorId,
  canInteractSeatSwapMap,
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  canPublishSeatSwapRequest,
  filterOutMySeatSwapRequest,
  filterSeatSwapRequestsByDesiredRegion,
  groupSeatSwapRequestsByRegion,
  countSeatSwapDesiredRegions,
  previousSeatSwapStep,
  resolveSeatSwapBrowseFilterKey,
  resolveDefaultExpandedSeatSwapRegionKeys,
  shouldDimSeatSwapRegion,
  SEAT_SWAP_POOL_ANCHOR_ID,
  buildSeatSwapSwapSummary,
  resolveSeatSwapPitchBox,
  buildSeatSwapSeatPriceRows,
  formatSeatSwapRegionPrice,
  SEAT_SWAP_STADIUM_REGION_NAMES,
  buildSeatSwapThumbSvg,
  resolveSeatSwapThumbHighlights,
  formatSeatSwapKickoffShort,
  isSeatSwapMapRectInViewport,
  resolveSeatSwapCandidateAction,
  resolveSeatSwapMapFabLabel,
  resolveSeatSwapMapTap,
  resolveSeatSwapStaggerDelayMs,
  shouldShowSeatSwapMapFab,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  validateSeatSwapPhoneNumber,
  type SeatSwapDesiredSeatFormState,
  type SeatSwapFormState,
} from './helpers'
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'
import { formatSeatSwapSeatLabel, resolveSeatSwapRegionLayout, seatSwapStatusLabel } from '../../utils/stadiumRegions'

function validForm(): SeatSwapFormState {
  return {
    current_region_key: 'A',
    current_region_name: 'A区',
    current_row: '8',
    current_seat_no: '15',
    wechat_id: 'wx-test',
    phone_number: '',
    desired_seats: [
      {
        region_key: 'B',
        region_name: 'B区',
        desired_row: '',
        desired_seat_no: '',
      },
    ],
  }
}

describe('seat swap form validation', () => {
  test('requires at least one contact method', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      wechat_id: '',
      phone_number: '',
    })

    expect(errors.contact).toBe('请至少填写微信号或手机号')
  })

  test('validates the phone number format only when provided', () => {
    expect(validateSeatSwapPhoneNumber('')).toBe(undefined)
    expect(validateSeatSwapPhoneNumber(' 13812345678 ')).toBe(undefined)
    expect(validateSeatSwapPhoneNumber('1381234567')).toBe('手机号应为 11 位，当前 10 位')
    expect(validateSeatSwapPhoneNumber('12812345678')).toBe('手机号格式不正确')
    expect(validateSeatSwapPhoneNumber('1381234567a')).toBe('手机号只能是数字')
    expect(validateSeatSwapForm({ ...validForm(), phone_number: '23812345678' }).phone_number).toBe('手机号格式不正确')
  })

  test('requires current row and seat number', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      current_row: '',
      current_seat_no: '',
    })

    expect(errors.current_row).toBe('请输入当前排号')
    expect(errors.current_seat_no).toBe('请输入当前座号')
  })

  test('requires desired region for each target', () => {
    const errors = validateSeatSwapForm({
      ...validForm(),
      desired_seats: [{ region_key: '', region_name: '', desired_row: '', desired_seat_no: '' }],
    })

    expect(errors.desired_seats).toBe('请选择目标座位分区')
  })
})

describe('seat swap request grouping', () => {
  test('filters my own request out of the public swap pool', () => {
    const requests = [
      { request_id: 'mine', current_region_key: '101', current_region_name: '101', created_at: '1' },
      { request_id: 'peer', current_region_key: '102', current_region_name: '102', created_at: '2' },
    ]

    expect(filterOutMySeatSwapRequest(requests, 'mine')).toEqual([
      { request_id: 'peer', current_region_key: '102', current_region_name: '102', created_at: '2' },
    ])
  })

  test('groups published requests by their current region', () => {
    const groups = groupSeatSwapRequestsByRegion([
      {
        request_id: '1',
        current_region_key: '101',
        current_region_name: '101',
        created_at: '2026-05-19T12:00:00+08:00',
      },
      {
        request_id: '2',
        current_region_key: '102',
        current_region_name: '102',
        created_at: '2026-05-19T12:01:00+08:00',
      },
      {
        request_id: '3',
        current_region_key: '101',
        current_region_name: '101',
        created_at: '2026-05-19T12:02:00+08:00',
      },
    ])

    expect(groups).toEqual([
      {
        region_key: '101',
        region_name: '101',
        requests: [
          {
            request_id: '3',
            current_region_key: '101',
            current_region_name: '101',
            created_at: '2026-05-19T12:02:00+08:00',
          },
          {
            request_id: '1',
            current_region_key: '101',
            current_region_name: '101',
            created_at: '2026-05-19T12:00:00+08:00',
          },
        ],
      },
      {
        region_key: '102',
        region_name: '102',
        requests: [
          {
            request_id: '2',
            current_region_key: '102',
            current_region_name: '102',
            created_at: '2026-05-19T12:01:00+08:00',
          },
        ],
      },
    ])
  })

  test('counts and filters published requests by desired regions for the stadium badge', () => {
    const requests = [
      {
        request_id: '1',
        current_region_key: '131',
        current_region_name: '131',
        created_at: '2026-05-19T12:00:00+08:00',
        desired_seats: [
          { region_key: '531', region_name: '531' },
          { region_key: '532', region_name: '532' },
        ],
      },
      {
        request_id: '2',
        current_region_key: '132',
        current_region_name: '132',
        created_at: '2026-05-19T12:01:00+08:00',
        desired_seats: [
          { region_key: '531', region_name: '531' },
          { region_key: '131', region_name: '131' },
        ],
      },
    ]

    expect(countSeatSwapDesiredRegions(requests)).toEqual({
      131: 1,
      531: 2,
      532: 1,
    })
    expect(filterSeatSwapRequestsByDesiredRegion(requests, '531').map((request) => request.request_id)).toEqual([
      '1',
      '2',
    ])
    expect(filterSeatSwapRequestsByDesiredRegion(requests, '132')).toEqual([])
  })

  test('formats seat labels without requiring row and seat number', () => {
    expect(formatSeatSwapSeatLabel({
      current_region_name: '101',
      current_row: '',
      current_seat_no: '',
    })).toBe('101')
    expect(formatSeatSwapSeatLabel({
      current_region_name: '101',
      current_row: '8',
      current_seat_no: '',
    })).toBe('101 8排')
  })

  test('does not show a display-only status label', () => {
    expect(seatSwapStatusLabel('display_only')).toBe('')
  })
})

describe('seat swap candidate actions', () => {
  test('lets a logged-in viewer start publishing against a display-only candidate before they have their own request', () => {
    expect(resolveSeatSwapCandidateAction({
      candidateStatus: 'display_only',
      candidateRequestId: 'target-request',
      isLoggedIn: true,
    })).toBe('confirm')
  })

  test('lets the viewer cancel their pending confirmation instead of confirming again', () => {
    expect(resolveSeatSwapCandidateAction({
      candidateStatus: 'waiting_peer_confirmation',
      candidateRequestId: 'target-request',
      myRequestId: 'my-request',
      isLoggedIn: true,
    })).toBe('cancel_confirmation')
  })

  test('does not show an inline action for matched candidates', () => {
    expect(resolveSeatSwapCandidateAction({
      candidateStatus: 'matched',
      candidateRequestId: 'target-request',
      myRequestId: 'my-request',
      isLoggedIn: true,
    })).toBe('matched_cancel')
  })
})

describe('seat swap map interaction', () => {
  test('keeps the stadium map tappable in published mode', () => {
    expect(canInteractSeatSwapMap('browse')).toBe(true)
    expect(canInteractSeatSwapMap('filter')).toBe(true)
    expect(canInteractSeatSwapMap('published')).toBe(true)
    expect(canInteractSeatSwapMap('review')).toBe(false)
  })

  test('builds stable anchor ids for region groups', () => {
    expect(buildSeatSwapRegionAnchorId('531')).toBe('seat-swap-group-531')
    expect(buildSeatSwapRegionAnchorId('VIP2')).toBe('seat-swap-group-VIP2')
  })

  test('expands every available region group by default', () => {
    expect(resolveDefaultExpandedSeatSwapRegionKeys(['508', '522', '531'])).toEqual(['508', '522', '531'])
  })

  test('toggles the selected browse region off when tapped again', () => {
    expect(resolveSeatSwapBrowseFilterKey('', '531')).toBe('531')
    expect(resolveSeatSwapBrowseFilterKey('531', '531')).toBe('')
    expect(resolveSeatSwapBrowseFilterKey('531', '532')).toBe('532')
  })

  test('does not gray out non-selected regions in browse and filter modes', () => {
    expect(shouldDimSeatSwapRegion({
      mode: 'browse',
      hasFilterKey: false,
      isFilterMatched: false,
      hasStagedCurrentKey: false,
      isStagedCurrent: false,
      hasStagedDesiredKeys: false,
      isStagedDesired: false,
      isCurrent: false,
      isDesired: false,
    })).toBe(false)

    expect(shouldDimSeatSwapRegion({
      mode: 'filter',
      hasFilterKey: true,
      isFilterMatched: false,
      hasStagedCurrentKey: false,
      isStagedCurrent: false,
      hasStagedDesiredKeys: false,
      isStagedDesired: false,
      isCurrent: false,
      isDesired: false,
    })).toBe(false)
  })
})

describe('seat swap mock data', () => {
  test('builds a large candidate list for scroll testing', () => {
    const mockView = buildSeatSwapMockCurrentResponse({
      candidateCount: 72,
      includeMyRequest: true,
    })

    expect(mockView.available).toBe(true)
    expect(mockView.current_match?.home_team_name).toBe('成都蓉城')
    expect(mockView.candidates.length).toBe(72)
    expect(Boolean(mockView.my_request?.request_id)).toBe(true)
    expect(new Set(mockView.candidates.map((candidate) => candidate.current_region_key)).size > 8).toBe(true)
    expect(
      mockView.candidates.some((candidate) => candidate.desired_seats.length > 1),
    ).toBe(true)
  })

  test('builds a region list that can drive the stadium map in mock mode', () => {
    const regions = buildSeatSwapMockRegions()
    const regionKeys = new Set(regions.map((region) => region.block_key || region.block_name))

    expect(regions.length > 40).toBe(true)
    expect(regionKeys.has('101')).toBe(true)
    expect(regionKeys.has('531')).toBe(true)
    expect(regionKeys.has('VIP2')).toBe(true)
    for (let region = 101; region <= 132; region += 1) {
      expect(regionKeys.has(String(region))).toBe(true)
    }
    for (let region = 501; region <= 536; region += 1) {
      expect(regionKeys.has(String(region))).toBe(true)
    }
  })
})

describe('seat swap stadium region layout', () => {
  function layoutBounds(start: number, end: number) {
    const layouts = Array.from({ length: end - start + 1 }, (_, index) =>
      resolveSeatSwapRegionLayout(String(start + index)),
    ).filter((layout): layout is NonNullable<typeof layout> => Boolean(layout))

    const minLeft = Math.min(...layouts.map((layout) => layout.left))
    const maxRight = Math.max(...layouts.map((layout) => layout.left + layout.width))
    const minTop = Math.min(...layouts.map((layout) => layout.top))
    const maxBottom = Math.max(...layouts.map((layout) => layout.top + layout.height))

    return {
      minLeft,
      maxRight,
      minTop,
      maxBottom,
      centerX: (minLeft + maxRight) / 2,
      width: maxRight - minLeft,
      height: maxBottom - minTop,
    }
  }

  test('places the 100-level ring from 101 at the lower-right and counterclockwise', () => {
    const first = resolveSeatSwapRegionLayout('101')
    const next = resolveSeatSwapRegionLayout('102')
    const topStart = resolveSeatSwapRegionLayout('108')
    const bottomStart = resolveSeatSwapRegionLayout('124')
    const last = resolveSeatSwapRegionLayout('132')

    expect(first?.ring).toBe('inner')
    expect(first?.side).toBe('right')
    expect(next?.side).toBe('right')
    expect((next?.top || 0) < (first?.top || 0)).toBe(true)
    expect((first?.left || 0) > 80).toBe(true)
    expect((first?.top || 0) > 60).toBe(true)
    expect(topStart?.side).toBe('top')
    expect(bottomStart?.side).toBe('bottom')
    expect((bottomStart?.left || 0) < (first?.left || 0)).toBe(true)
    expect(last?.side).toBe('bottom')
    expect((last?.left || 0) > (bottomStart?.left || 0)).toBe(true)
  })

  test('places 100-level regions on a rounded stadium bowl instead of straight stadium edges', () => {
    const rightLower = resolveSeatSwapRegionLayout('101')
    const topStart = resolveSeatSwapRegionLayout('108')
    const topMiddle = resolveSeatSwapRegionLayout('112')
    const leftMiddle = resolveSeatSwapRegionLayout('117')
    const bottomStart = resolveSeatSwapRegionLayout('124')
    const bottomMiddle = resolveSeatSwapRegionLayout('129')

    expect(rightLower?.side).toBe('right')
    expect(topMiddle?.side).toBe('top')
    expect(leftMiddle?.side).toBe('left')
    expect(bottomMiddle?.side).toBe('bottom')
    expect((rightLower?.left || 0) > (topMiddle?.left || 0)).toBe(true)
    expect((topMiddle?.left || 0) > (leftMiddle?.left || 0)).toBe(true)
    expect((bottomMiddle?.left || 0) > (leftMiddle?.left || 0)).toBe(true)
    expect((bottomMiddle?.top || 0) > (rightLower?.top || 0)).toBe(true)
    expect(topStart?.top === topMiddle?.top).toBe(false)
    expect(bottomStart?.top === bottomMiddle?.top).toBe(false)
  })

  test('keeps 100-level corner regions from overlapping their neighboring side', () => {
    const cornerPairs = [
      ['107', '108'],
      ['116', '117'],
      ['123', '124'],
      ['132', '101'],
    ] as const

    for (const [leftRegion, rightRegion] of cornerPairs) {
      const left = resolveSeatSwapRegionLayout(leftRegion)
      const right = resolveSeatSwapRegionLayout(rightRegion)

      expect(Boolean(left)).toBe(true)
      expect(Boolean(right)).toBe(true)
      expect(`${left?.left},${left?.top}` === `${right?.left},${right?.top}`).toBe(false)
      expect(rectanglesOverlap(left!, right!)).toBe(false)
    }
  })

  test('places the 500-level ring outside the 100-level ring with 501 at the lower-right', () => {
    const inner = resolveSeatSwapRegionLayout('101')
    const outer = resolveSeatSwapRegionLayout('501')
    const outerNext = resolveSeatSwapRegionLayout('502')
    const outerTopStart = resolveSeatSwapRegionLayout('510')
    const outerTop = resolveSeatSwapRegionLayout('514')
    const outerBottomStart = resolveSeatSwapRegionLayout('528')

    expect(outer?.ring).toBe('outer')
    expect(outer?.side).toBe('right')
    expect(outerNext?.side).toBe('right')
    expect((outerNext?.top || 0) < (outer?.top || 0)).toBe(true)
    expect((outer?.left || 0) > (inner?.left || 0)).toBe(true)
    expect(outerTopStart?.side).toBe('top')
    expect((outerTop?.top || 0) < (inner?.top || 0)).toBe(true)
    expect(outerBottomStart?.side).toBe('bottom')
  })

  test('centers the region bowl and uses most of the available width', () => {
    const inner = layoutBounds(101, 132)
    const outer = layoutBounds(501, 536)

    expect(Math.abs(inner.centerX - 50) < 2).toBe(true)
    expect(Math.abs(outer.centerX - 50) < 2).toBe(true)
    expect(inner.width > 72).toBe(true)
    expect(inner.width < 82).toBe(true)
    expect(outer.width > 88).toBe(true)
    expect(outer.width < 96).toBe(true)
  })

  test('uses wider horizontal regions without pushing the bowl to the page edge', () => {
    const innerTop = resolveSeatSwapRegionLayout('113')
    const innerSide = resolveSeatSwapRegionLayout('104')
    const outerTop = resolveSeatSwapRegionLayout('514')
    const outerSide = resolveSeatSwapRegionLayout('506')
    const outer = layoutBounds(501, 536)

    expect((innerTop?.width || 0) > (innerSide?.width || 0)).toBe(true)
    expect((outerTop?.width || 0) > (outerSide?.width || 0)).toBe(true)
    expect(outer.minLeft >= 2).toBe(true)
    expect(outer.maxRight <= 98).toBe(true)
  })

  test('keeps 500-level corner regions from overlapping their neighboring side', () => {
    const cornerPairs = [
      ['509', '510'],
      ['518', '519'],
      ['527', '528'],
      ['536', '501'],
    ] as const

    for (const [leftRegion, rightRegion] of cornerPairs) {
      const left = resolveSeatSwapRegionLayout(leftRegion)
      const right = resolveSeatSwapRegionLayout(rightRegion)

      expect(Boolean(left)).toBe(true)
      expect(Boolean(right)).toBe(true)
      expect(`${left?.left},${left?.top}` === `${right?.left},${right?.top}`).toBe(false)
      expect(rectanglesOverlap(left!, right!)).toBe(false)
    }
  })

  test('keeps all rendered stadium regions independently tappable', () => {
    const regions = [...range(101, 132), ...range(501, 536), 'VIP1', 'VIP2', 'VIP3'].map((name) => {
      const layout = resolveSeatSwapRegionLayout(name)
      expect(Boolean(layout)).toBe(true)
      return { name, layout: layout! }
    })

    for (let index = 0; index < regions.length; index += 1) {
      for (let compareIndex = index + 1; compareIndex < regions.length; compareIndex += 1) {
        const pair = `${regions[index].name}/${regions[compareIndex].name}`
        expect(`${pair}:${rectanglesOverlap(regions[index].layout, regions[compareIndex].layout)}`).toBe(
          `${pair}:false`,
        )
      }
    }
  })

  test('places vip regions between 100-level and 500-level bottom stands', () => {
    const innerBottom = resolveSeatSwapRegionLayout('128')
    const outerBottom = resolveSeatSwapRegionLayout('532')
    const vip1 = resolveSeatSwapRegionLayout('VIP1')
    const vip2 = resolveSeatSwapRegionLayout('VIP2')
    const vip3 = resolveSeatSwapRegionLayout('VIP3')

    expect(vip1?.ring).toBe('vip')
    expect(vip2?.ring).toBe('vip')
    expect(vip3?.ring).toBe('vip')
    expect((vip1?.top || 0) > (innerBottom?.top || 0)).toBe(true)
    expect((vip1?.top || 0) < (outerBottom?.top || 0)).toBe(true)
    expect((vip2?.left || 0) > (vip1?.left || 0)).toBe(true)
    expect((vip3?.left || 0) > (vip2?.left || 0)).toBe(true)
  })

  test('does not force unknown regions into the stadium map', () => {
    expect(resolveSeatSwapRegionLayout('VIP')).toBeNull()
    expect(resolveSeatSwapRegionLayout('UNKNOWN')).toBeNull()
  })
})

function rectanglesOverlap(
  a: NonNullable<ReturnType<typeof resolveSeatSwapRegionLayout>>,
  b: NonNullable<ReturnType<typeof resolveSeatSwapRegionLayout>>,
): boolean {
  return (
    a.left < b.left + b.width &&
    a.left + a.width > b.left &&
    a.top < b.top + b.height &&
    a.top + a.height > b.top
  )
}

function range(start: number, end: number): string[] {
  return Array.from({ length: end - start + 1 }, (_, index) => String(start + index))
}

describe('seat swap stadium selection flow', () => {
  test('confirms the current region only after a region is selected', () => {
    expect(canConfirmCurrentSeatRegion('', '8', '15')).toBe(false)
    expect(canConfirmCurrentSeatRegion(' 123 ', '', '15')).toBe(false)
    expect(canConfirmCurrentSeatRegion(' 123 ', '8', '')).toBe(false)
    expect(canConfirmCurrentSeatRegion(' 123 ', '8', '15')).toBe(true)
  })

  test('toggles desired regions without duplicating selections', () => {
    const selected: SeatSwapDesiredSeatFormState[] = [
      { region_key: '123', region_name: '123区', desired_row: '', desired_seat_no: '' },
    ]

    expect(toggleDesiredSeatRegion(selected, { region_key: '124', region_name: '124区' })).toEqual([
      { region_key: '123', region_name: '123区', desired_row: '', desired_seat_no: '' },
      { region_key: '124', region_name: '124区', desired_row: '', desired_seat_no: '' },
    ])

    expect(toggleDesiredSeatRegion(selected, { region_key: '123', region_name: '123区' })).toEqual([])
  })

  test('publishes only after current and desired regions are confirmed', () => {
    expect(canConfirmDesiredSeatRegions([])).toBe(false)
    expect(canConfirmDesiredSeatRegions([{ region_key: '123', region_name: '123区', desired_row: '', desired_seat_no: '' }])).toBe(true)

    expect(canPublishSeatSwapRequest('select_current')).toBe(false)
    expect(canPublishSeatSwapRequest('select_desired')).toBe(false)
    expect(canPublishSeatSwapRequest('ready_to_publish')).toBe(true)
  })

  test('moves back through the wizard one step at a time', () => {
    expect(previousSeatSwapStep('ready_to_publish')).toBe('select_desired')
    expect(previousSeatSwapStep('select_desired')).toBe('select_current')
    expect(previousSeatSwapStep('select_current')).toBe('select_current')
  })

  test('returns to my seat instead of editing the fixed peer seat when confirming a candidate', () => {
    expect(previousSeatSwapStep('ready_to_publish', true)).toBe('select_current')
    expect(previousSeatSwapStep('select_current', true)).toBe('select_current')
  })
})

describe('seat swap map floating button', () => {
  const labelCases: Array<[string, { text: string; clearable: boolean }]> = [
    ['', { text: '球场图', clearable: false }],
    ['   ', { text: '球场图', clearable: false }],
    ['112 区', { text: '112 区', clearable: true }],
    [' 112 区 ', { text: '112 区', clearable: true }],
  ]
  for (const [filterName, expected] of labelCases) {
    test(`labels ${JSON.stringify(filterName)}`, () => {
      expect(resolveSeatSwapMapFabLabel(filterName)).toEqual(expected)
    })
  }

  const fabCases: Array<[boolean, boolean, boolean, boolean]> = [
    // available, mapInViewport, sheetOpen, expected
    [true, false, false, true],
    [true, true, false, false],
    [true, false, true, false],
    [true, true, true, false],
    [false, false, false, false],
    [false, true, false, false],
    [false, false, true, false],
    [false, true, true, false],
  ]
  for (const [available, mapInViewport, sheetOpen, expected] of fabCases) {
    test(`available=${available} mapInViewport=${mapInViewport} sheetOpen=${sheetOpen} → ${expected}`, () => {
      expect(shouldShowSeatSwapMapFab({ available, mapInViewport, sheetOpen })).toBe(expected)
    })
  }
})

describe('seat swap list stagger', () => {
  const staggerCases: Array<[number, number]> = [
    [0, 0], [1, 40], [5, 200], [9, 200], [-1, 0], [Number.NaN, 0], [Number.POSITIVE_INFINITY, 0],
  ]
  for (const [index, expected] of staggerCases) {
    test(`index ${index} → ${expected}ms`, () => {
      expect(resolveSeatSwapStaggerDelayMs(index)).toBe(expected)
    })
  }

  test('honours custom step and cap', () => {
    expect(resolveSeatSwapStaggerDelayMs(3, 30, 2)).toBe(60)
  })
})

describe('seat swap map tap outcome', () => {
  test('main map: tapping a region with candidates filters and scrolls without a sheet', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '112', hasBadge: true, fromSheet: false }))
      .toEqual({ filterKey: '112', closeSheet: false, scrollTo: SEAT_SWAP_POOL_ANCHOR_ID })
  })

  test('sheet: tapping a region with candidates filters, closes the sheet and scrolls', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '112', hasBadge: true, fromSheet: true }))
      .toEqual({ filterKey: '112', closeSheet: true, scrollTo: SEAT_SWAP_POOL_ANCHOR_ID })
  })

  test('tapping the active region again clears the filter and stays put', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '112', tappedKey: '112', hasBadge: true, fromSheet: true }))
      .toEqual({ filterKey: '', closeSheet: false, scrollTo: null })
  })

  test('tapping a region without candidates only switches the filter', () => {
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '113', hasBadge: false, fromSheet: true }))
      .toEqual({ filterKey: '113', closeSheet: false, scrollTo: null })
    expect(resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '113', hasBadge: false, fromSheet: false }))
      .toEqual({ filterKey: '113', closeSheet: false, scrollTo: null })
  })
})

describe('seat swap map scroll target', () => {
  // 筛选态下列表只渲染筛选结果，分组锚点不存在；滚动目标必须是始终存在的换座池锚点。
  test('scrolls to the always-rendered pool anchor, not a region group anchor', () => {
    expect(SEAT_SWAP_POOL_ANCHOR_ID).toBe('seat-swap-pool-anchor')
    const outcome = resolveSeatSwapMapTap({ currentFilterKey: '', tappedKey: '112', hasBadge: true, fromSheet: true })
    expect(outcome.scrollTo).toBe(SEAT_SWAP_POOL_ANCHOR_ID)
  })
})

describe('seat swap map initial visibility', () => {
  const cases: Array<[string, { top: number; bottom: number } | null, boolean]> = [
    ['unknown rect keeps the map treated as visible', null, true],
    ['map fully inside the viewport', { top: 120, bottom: 400 }, true],
    ['map partly under the nav but still showing', { top: 20, bottom: 100 }, true],
    ['map scrolled entirely under the nav', { top: -300, bottom: 60 }, false],
    ['map scrolled above the screen', { top: -500, bottom: -100 }, false],
    ['map below the screen', { top: 900, bottom: 1200 }, false],
  ]
  for (const [label, rect, expected] of cases) {
    test(label, () => {
      expect(isSeatSwapMapRectInViewport(rect, 64, 800)).toBe(expected)
    })
  }
})

describe('seat swap compact kickoff label', () => {
  test('formats local kickoff as MM/DD HH:mm', () => {
    expect(formatSeatSwapKickoffShort(new Date(2026, 9, 25, 19, 35).toISOString())).toBe('10/25 19:35')
    expect(formatSeatSwapKickoffShort(new Date(2026, 0, 3, 9, 5).toISOString())).toBe('01/03 09:05')
  })

  test('returns empty text for missing or invalid input', () => {
    expect(formatSeatSwapKickoffShort('')).toBe('')
    expect(formatSeatSwapKickoffShort('not-a-date')).toBe('')
  })
})

describe('seat swap seat thumbnail', () => {
  test('lists every stadium region once: 101-132, 501-536 and three VIP blocks', () => {
    expect(SEAT_SWAP_STADIUM_REGION_NAMES.length).toBe(71)
    expect(new Set(SEAT_SWAP_STADIUM_REGION_NAMES).size).toBe(71)
    expect(SEAT_SWAP_STADIUM_REGION_NAMES.includes('132')).toBe(true)
    expect(SEAT_SWAP_STADIUM_REGION_NAMES.includes('536')).toBe(true)
    expect(SEAT_SWAP_STADIUM_REGION_NAMES.includes('VIP3')).toBe(true)
  })

  test('highlights the current region and each desired region with percentage boxes', () => {
    const highlights = resolveSeatSwapThumbHighlights('108', ['112', 'VIP2'])
    expect(highlights.map((h) => [h.key, h.kind])).toEqual([
      ['112', 'desired'],
      ['VIP2', 'desired'],
      ['108', 'current'],
    ])
    const layout = resolveSeatSwapRegionLayout('108')
    expect(highlights[2].style).toBe(
      `left:${layout?.left}%;top:${layout?.top}%;width:${layout?.width}%;height:${layout?.height}%;`,
    )
  })

  test('skips unknown regions and lets current win when a region is both', () => {
    const highlights = resolveSeatSwapThumbHighlights('108', ['108', 'no-such', '112', '112'])
    expect(highlights.map((h) => [h.key, h.kind])).toEqual([
      ['112', 'desired'],
      ['108', 'current'],
    ])
  })

  test('returns nothing when there is no mappable region', () => {
    expect(resolveSeatSwapThumbHighlights('', [])).toEqual([])
    expect(resolveSeatSwapThumbHighlights('no-such', ['also-no'])).toEqual([])
  })

  test('builds a text-free base svg with one block per region', () => {
    const svg = buildSeatSwapThumbSvg()
    expect(svg.startsWith('<svg')).toBe(true)
    expect((svg.match(/<rect x=/g) || []).length).toBe(71)
    expect(svg.includes('<text')).toBe(false)
  })
})

describe('seat swap seat sheet price rows', () => {
  const regions = [
    { block_key: '101', block_name: '101', price: '180.00' },
    { block_key: '109', block_name: '109', price: '220.00' },
    { block_key: 'VIP2', block_name: 'VIP2', price: '1288.00' },
    { block_key: '505', block_name: '505', price: '99.50' },
  ]

  test('formats a region price from the regions list', () => {
    expect(formatSeatSwapRegionPrice(regions, '101')).toBe('180元')
    expect(formatSeatSwapRegionPrice(regions, 'VIP2')).toBe('1288元')
    expect(formatSeatSwapRegionPrice(regions, '505')).toBe('99.5元')
  })

  test('returns empty text when the region or price is unknown', () => {
    expect(formatSeatSwapRegionPrice(regions, '999')).toBe('')
    expect(formatSeatSwapRegionPrice([{ block_key: '101', block_name: '101', price: '' }], '101')).toBe('')
    expect(formatSeatSwapRegionPrice([{ block_key: '101', block_name: '101', price: 'abc' }], '101')).toBe('')
  })

  test('lists the current seat then one row per desired region, each with its own price', () => {
    const rows = buildSeatSwapSeatPriceRows(
      {
        current_region_key: '101',
        current_region_name: '101',
        current_row: '1',
        current_seat_no: '1',
        desired_seats: [
          { region_key: '109', region_name: '109', desired_row: '1', desired_seat_no: '1' },
          { region_key: '999', region_name: '999', desired_row: null, desired_seat_no: null },
        ],
      },
      regions,
    )
    expect(rows).toEqual([
      { kind: 'current', label: '101 1排 1号', price: '180元' },
      { kind: 'desired', label: '109 1排 1号', price: '220元' },
      { kind: 'desired', label: '999', price: '' },
    ])
  })
})

describe('seat swap pitch box', () => {
  test('sits inside the 100-level ring without overlapping any inner region', () => {
    const pitch = resolveSeatSwapPitchBox()
    expect(pitch.width > 20 && pitch.height > 20).toBe(true)
    for (let n = 101; n <= 132; n += 1) {
      const r = resolveSeatSwapRegionLayout(String(n))
      if (!r) continue
      const overlaps = r.left < pitch.left + pitch.width && r.left + r.width > pitch.left
        && r.top < pitch.top + pitch.height && r.top + r.height > pitch.top
      expect(`${n}:${overlaps}`).toBe(`${n}:false`)
    }
  })

  test('the thumbnail base svg draws the pitch too', () => {
    expect(buildSeatSwapThumbSvg().includes('class="pitch"')).toBe(true)
  })
})

describe('seat swap swap summary', () => {
  const regions = [
    { block_key: '104', block_name: '104', price: '180.00' },
    { block_key: '503', block_name: '503', price: '100.00' },
    { block_key: '110', block_name: '110', price: '220.00' },
    { block_key: '108', block_name: '108', price: '180.00' },
  ]
  const request = {
    current_region_key: '104',
    current_region_name: '104',
    current_row: '21',
    current_seat_no: '40',
    desired_seats: [
      { region_key: '503', region_name: '503', desired_row: null, desired_seat_no: null },
      { region_key: '110', region_name: '110', desired_row: '1', desired_seat_no: '1' },
      { region_key: '108', region_name: '108', desired_row: null, desired_seat_no: null },
      { region_key: '999', region_name: '999', desired_row: null, desired_seat_no: null },
    ],
  }

  test('splits the current seat into region and row/seat with its price', () => {
    expect(buildSeatSwapSwapSummary(request, regions).current).toEqual({
      region: '104',
      seat: '21排 40号',
      price: '180元',
    })
  })

  test('lists each desired region with price and the difference against the current seat', () => {
    expect(buildSeatSwapSwapSummary(request, regions).desired).toEqual([
      { key: '503', region: '503', seat: '', price: '100元', delta: '省80元', deltaKind: 'cheaper' },
      { key: '110', region: '110', seat: '1排 1号', price: '220元', delta: '贵40元', deltaKind: 'pricier' },
      { key: '108', region: '108', seat: '', price: '180元', delta: '同价', deltaKind: 'same' },
      { key: '999', region: '999', seat: '', price: '', delta: '', deltaKind: 'unknown' },
    ])
  })

  test('omits the difference when the current price is unknown', () => {
    const summary = buildSeatSwapSwapSummary({ ...request, current_region_key: 'x', current_region_name: 'x' }, regions)
    expect(summary.current.price).toBe('')
    expect(summary.desired[0].delta).toBe('')
    expect(summary.desired[0].deltaKind).toBe('unknown')
  })
})
