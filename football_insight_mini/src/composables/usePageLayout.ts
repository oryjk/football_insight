import { computed, type ComputedRef } from 'vue'
import { usePageLayoutStore } from '../stores/pageLayout'
import type { PageLayoutBlock, PageLayoutKey, PublicPageLayout } from '../types/pageLayout'

export interface UsePageLayoutResult {
  layout: ComputedRef<PublicPageLayout>
  pageBackgroundImageUrl: ComputedRef<string | null>
  visibleBlocks: ComputedRef<PageLayoutBlock[]>
  allBlocksHidden: ComputedRef<boolean>
  isPreviewMode: ComputedRef<boolean>
  isBlockVisible: (blockKey: string) => boolean
  blockBackground: (blockKey: string) => string | null
  refreshLayout: (options?: { force?: boolean }) => Promise<void>
}

export function usePageLayout(pageKey: PageLayoutKey): UsePageLayoutResult {
  const store = usePageLayoutStore()

  const layout = computed(() => store.getLayout(pageKey))
  const pageBackgroundImageUrl = computed(() => layout.value.background_image_url)
  const visibleBlocks = computed(() => layout.value.blocks.filter((b) => b.visible))
  const allBlocksHidden = computed(() => visibleBlocks.value.length === 0)
  const isPreviewMode = computed(() => store.previewMode.value)

  function isBlockVisible(blockKey: string): boolean {
    const found = layout.value.blocks.find((b) => b.key === blockKey)
    return found ? found.visible : true
  }

  function blockBackground(blockKey: string): string | null {
    const found = layout.value.blocks.find((b) => b.key === blockKey)
    return found?.background_image_url ?? null
  }

  function refreshLayout(options?: { force?: boolean }): Promise<void> {
    return store.ensurePageLayouts(options)
  }

  return {
    layout,
    pageBackgroundImageUrl,
    visibleBlocks,
    allBlocksHidden,
    isPreviewMode,
    isBlockVisible,
    blockBackground,
    refreshLayout,
  }
}
