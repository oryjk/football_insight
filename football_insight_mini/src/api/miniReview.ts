import type { MiniReviewStatus } from '../types/system'
import { MINI_PROGRAM_VERSION } from './system'
import { request } from '../utils/request'

const MINI_REVIEW_BASE_URL = 'https://match.oryjk.cn/mini-review'
export const MINI_REVIEW_PROJECT_CODE = 'football_insight_mini'

export function buildMiniReviewStatusUrl(
  projectCode: string = MINI_REVIEW_PROJECT_CODE,
  version: string = MINI_PROGRAM_VERSION,
): string {
  return `${MINI_REVIEW_BASE_URL}/api/public/review-status?project_code=${encodeURIComponent(projectCode)}&version=${encodeURIComponent(version)}`
}

export function getMiniReviewStatus(
  projectCode: string = MINI_REVIEW_PROJECT_CODE,
  version: string = MINI_PROGRAM_VERSION,
): Promise<MiniReviewStatus> {
  return request<MiniReviewStatus>({ url: buildMiniReviewStatusUrl(projectCode, version) })
}
