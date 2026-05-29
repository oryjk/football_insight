import { getMiniReviewStatus } from '../api/miniReview'
import type { MiniReviewStatus } from '../types/system'

export function resolveSystemConfigUnderReview(config: Pick<MiniReviewStatus, 'is_reviewing'> | null): boolean {
  return Boolean(config?.is_reviewing)
}

export async function loadSystemConfig(): Promise<MiniReviewStatus | null> {
  return getMiniReviewStatus().catch((error) => {
    console.warn('[mini-review] load failed', error)
    return null
  })
}

export async function loadSystemConfigUnderReview(): Promise<boolean> {
  return resolveSystemConfigUnderReview(await loadSystemConfig())
}

export function resetSystemConfigCacheForTest(): void {
  // 保留给测试重置调用，当前实现不做跨页面长期缓存。
}
