import { describe, expect, test } from 'bun:test'
import { lineupDateLabel } from './lineupPresentation'

describe('lineup creation date', () => {
  test('uses Beijing time across a UTC day boundary and respects explicit source offsets', () => {
    expect(lineupDateLabel('2026-10-18T20:05:00Z')).toBe('2026-10-19 04:05')
    expect(lineupDateLabel('2026-10-19T04:05:00+08:00')).toBe('2026-10-19 04:05')
  })
  test('does not display an invalid timestamp', () => {
    expect(lineupDateLabel('invalid')).toBe('')
  })
})
