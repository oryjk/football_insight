import type { MiniReviewStatus } from '../types/system'
import { MINI_PROGRAM_VERSION } from './system'
import { request } from '../utils/request'

// 审核状态登记库由本项目后端提供（GET /api/v1/mini-review/review-status），
// 版本号由发版时 scripts/sync-manifest-version.mjs 通过 /mini-review/allocate 登记。
// 相对路径由 request 工具按 API_BASE_URL 拼接：开发连本地 8092，生产连 match.oryjk.cn。
export const MINI_REVIEW_PROJECT_CODE = 'football_insight_mini'

export function buildMiniReviewStatusUrl(
  projectCode: string = MINI_REVIEW_PROJECT_CODE,
  version: string = MINI_PROGRAM_VERSION,
): string {
  return `/mini-review/review-status?project_code=${encodeURIComponent(projectCode)}&version=${encodeURIComponent(version)}`
}

export function getMiniReviewStatus(
  projectCode: string = MINI_REVIEW_PROJECT_CODE,
  version: string = MINI_PROGRAM_VERSION,
): Promise<MiniReviewStatus> {
  return request<MiniReviewStatus>({ url: buildMiniReviewStatusUrl(projectCode, version) })
}
