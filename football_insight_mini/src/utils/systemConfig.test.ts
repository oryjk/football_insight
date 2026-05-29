import { describe, expect, test } from 'bun:test'

import { resolveSystemConfigUnderReview } from './systemConfig'

describe('resolveSystemConfigUnderReview', () => {
  test('treats reviewing status as review mode', () => {
    expect(resolveSystemConfigUnderReview({ is_reviewing: true })).toBe(true)
  })

  test('defaults to normal mode when backend config is absent or not under review', () => {
    expect(resolveSystemConfigUnderReview(null)).toBe(false)
    expect(resolveSystemConfigUnderReview({ is_reviewing: false })).toBe(false)
  })
})
