import { useMiniReviewStatus } from '../stores/miniReview'
import type { MiniReviewStatus } from '../types/system'

export function resolveSystemConfigUnderReview(config: Pick<MiniReviewStatus, 'is_reviewing'> | null): boolean {
  return Boolean(config?.is_reviewing)
}

/**
 * 兼容入口：委托 miniReview 共享 store（并发去重 + 60 秒成功缓存）。
 * 查询失败返回 null（isUnderReview 视为 false），不能用旧成功值冒充本次确认成功。
 */
export async function loadSystemConfig(): Promise<MiniReviewStatus | null> {
  return useMiniReviewStatus().ensureMiniReviewReady()
}

export async function loadSystemConfigUnderReview(): Promise<boolean> {
  return resolveSystemConfigUnderReview(await loadSystemConfig())
}

export function resetSystemConfigCacheForTest(): void {
  // 保留给测试重置调用；共享 store 的行为测试通过 create 注入独立实例完成。
}
