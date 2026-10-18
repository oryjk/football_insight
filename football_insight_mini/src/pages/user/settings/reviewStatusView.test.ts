import { describe, expect, test } from 'bun:test'

import type { MiniReviewStatus } from '../../../types/system'
import { resolveReviewStatusView } from './reviewStatusView'

function status(overrides: Partial<MiniReviewStatus>): MiniReviewStatus {
  return { project_code: 'football_insight_mini', version: '1.0.99', is_reviewing: true, status_text: '正在审核', ...overrides }
}

describe('resolveReviewStatusView', () => {
  test('exempt account still sees and edits the real registry status', () => {
    const view = resolveReviewStatusView(status({
      is_reviewing: false,
      registry_is_reviewing: true,
      viewer_is_reviewing: false,
      viewer_is_exempt: true,
      status_text: '审核豁免账号',
    }))
    expect(view).toEqual({ registryReviewing: true, statusText: '正在审核（本账号已豁免）' })
  })

  test('ordinary accounts keep the server text', () => {
    const view = resolveReviewStatusView(status({ registry_is_reviewing: true, viewer_is_exempt: false }))
    expect(view).toEqual({ registryReviewing: true, statusText: '正在审核' })
  })

  test('old backends without the explicit fields fall back to is_reviewing', () => {
    const view = resolveReviewStatusView(status({ is_reviewing: false, status_text: '审核通过' }))
    expect(view).toEqual({ registryReviewing: false, statusText: '审核通过' })
  })
})
