import { describe, expect, test } from 'bun:test'
import { buildOutcomeDots, buildSeasonDots, CSL_SEASON_TOTAL_ROUNDS, padUpcomingDots } from './resultDots'

describe('result dots', () => {
  test('keeps the newest valid results in oldest-to-newest order', () => {
    const dots = buildOutcomeDots(['loss', 'win', null, 'live', 'draw', 'win'], 3)
    expect(dots.map(dot => dot.tone)).toEqual(['win', 'draw', 'win'])
  })

  test('returns nothing for missing or invalid records', () => {
    expect(buildOutcomeDots(undefined)).toEqual([])
    expect(buildOutcomeDots(['live', 'scheduled'])).toEqual([])
  })

  test('season dots pad unplayed rounds with upcoming placeholders to 30', () => {
    const dots = buildSeasonDots(['win', 'loss', 'draw'])
    expect(dots.length).toBe(CSL_SEASON_TOTAL_ROUNDS)
    expect(dots.map(dot => dot.tone).slice(0, 3)).toEqual(['win', 'loss', 'draw'])
    expect(dots.slice(3).every(dot => dot.tone === 'upcoming')).toBe(true)
  })

  test('season dots never exceed the total and handle empty records', () => {
    expect(buildSeasonDots(undefined).length).toBe(CSL_SEASON_TOTAL_ROUNDS)
    const full = Array.from({ length: CSL_SEASON_TOTAL_ROUNDS }, () => 'win')
    expect(buildSeasonDots(full).length).toBe(CSL_SEASON_TOTAL_ROUNDS)
  })

  test('padUpcomingDots keeps played ids untouched for interactive sheets', () => {
    const dots = padUpcomingDots([{ id: 12345, tone: 'win' }, { id: 12346, tone: 'loss' }], 4)
    expect(dots.map(dot => dot.id)).toEqual([12345, 12346, 'upcoming-0', 'upcoming-1'])
  })
})
