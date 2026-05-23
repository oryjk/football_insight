import { describe, expect, test } from 'bun:test'

import {
  canConfirmCurrentSeatRegion,
  canConfirmDesiredSeatRegions,
  canPublishSeatSwapRequest,
  filterOutMySeatSwapRequest,
  formatSeatLabel,
  groupSeatSwapRequestsByRegion,
  previousSeatSwapStep,
  resolveSeatSwapCandidateAction,
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

    expect(errors.desired_seats).toBe('请选择想换到的分区')
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
    })).toBe('none')
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
      centerX: (minLeft + maxRight) / 2,
      width: maxRight - minLeft,
      height: maxBottom - minTop,
    }
  }

  test('places the 100-level ring from 101 at the lower-right and counterclockwise', () => {
    const first = resolveSeatSwapRegionLayout('101')
    const next = resolveSeatSwapRegionLayout('102')
    const bottomStart = resolveSeatSwapRegionLayout('125')
    const last = resolveSeatSwapRegionLayout('132')

    expect(first?.ring).toBe('inner')
    expect(first?.side).toBe('right')
    expect(next?.side).toBe('right')
    expect((next?.top || 0) < (first?.top || 0)).toBe(true)
    expect(bottomStart?.side).toBe('bottom')
    expect((bottomStart?.left || 0) < (first?.left || 0)).toBe(true)
    expect(last?.side).toBe('bottom')
    expect((last?.left || 0) < (first?.left || 0)).toBe(true)
  })

  test('places 100-level regions on a rounded stadium bowl instead of straight stadium edges', () => {
    const rightLower = resolveSeatSwapRegionLayout('101')
    const topStart = resolveSeatSwapRegionLayout('109')
    const topMiddle = resolveSeatSwapRegionLayout('113')
    const leftMiddle = resolveSeatSwapRegionLayout('117')
    const bottomStart = resolveSeatSwapRegionLayout('125')
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
      ['108', '109'],
      ['116', '117'],
      ['124', '125'],
      ['132', '101'],
    ] as const

    for (const [leftRegion, rightRegion] of cornerPairs) {
      const left = resolveSeatSwapRegionLayout(leftRegion)
      const right = resolveSeatSwapRegionLayout(rightRegion)

      expect(Boolean(left)).toBe(true)
      expect(Boolean(right)).toBe(true)
      expect(`${left?.left},${left?.top}` === `${right?.left},${right?.top}`).toBe(false)
    }
  })

  test('places the 500-level ring outside the 100-level ring with 501 at the lower-right', () => {
    const inner = resolveSeatSwapRegionLayout('101')
    const outer = resolveSeatSwapRegionLayout('501')
    const outerNext = resolveSeatSwapRegionLayout('502')
    const outerTop = resolveSeatSwapRegionLayout('514')

    expect(outer?.ring).toBe('outer')
    expect(outer?.side).toBe('right')
    expect(outerNext?.side).toBe('right')
    expect((outerNext?.top || 0) < (outer?.top || 0)).toBe(true)
    expect((outer?.left || 0) > (inner?.left || 0)).toBe(true)
    expect((outerTop?.top || 0) < (inner?.top || 0)).toBe(true)
  })

  test('centers the region bowl and uses most of the available width', () => {
    const inner = layoutBounds(101, 132)
    const outer = layoutBounds(501, 536)

    expect(Math.abs(inner.centerX - 50) < 2).toBe(true)
    expect(Math.abs(outer.centerX - 50) < 2).toBe(true)
    expect(inner.width > 70).toBe(true)
    expect(outer.width > 88).toBe(true)
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
    }
  })

  test('does not force unknown regions into the two-ring map', () => {
    expect(resolveSeatSwapRegionLayout('VIP')).toBeNull()
  })
})

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
