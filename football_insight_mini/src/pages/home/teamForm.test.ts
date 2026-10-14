import { describe, expect, test } from 'bun:test'
import { buildHomeTeamForm } from './teamForm'

describe('home team form', () => {
  test('puts wins above draws and losses and preserves oldest-to-newest order', () => {
    const form = buildHomeTeamForm(['loss', 'draw', 'win', 'win', 'draw', 'loss'])
    expect(form.points.map(point => point.top)).toEqual([80, 50, 20, 20, 50, 80])
    expect(form.segments.map(segment => segment.direction)).toEqual(['up', 'up', 'flat', 'down', 'down'])
    expect(form.latestLabel).toBe('负')
    expect(form.description).toBe('近6场，从旧到新：负、平、胜、胜、平、负')
  })

  test('keeps only the newest six valid results without manufacturing matches', () => {
    const form = buildHomeTeamForm(['loss', 'loss', 'win', null, 'draw', 'live', 'win', 'draw', 'win'])
    expect(form.points.map(point => point.result)).toEqual(['loss', 'win', 'draw', 'win', 'draw', 'win'])
    expect(form.latestLabel).toBe('胜')
  })

  test('handles missing data and a single match without a fake connecting line', () => {
    expect(buildHomeTeamForm(undefined).points).toEqual([])
    expect(buildHomeTeamForm([]).latestLabel).toBe('')
    const form = buildHomeTeamForm(['draw'])
    expect(form.points.length).toBe(1)
    expect(form.points[0]?.left).toBe(100)
    expect(form.segments).toEqual([])
    expect(form.description).toBe('近1场，从旧到新：平')
  })
})
