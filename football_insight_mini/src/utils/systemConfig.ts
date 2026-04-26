import { getSystemConfig } from '../api/system'
import type { SystemConfig } from '../types/system'

export function resolveSystemConfigUnderReview(config: Pick<SystemConfig, 'is_under_review' | 'matched'> | null): boolean {
  return Boolean(config?.matched && config.is_under_review)
}

export async function loadSystemConfig(): Promise<SystemConfig | null> {
  return getSystemConfig().catch((error) => {
    console.warn('[system_config] load failed', error)
    return null
  })
}

export async function loadSystemConfigUnderReview(): Promise<boolean> {
  return resolveSystemConfigUnderReview(await loadSystemConfig())
}

export function resetSystemConfigCacheForTest(): void {
  // 保留给测试重置调用，当前实现不做跨页面长期缓存。
}
