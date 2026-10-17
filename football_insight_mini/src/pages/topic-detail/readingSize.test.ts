import { describe, expect, test } from 'bun:test'
import { normalizeReadingSize, readingSizeMenu, readingSizeRpx } from './readingSize'

describe('reading size', () => {
  test('unknown stored values fall back to standard', () => {
    expect(normalizeReadingSize(undefined)).toBe('standard')
    expect(normalizeReadingSize('huge')).toBe('standard')
    expect(normalizeReadingSize('large')).toBe('large')
  })
  test('each level maps to a larger body size', () => {
    expect(['standard', 'large', 'xlarge'].map(value => readingSizeRpx(normalizeReadingSize(value)))).toEqual([32, 36, 40])
  })
  test('menu marks only the current level', () => {
    expect(readingSizeMenu('large')).toEqual(['标准', '大（当前）', '特大'])
  })
})
