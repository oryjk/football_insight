import { expect, test } from 'bun:test'
import {
  DEFAULT_BRAND_SUBTITLE,
  MAX_BRAND_SUBTITLE_CHARS,
  createBrandSubtitle,
  sanitizeBrandSubtitle,
} from './brandSubtitle'

function memoryStorage(initial: Record<string, unknown> = {}) {
  const data = { ...initial }
  return {
    data,
    getStorageSync: (key: string) => data[key],
    setStorageSync: (key: string, value: unknown) => { data[key] = value },
    removeStorageSync: (key: string) => { delete data[key] },
  }
}

test('sanitize trims and rejects blank, control characters, overlong and non-strings', () => {
  expect(sanitizeBrandSubtitle('  球迷的数据主场 ')).toBe('球迷的数据主场')
  expect(sanitizeBrandSubtitle('   ')).toBe(undefined)
  expect(sanitizeBrandSubtitle('a\nb')).toBe(undefined)
  expect(sanitizeBrandSubtitle(12)).toBe(undefined)
  expect(sanitizeBrandSubtitle(null)).toBe(undefined)
  expect(sanitizeBrandSubtitle('蓉'.repeat(MAX_BRAND_SUBTITLE_CHARS)) !== undefined).toBe(true)
  expect(sanitizeBrandSubtitle('蓉'.repeat(MAX_BRAND_SUBTITLE_CHARS + 1))).toBe(undefined)
})

test('starts from the cached subtitle so the first frame already shows it', () => {
  const store = createBrandSubtitle(memoryStorage({ fi_brand_subtitle_v1: '主场就是家' }))
  expect(store.subtitle.value).toBe('主场就是家')
  expect(createBrandSubtitle(memoryStorage()).subtitle.value).toBe(DEFAULT_BRAND_SUBTITLE)
})

test('a configured value is applied and cached; null restores the default and clears the cache', () => {
  const storage = memoryStorage()
  const store = createBrandSubtitle(storage)
  store.apply('新文案')
  expect(store.subtitle.value).toBe('新文案')
  expect(storage.data.fi_brand_subtitle_v1).toBe('新文案')
  store.apply(null)
  expect(store.subtitle.value).toBe(DEFAULT_BRAND_SUBTITLE)
  expect('fi_brand_subtitle_v1' in storage.data).toBe(false)
})

test('an old backend without the field leaves the current subtitle untouched', () => {
  const storage = memoryStorage({ fi_brand_subtitle_v1: '已缓存' })
  const store = createBrandSubtitle(storage)
  store.apply(undefined)
  expect(store.subtitle.value).toBe('已缓存')
})

test('storage failures never break rendering', () => {
  const broken = {
    getStorageSync: () => { throw new Error('denied') },
    setStorageSync: () => { throw new Error('denied') },
    removeStorageSync: () => { throw new Error('denied') },
  }
  const store = createBrandSubtitle(broken)
  expect(store.subtitle.value).toBe(DEFAULT_BRAND_SUBTITLE)
  store.apply('可用')
  expect(store.subtitle.value).toBe('可用')
})
