import { describe, expect, test } from 'bun:test'

import { segmentedThumbStyle } from './segmented'

describe('segmentedThumbStyle', () => {
  test('width splits the track evenly after the outer padding and the gaps between segments', () => {
    expect(segmentedThumbStyle(2, 0).includes('width: calc((100% - var(--fi-space-6) * 3) / 2)')).toBe(true)
    expect(segmentedThumbStyle(3, 0).includes('width: calc((100% - var(--fi-space-6) * 4) / 3)')).toBe(true)
  })

  test('the first segment sits at the start and later ones shift by thumb width plus gap', () => {
    expect(segmentedThumbStyle(3, 0).includes('translateX(calc(0% + 0 * var(--fi-space-6)))')).toBe(true)
    expect(segmentedThumbStyle(3, 1).includes('translateX(calc(100% + 1 * var(--fi-space-6)))')).toBe(true)
    expect(segmentedThumbStyle(3, 2).includes('translateX(calc(200% + 2 * var(--fi-space-6)))')).toBe(true)
  })

  test('works for any number of segments, not just two or three', () => {
    expect(segmentedThumbStyle(5, 4).includes('translateX(calc(400% + 4 * var(--fi-space-6)))')).toBe(true)
    expect(segmentedThumbStyle(5, 4).includes('* 6) / 5)')).toBe(true)
  })

  test('an unknown selection hides the thumb instead of pointing at a wrong segment', () => {
    expect(segmentedThumbStyle(3, -1).includes('opacity: 0')).toBe(true)
    expect(segmentedThumbStyle(3, 3).includes('opacity: 0')).toBe(true)
  })

  test('no segments yields no style', () => {
    expect(segmentedThumbStyle(0, 0)).toBe('')
  })
})
