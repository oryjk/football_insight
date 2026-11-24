import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import {
  DEFAULT_PAGE_BLOCK_KEYS,
  createDefaultPageLayout,
  normalizePageLayout,
  sanitizeImageUrl,
} from '../config/pageLayoutDefaults'
import type * as pageLayoutApi from '../api/pageLayout'
import type { PageLayoutKey, PublicPageLayout } from '../types/pageLayout'
import { createPageLayoutStore } from './pageLayout'

const storage = new Map<string, string>()

beforeEach(() => {
  storage.clear()
  ;(globalThis as unknown as { uni: Record<string, unknown> }).uni = {
    getStorageSync: (key: string) => storage.get(key) ?? '',
    setStorageSync: (key: string, value: string) => {
      storage.set(key, value)
    },
    switchTab: () => undefined,
    reLaunch: () => undefined,
  }
})

afterEach(() => {
  storage.clear()
})

describe('pageLayoutDefaults', () => {
  test('registers all 25 mini-program pages with non-empty unique block keys', () => {
    const keys = Object.keys(DEFAULT_PAGE_BLOCK_KEYS) as PageLayoutKey[]
    expect(keys.length).toBe(25)
    for (const key of keys) {
      const blocks = DEFAULT_PAGE_BLOCK_KEYS[key]
      expect(blocks.length > 0).toBe(true)
      expect(new Set(blocks).size).toBe(blocks.length)
    }
  })

  test('sanitizeImageUrl allows only trimmed http(s) URLs without whitespace', () => {
    expect(sanitizeImageUrl('https://cdn.example.com/bg.png')).toBe('https://cdn.example.com/bg.png')
    expect(sanitizeImageUrl('  http://cdn.example.com/a.jpg  ')).toBe('http://cdn.example.com/a.jpg')
    expect(sanitizeImageUrl('https://oryjk.cn:82/minio/football-insight/bg.webp')).toBe(
      'https://oryjk.cn:18443/minio/football-insight/bg.webp',
    )
    expect(sanitizeImageUrl('javascript:alert(1)')).toBeNull()
    expect(sanitizeImageUrl('data:image/png;base64,abc')).toBeNull()
    expect(sanitizeImageUrl('https://cdn.example.com/a b.png')).toBeNull()
    expect(sanitizeImageUrl('   ')).toBeNull()
    expect(sanitizeImageUrl(null)).toBeNull()
  })

  test('normalizePageLayout preserves order, all-hidden state, backgrounds, and appends missing defaults', () => {
    const normalized = normalizePageLayout('home', {
      page_key: 'home',
      background_image_url: 'https://cdn.example.com/home-bg.jpg',
      blocks: [
        { key: 'my_team_support', visible: true, background_image_url: 'https://cdn.example.com/card.png' },
        { key: 'unknown_legacy_card', visible: true, background_image_url: null },
        { key: 'daily_topics', visible: false, background_image_url: 'javascript:bad' },
        { key: 'my_team_support', visible: false, background_image_url: null },
      ],
      version: 3,
      updated_at: '2026-11-19T00:00:00Z',
    })

    expect(normalized.background_image_url).toBe('https://cdn.example.com/home-bg.jpg')
    expect(normalized.version).toBe(3)
    expect(normalized.blocks.map((b) => b.key)).toEqual([
      'my_team_support',
      'daily_topics',
      'fan_entries',
      'story_pulse',
      'standings',
      'top_scorers',
    ])
    expect(normalized.blocks[0]).toEqual({
      key: 'my_team_support',
      visible: true,
      background_image_url: 'https://cdn.example.com/card.png',
    })
    expect(normalized.blocks[1]).toEqual({
      key: 'daily_topics',
      visible: false,
      background_image_url: null,
    })
    expect(normalized.blocks[2].visible).toBe(true)
  })

  test('normalizePageLayout permits hiding every block on a page', () => {
    const allHidden = DEFAULT_PAGE_BLOCK_KEYS.seat_swap.map((key) => ({
      key,
      visible: false,
      background_image_url: null,
    }))
    const normalized = normalizePageLayout('seat_swap', { blocks: allHidden })
    expect(normalized.blocks.every((b) => !b.visible)).toBe(true)
  })

  test('defaults built-in stadium MinIO background and honors explicit null clear', () => {
    const defaultHome = createDefaultPageLayout('home')
    expect(defaultHome.background_image_url).toBe(
      'https://oryjk.cn:18443/minio/football-insight/static/user/phoenix-stadium-bg.webp',
    )
    const defaultTopics = createDefaultPageLayout('topics')
    expect(defaultTopics.background_image_url).toBeNull()

    const clearedHome = normalizePageLayout('home', {
      background_image_url: null,
      blocks: defaultHome.blocks,
    })
    expect(clearedHome.background_image_url).toBeNull()
  })
})

describe('createPageLayoutStore', () => {
  test('fetches layouts, caches them, and lets preview draft override without mutating saved layouts', async () => {
    const serverHome: PublicPageLayout = {
      ...createDefaultPageLayout('home'),
      background_image_url: 'https://cdn.example.com/server-bg.jpg',
      blocks: [
        { key: 'my_team_support', visible: true, background_image_url: null },
        { key: 'daily_topics', visible: true, background_image_url: null },
        { key: 'fan_entries', visible: false, background_image_url: null },
        { key: 'story_pulse', visible: true, background_image_url: null },
        { key: 'standings', visible: true, background_image_url: null },
        { key: 'top_scorers', visible: true, background_image_url: null },
      ],
      version: 2,
      updated_at: '2026-11-19T01:00:00Z',
    }

    const source = {
      getPageLayouts: async () => ({ items: [serverHome] }),
      getPageLayout: async () => serverHome,
    } as unknown as typeof pageLayoutApi

    const store = createPageLayoutStore(source)
    await store.ensurePageLayouts({ force: true })

    expect(store.getLayout('home').blocks[0].key).toBe('my_team_support')
    expect(store.getLayout('home').background_image_url).toBe('https://cdn.example.com/server-bg.jpg')
    expect(storage.has('fi_page_layouts_v1')).toBe(true)

    store.applyPreviewDraft('home', {
      background_image_url: 'https://cdn.example.com/preview-bg.jpg',
      blocks: [
        { key: 'standings', visible: true, background_image_url: 'https://cdn.example.com/standings.png' },
        { key: 'my_team_support', visible: false, background_image_url: null },
      ],
    })

    expect(store.previewMode.value).toBe(true)
    expect(store.getLayout('home').background_image_url).toBe('https://cdn.example.com/preview-bg.jpg')
    expect(store.getLayout('home').blocks[0].key).toBe('standings')
    expect(store.getLayout('home').blocks[0].background_image_url).toBe('https://cdn.example.com/standings.png')
    expect(store.getLayout('home').blocks[1]).toEqual({
      key: 'my_team_support',
      visible: false,
      background_image_url: null,
    })

    store.clearPreviewDraft('home')
    expect(store.getLayout('home').background_image_url).toBe('https://cdn.example.com/server-bg.jpg')
    expect(store.getLayout('home').blocks[0].key).toBe('my_team_support')
  })
})
