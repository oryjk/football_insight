import { describe, expect, test } from 'bun:test'
import { buildOutcomeDots } from './resultDots'

describe('result dots', () => {
  test('keeps the newest valid results in oldest-to-newest order', () => {
    const dots = buildOutcomeDots(['loss', 'win', null, 'live', 'draw', 'win'], 3)
    expect(dots.map(dot => dot.tone)).toEqual(['win', 'draw', 'win'])
  })

  test('returns nothing for missing or invalid records', () => {
    expect(buildOutcomeDots(undefined)).toEqual([])
    expect(buildOutcomeDots(['live', 'scheduled'])).toEqual([])
  })
})
