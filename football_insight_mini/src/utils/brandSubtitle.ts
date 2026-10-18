import { readonly, ref, type Ref } from 'vue'

interface SubtitleStorage {
  getStorageSync(key: string): unknown
  setStorageSync(key: string, value: unknown): void
  removeStorageSync(key: string): void
}

export const DEFAULT_BRAND_SUBTITLE = '最懂球迷的工具'
// 与后端 MAX_BRAND_SUBTITLE_CHARS 保持一致，后端是权威校验。
export const MAX_BRAND_SUBTITLE_CHARS = 20
const STORAGE_KEY = 'fi_brand_subtitle_v1'

export function sanitizeBrandSubtitle(value: unknown): string | undefined {
  if (typeof value !== 'string') return undefined
  const text = value.trim()
  if (!text || [...text].length > MAX_BRAND_SUBTITLE_CHARS) return undefined
  if (/[\u0000-\u001f\u007f-\u009f]/.test(text)) return undefined
  return text
}

export interface BrandSubtitleStore {
  readonly subtitle: Readonly<Ref<string>>
  /** undefined = 旧后端未返回该字段，保持现状；null/非法 = 恢复默认并清缓存。 */
  apply(value: unknown): void
}

/**
 * 品牌栏副标题：上次成功拉取的值缓存在本地，冷启动第一帧就能显示，
 * 公共配置回来后再刷新；存储不可用时退回内置默认文案，不影响渲染。
 */
export function createBrandSubtitle(storage?: SubtitleStorage): BrandSubtitleStore {
  const target = (): SubtitleStorage => storage ?? uni
  function readCached(): string | undefined {
    try {
      return sanitizeBrandSubtitle(target().getStorageSync(STORAGE_KEY))
    } catch {
      return undefined
    }
  }
  const current = ref(readCached() ?? DEFAULT_BRAND_SUBTITLE)

  function apply(value: unknown) {
    if (value === undefined) return
    const clean = sanitizeBrandSubtitle(value)
    current.value = clean ?? DEFAULT_BRAND_SUBTITLE
    try {
      if (clean) target().setStorageSync(STORAGE_KEY, clean)
      else target().removeStorageSync(STORAGE_KEY)
    } catch {
      console.warn('brand subtitle storage unavailable')
    }
  }

  return { subtitle: readonly(current), apply }
}

const shared = createBrandSubtitle()
export const brandSubtitle = shared.subtitle
export const applyBrandSubtitle = shared.apply
