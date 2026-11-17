import { describe, expect, test } from 'bun:test'

import { telegraphCalendarLabel } from './telegraphCalendar'

const NOW = Date.parse('2026-11-17T12:00:00+08:00')

describe('telegraphCalendarLabel', () => {
  test('marks a same-day Beijing edition as today', () => {
    expect(telegraphCalendarLabel('2026-11-17T00:10:00+08:00', NOW)).toEqual({ label: '今天', isToday: true })
  })

  test('uses Beijing calendar days rather than a rolling 24-hour window', () => {
    const now = Date.parse('2026-11-17T00:30:00+08:00')
    expect(telegraphCalendarLabel('2026-11-16T23:30:00+08:00', now)).toEqual({ label: '昨天', isToday: false })
  })

  test('formats older editions with their month and day', () => {
    expect(telegraphCalendarLabel('2026-11-15T23:59:00+08:00', NOW)).toEqual({ label: '11月15日', isToday: false })
  })

  test('includes the year when an edition is from a different year', () => {
    const now = Date.parse('2026-01-01T08:00:00+08:00')
    expect(telegraphCalendarLabel('2025-12-30T08:00:00+08:00', now)).toEqual({ label: '2025年12月30日', isToday: false })
  })
})
