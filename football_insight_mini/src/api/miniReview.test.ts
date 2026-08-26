import { describe, expect, test } from 'bun:test'

import { buildMiniReviewStatusUrl } from './miniReview'

describe('buildMiniReviewStatusUrl', () => {
  test('uses project_code and version query params on the relative backend path', () => {
    expect(buildMiniReviewStatusUrl('football_insight_mini', '1.0.51')).toBe(
      '/mini-review/review-status?project_code=football_insight_mini&version=1.0.51',
    )
  })

  test('encodes query values', () => {
    expect(buildMiniReviewStatusUrl('football insight mini', '1.0.51 beta')).toBe(
      '/mini-review/review-status?project_code=football%20insight%20mini&version=1.0.51%20beta',
    )
  })
})
