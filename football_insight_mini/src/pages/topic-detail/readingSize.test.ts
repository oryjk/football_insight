import { describe, expect, test } from 'bun:test'
import { READING_SIZE_ITEMS, normalizeReadingSize, readingSizeRpx } from './readingSize'

describe('reading size', () => {
  test('unknown stored values fall back to standard', () => {
    expect(normalizeReadingSize(undefined)).toBe('standard')
    expect(normalizeReadingSize('huge')).toBe('standard')
    expect(normalizeReadingSize('large')).toBe('large')
  })
  test('each level maps to a larger body size', () => {
    expect(['standard', 'large', 'xlarge'].map(value => readingSizeRpx(normalizeReadingSize(value)))).toEqual([32, 36, 40])
  })
  test('capsule shows every level, all visible at once', () => {
    expect(READING_SIZE_ITEMS.map(item => item.label)).toEqual(['标准', '大', '特大'])
    expect(READING_SIZE_ITEMS.map(item => item.key)).toEqual(['standard', 'large', 'xlarge'])
  })
})
