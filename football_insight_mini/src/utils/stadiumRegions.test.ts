import { describe, expect, test } from 'bun:test'

import {
  formatSeatSwapDesiredSeats,
  formatSeatSwapSeatLabel,
  resolveSeatSwapRegionLayout,
  resolveSeatSwapRegionColorGroup,
  seatSwapStatusLabel,
} from './stadiumRegions'

describe('stadium region utilities', () => {
  test('places Phoenix Hill seat swap rings in a centered stadium bowl', () => {
    const inner = resolveSeatSwapRegionLayout('101')
    const outer = resolveSeatSwapRegionLayout('501')
    const top = resolveSeatSwapRegionLayout('514')

    expect(inner?.ring).toBe('inner')
    expect(inner?.side).toBe('right')
    expect(outer?.ring).toBe('outer')
    expect(outer?.side).toBe('right')
    expect((outer?.left || 0) > (inner?.left || 0)).toBe(true)
    expect((top?.top || 0) < (inner?.top || 0)).toBe(true)
  })

  test('exposes Chengdu seat region color groups for shared stadium rendering', () => {
    expect(resolveSeatSwapRegionColorGroup('511')).toBe('blue')
    expect(resolveSeatSwapRegionColorGroup('116')).toBe('green')
    expect(resolveSeatSwapRegionColorGroup('505')).toBe('purple')
    expect(resolveSeatSwapRegionColorGroup('130')).toBe('yellow')
    expect(resolveSeatSwapRegionColorGroup('118')).toBe('navy')
    expect(resolveSeatSwapRegionColorGroup('520')).toBe('navy')
    expect(resolveSeatSwapRegionColorGroup('127')).toBe('red')
    expect(resolveSeatSwapRegionColorGroup('VIP1')).toBe('vip')
    expect(resolveSeatSwapRegionColorGroup('VIP')).toBe('vip')
  })

  test('formats seat swap labels consistently across page and card components', () => {
    expect(formatSeatSwapSeatLabel({
      current_region_name: '101',
      current_row: '',
      current_seat_no: '',
    })).toBe('101')
    expect(formatSeatSwapSeatLabel({
      current_region_name: '101',
      current_row: '8',
      current_seat_no: '15',
    })).toBe('101 8排 15号')
    expect(formatSeatSwapDesiredSeats([
      { region_name: '102', desired_row: '', desired_seat_no: '' },
      { region_name: '103', desired_row: '9', desired_seat_no: '10' },
    ])).toBe('102、103 9排 10号')
    expect(seatSwapStatusLabel('peer_confirmed_me')).toBe('对方已确认你')
    expect(seatSwapStatusLabel('display_only')).toBe('')
  })
})
