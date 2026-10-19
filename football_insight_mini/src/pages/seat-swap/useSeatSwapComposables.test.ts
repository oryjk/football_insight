import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'
import { useSeatSwapBrowse } from './useSeatSwapBrowse'
import { useSeatSwapPage } from './useSeatSwapPage'
import { SEAT_SWAP_POOL_ANCHOR_ID } from './helpers'
import type { SeatSwapCurrentResponse } from '../../types/seatSwap'

function setupBrowse(
  view: SeatSwapCurrentResponse | null = buildSeatSwapMockCurrentResponse({ candidateCount: 12, includeMyRequest: true }),
) {
  const currentView = ref<SeatSwapCurrentResponse | null>(view)
  const regions = ref(buildSeatSwapMockRegions())
  const sheetOpen = ref(false)
  const browse = useSeatSwapBrowse({ currentView, regions, sheetOpen })
  return { currentView, regions, sheetOpen, browse }
}

describe('useSeatSwapPage', () => {
  test('loads the current view and regions, then calls onLoaded', async () => {
    const view = buildSeatSwapMockCurrentResponse({ candidateCount: 3 })
    let loadedCalls = 0
    const page = useSeatSwapPage({
      loaders: {
        getCurrentSeatSwap: async () => view,
        getTicketWatchRegions: async () => buildSeatSwapMockRegions(),
      },
      onLoaded: () => { loadedCalls += 1 },
    })
    expect(page.loading.value).toBe(true)
    await page.loadPage()
    expect(page.loading.value).toBe(false)
    expect(page.errorMessage.value).toBe('')
    expect(page.currentView.value).toEqual(view)
    expect(page.regions.value.length > 0).toBe(true)
    expect(loadedCalls).toBe(1)
  })

  test('surfaces the error message and skips onLoaded when loading fails', async () => {
    let loadedCalls = 0
    const page = useSeatSwapPage({
      loaders: {
        getCurrentSeatSwap: async () => { throw new Error('boom') },
        getTicketWatchRegions: async () => [],
      },
      onLoaded: () => { loadedCalls += 1 },
    })
    await page.loadPage()
    expect(page.loading.value).toBe(false)
    expect(page.errorMessage.value).toBe('boom')
    expect(loadedCalls).toBe(0)
  })

  test('falls back to the default message when the error has no message', async () => {
    const page = useSeatSwapPage({
      loaders: {
        getCurrentSeatSwap: async () => { throw new Error('') },
        getTicketWatchRegions: async () => [],
      },
    })
    await page.loadPage()
    expect(page.errorMessage.value).toBe('换座池加载失败')
  })

  test('uses mock data without calling loaders when useMock is set', async () => {
    const page = useSeatSwapPage({
      useMock: true,
      loaders: {
        getCurrentSeatSwap: async () => { throw new Error('should not be called') },
        getTicketWatchRegions: async () => { throw new Error('should not be called') },
      },
    })
    await page.loadPage()
    expect(page.errorMessage.value).toBe('')
    expect(page.currentView.value?.candidates.length).toBe(96)
  })
})

describe('useSeatSwapBrowse', () => {
  test('hides my own request from the pool', () => {
    const { currentView, browse } = setupBrowse()
    const myId = currentView.value?.my_request?.request_id
    expect(typeof myId).toBe('string')
    expect(browse.displayCandidates.value.some((c) => c.request_id === myId)).toBe(false)
  })

  test('main-map tap on a region with candidates filters and asks to scroll', () => {
    const { browse } = setupBrowse()
    const key = Object.keys(browse.regionBadgeCounts.value)[0]
    const outcome = browse.applyMapTap(key, false)
    expect(outcome).toEqual({ filterKey: key, closeSheet: false, scrollTo: SEAT_SWAP_POOL_ANCHOR_ID })
    expect(browse.browsingFilterKey.value).toBe(key)
    expect(browse.mainMapMode.value).toBe('filter')
    expect(browse.mapFabLabel.value.clearable).toBe(true)
  })

  test('clearFilter resets the floating button label', () => {
    const { browse } = setupBrowse()
    const key = Object.keys(browse.regionBadgeCounts.value)[0]
    browse.applyMapTap(key, true)
    browse.clearFilter()
    expect(browse.browsingFilterKey.value).toBe('')
    expect(browse.mapFabLabel.value).toEqual({ text: '地图', clearable: false })
  })

  test('falls back to the region key when the filtered region is unknown', () => {
    const { browse } = setupBrowse()
    browse.applyMapTap('no-such-region', false)
    expect(browse.browsingFilterName.value).toBe('no-such-region')
    expect(browse.filteredCandidates.value).toEqual([])
  })

  test('floating button follows map visibility, sheets and availability', () => {
    const { currentView, sheetOpen, browse } = setupBrowse()
    expect(currentView.value?.available).toBe(true)
    expect(browse.mapInViewport.value).toBe(true)
    expect(browse.mapFabVisible.value).toBe(false)
    browse.mapInViewport.value = false
    expect(browse.mapFabVisible.value).toBe(true)
    sheetOpen.value = true
    expect(browse.mapFabVisible.value).toBe(false)
    sheetOpen.value = false
    currentView.value = currentView.value ? { ...currentView.value, available: false } : null
    expect(browse.mapFabVisible.value).toBe(false)
  })

  test('expands every group by default and keeps expansion on toggle', () => {
    const { browse } = setupBrowse()
    browse.syncExpandedRegionKeys()
    const keys = browse.seatSwapRegionGroups.value.map((g) => g.region_key)
    expect(browse.expandedRegionKeys.value).toEqual(keys)
    browse.toggleRegionGroup(keys[0])
    expect(browse.expandedRegionKeys.value.includes(keys[0])).toBe(false)
    browse.toggleRegionGroup(keys[0])
    expect(browse.expandedRegionKeys.value.includes(keys[0])).toBe(true)
  })
})
