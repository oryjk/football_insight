import { describe, expect, test } from 'bun:test'

import {
  buildSeatSwapMockCurrentResponse,
  buildSeatSwapMockRegions,
  buildSeatSwapRegionAnchorId,
  canInteractSeatSwapMap,
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  canPublishSeatSwapRequest,
  filterOutMySeatSwapRequest,
  filterSeatSwapRequestsByDesiredRegion,
  formatSeatLabel,
  groupSeatSwapRequestsByRegion,
  countSeatSwapDesiredRegions,
  previousSeatSwapStep,
  resolveSeatSwapBrowseFilterKey,
  resolveDefaultExpandedSeatSwapRegionKeys,
  resolveSeatSwapStickyMapThreshold,
  readSeatSwapViewportScrollTop,
  shouldDimSeatSwapRegion,
  resolveSeatSwapCandidateAction,
  shouldShowSeatSwapStickyMap,
  statusLabel,
  toggleDesiredSeatRegion,
  validateSeatSwapForm,
  type SeatSwapDesiredSeatFormState,
  type SeatSwapFormState,
} from './helpers'
import { resolveSeatSwapRegionLayout } from '../../utils/stadiumRegions'

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
    expect(formatSeatLabel({
      current_region_name: '101',
      current_row: '',
      current_seat_no: '',
    })).toBe('101')
    expect(formatSeatLabel({
      current_region_name: '101',
      current_row: '8',
      current_seat_no: '',
    })).toBe('101 8排')
  })

  test('does not show a display-only status label', () => {
    expect(statusLabel('display_only')).toBe('')
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

describe('seat swap sticky map', () => {
  test('shows the sticky mini map only after the page has scrolled past the threshold', () => {
    expect(shouldShowSeatSwapStickyMap(0, 360)).toBe(false)
    expect(shouldShowSeatSwapStickyMap(359, 360)).toBe(false)
    expect(shouldShowSeatSwapStickyMap(360, 360)).toBe(true)
    expect(shouldShowSeatSwapStickyMap(520, 360)).toBe(true)
  })

  test('does not show the sticky mini map before a measured threshold is ready', () => {
    expect(shouldShowSeatSwapStickyMap(520, Number.POSITIVE_INFINITY)).toBe(false)
  })

  test('reads viewport scrollTop from selector query results', () => {
    expect(readSeatSwapViewportScrollTop({ scrollTop: 520 })).toBe(520)
    expect(readSeatSwapViewportScrollTop({ top: 520 })).toBeNull()
    expect(readSeatSwapViewportScrollTop(null)).toBeNull()
  })

  test('falls back to a default threshold when sticky-map measurement is unavailable', () => {
    expect(resolveSeatSwapStickyMapThreshold({
      mapTop: null,
      viewportScrollTop: null,
      topOffsetPx: 32,
      fallbackThreshold: 420,
    })).toBe(420)
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
  test('builds a large candidate list for scroll and sticky-map testing', () => {
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
})
