import { describe, expect, test } from 'bun:test'

import { resolveSystemConfigUnderReview } from './systemConfig'

describe('resolveSystemConfigUnderReview', () => {
  test('treats matched under-review backend config as review mode', () => {
    expect(resolveSystemConfigUnderReview({ is_under_review: true, matched: true })).toBe(true)
  })

  test('defaults to normal mode when backend config is absent or not under review', () => {
    expect(resolveSystemConfigUnderReview(null)).toBe(false)
    expect(resolveSystemConfigUnderReview({ is_under_review: false, matched: true })).toBe(false)
    expect(resolveSystemConfigUnderReview({ is_under_review: true, matched: false })).toBe(false)
  })
})
