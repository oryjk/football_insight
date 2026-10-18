import type { MiniReviewStatus } from '../../../types/system'

export interface ReviewStatusView {
  /** 登记库里的真实审核状态；设置页展示与切换都以它为准。 */
  registryReviewing: boolean
  statusText: string
}

/** 旧后端没有 registry_is_reviewing 时退回 is_reviewing（此时也没有豁免）。 */
export function resolveReviewStatusView(status: MiniReviewStatus): ReviewStatusView {
  const registryReviewing = status.registry_is_reviewing ?? status.is_reviewing
  if (status.viewer_is_exempt === true) {
    return { registryReviewing, statusText: `${registryReviewing ? '正在审核' : '已过审'}（本账号已豁免）` }
  }
  return { registryReviewing, statusText: status.status_text }
}
