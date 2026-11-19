import { ref, type Ref } from 'vue'
import * as pageLayoutApi from '../api/pageLayout'
import {
  DEFAULT_PAGE_BLOCK_KEYS,
  createDefaultPageLayout,
  normalizePageLayout,
} from '../config/pageLayoutDefaults'
import type { PageLayoutKey, PublicPageLayout } from '../types/pageLayout'

const STORAGE_KEY = 'fi_page_layouts_v1'
const REFRESH_TTL_MS = 5_000

const TAB_BAR_ROUTES = new Set([
  '/pages/home/index',
  '/pages/rankings/index',
  '/pages/insights/index',
  '/pages/seat-swap/index',
  '/pages/user/index',
])

const VALID_PAGE_KEYS = new Set<string>(Object.keys(DEFAULT_PAGE_BLOCK_KEYS))

export interface PageLayoutPreviewApplyMessage {
  type: 'fi-layout-preview:apply'
  pageKey: PageLayoutKey
  layout: Partial<PublicPageLayout>
}

export interface PageLayoutPreviewNavigateMessage {
  type: 'fi-layout-preview:navigate'
  routePath: string
}

export interface PageLayoutStore {
  readonly layouts: Ref<Record<PageLayoutKey, PublicPageLayout>>
  readonly previewMode: Ref<boolean>
  getLayout(pageKey: PageLayoutKey): PublicPageLayout
  ensurePageLayouts(options?: { force?: boolean }): Promise<void>
  applyPreviewDraft(pageKey: PageLayoutKey, draft: Partial<PublicPageLayout>): void
  clearPreviewDraft(pageKey?: PageLayoutKey): void
  initPreviewBridge(): void
}

function buildDefaultMap(): Record<PageLayoutKey, PublicPageLayout> {
  const map = {} as Record<PageLayoutKey, PublicPageLayout>
  for (const key of Object.keys(DEFAULT_PAGE_BLOCK_KEYS) as PageLayoutKey[]) {
    map[key] = createDefaultPageLayout(key)
  }
  return map
}

function readCachedLayouts(): Partial<Record<PageLayoutKey, PublicPageLayout>> {
  try {
    const raw = uni.getStorageSync(STORAGE_KEY)
    if (!raw || typeof raw !== 'string') return {}
    const parsed = JSON.parse(raw) as Record<string, Partial<PublicPageLayout>>
    if (!parsed || typeof parsed !== 'object') return {}
    const result: Partial<Record<PageLayoutKey, PublicPageLayout>> = {}
    for (const key of Object.keys(parsed)) {
      if (VALID_PAGE_KEYS.has(key)) {
        const pageKey = key as PageLayoutKey
        result[pageKey] = normalizePageLayout(pageKey, parsed[key])
      }
    }
    return result
  } catch {
    return {}
  }
}

function writeCachedLayouts(map: Record<PageLayoutKey, PublicPageLayout>): void {
  try {
    uni.setStorageSync(STORAGE_KEY, JSON.stringify(map))
  } catch {
    // Storage quota/unavailable errors must not block rendering.
  }
}

function detectPreviewMode(): boolean {
  // #ifdef H5
  try {
    const loc = (globalThis as { location?: { search?: string; hash?: string } }).location
    if (!loc) return false
    const search = loc.search ?? ''
    const hash = loc.hash ?? ''
    const queryPart = hash.includes('?') ? hash.slice(hash.indexOf('?')) : ''
    return /(?:^|[?&])layout_preview=1(?:&|$)/.test(search)
      || /(?:^|[?&])layout_preview=1(?:&|$)/.test(queryPart)
  } catch {
    return false
  }
  // #endif
  // #ifndef H5
  return false
  // #endif
}

export function createPageLayoutStore(source: typeof pageLayoutApi = pageLayoutApi): PageLayoutStore {
  const initial = buildDefaultMap()
  const cached = readCachedLayouts()
  for (const key of Object.keys(cached) as PageLayoutKey[]) {
    const entry = cached[key]
    if (entry) initial[key] = entry
  }

  const layouts = ref<Record<PageLayoutKey, PublicPageLayout>>(initial)
  const previewDrafts = ref<Partial<Record<PageLayoutKey, PublicPageLayout>>>({})
  const previewMode = ref<boolean>(detectPreviewMode())

  let lastFetchedAt = 0
  let inFlight: Promise<void> | null = null
  let bridgeBound = false

  function getLayout(pageKey: PageLayoutKey): PublicPageLayout {
    const draft = previewDrafts.value[pageKey]
    if (draft) return draft
    return layouts.value[pageKey] ?? createDefaultPageLayout(pageKey)
  }

  function applyPreviewDraft(pageKey: PageLayoutKey, draft: Partial<PublicPageLayout>): void {
    if (!VALID_PAGE_KEYS.has(pageKey)) return
    previewMode.value = true
    previewDrafts.value = {
      ...previewDrafts.value,
      [pageKey]: normalizePageLayout(pageKey, draft),
    }
  }

  function clearPreviewDraft(pageKey?: PageLayoutKey): void {
    if (!pageKey) {
      previewDrafts.value = {}
      return
    }
    const next = { ...previewDrafts.value }
    delete next[pageKey]
    previewDrafts.value = next
  }

  async function ensurePageLayouts(options?: { force?: boolean }): Promise<void> {
    const force = options?.force === true
    if (!force && lastFetchedAt > 0 && Date.now() - lastFetchedAt < REFRESH_TTL_MS) {
      return
    }
    if (inFlight) {
      return inFlight
    }
    inFlight = (async () => {
      try {
        const res = await source.getPageLayouts()
        if (!res || !Array.isArray(res.items)) return
        const next = buildDefaultMap()
        for (const item of res.items) {
          if (item && VALID_PAGE_KEYS.has(item.page_key)) {
            next[item.page_key] = normalizePageLayout(item.page_key, item)
          }
        }
        layouts.value = next
        writeCachedLayouts(next)
        lastFetchedAt = Date.now()
      } catch {
        // Keep cached or default layouts on transient network failure.
      } finally {
        inFlight = null
      }
    })()
    return inFlight
  }

  function initPreviewBridge(): void {
    // #ifdef H5
    if (bridgeBound) return
    const host = globalThis as unknown as {
      addEventListener?: (event: string, listener: (e: MessageEvent) => void) => void
      parent?: { postMessage?: (data: unknown, targetOrigin: string) => void }
    }
    if (typeof host.addEventListener !== 'function') return
    bridgeBound = true
    previewMode.value = detectPreviewMode()

    host.addEventListener('message', (event: MessageEvent) => {
      const data = event.data as
        | PageLayoutPreviewApplyMessage
        | PageLayoutPreviewNavigateMessage
        | { type?: string }
        | null
      if (!data || typeof data !== 'object') return

      if (data.type === 'fi-layout-preview:apply') {
        const msg = data as PageLayoutPreviewApplyMessage
        if (typeof msg.pageKey === 'string' && VALID_PAGE_KEYS.has(msg.pageKey)) {
          applyPreviewDraft(msg.pageKey, msg.layout)
        }
        return
      }

      if (data.type === 'fi-layout-preview:navigate') {
        const msg = data as PageLayoutPreviewNavigateMessage
        if (typeof msg.routePath === 'string' && msg.routePath.startsWith('/pages/')) {
          if (TAB_BAR_ROUTES.has(msg.routePath)) {
            uni.switchTab({ url: msg.routePath })
          } else {
            uni.reLaunch({ url: msg.routePath })
          }
        }
      }
    })

    if (previewMode.value && host.parent && host.parent !== (globalThis as unknown) && typeof host.parent.postMessage === 'function') {
      host.parent.postMessage({ type: 'fi-layout-preview:ready' }, '*')
    }
    // #endif
  }

  return {
    layouts,
    previewMode,
    getLayout,
    ensurePageLayouts,
    applyPreviewDraft,
    clearPreviewDraft,
    initPreviewBridge,
  }
}

let singleton: PageLayoutStore | null = null

export function usePageLayoutStore(): PageLayoutStore {
  if (!singleton) {
    singleton = createPageLayoutStore()
  }
  return singleton
}
