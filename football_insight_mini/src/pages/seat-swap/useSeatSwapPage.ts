import { ref } from 'vue'
import { getCurrentSeatSwap } from '../../api/seatSwap'
import { getTicketWatchRegions } from '../../api/ticketWatch'
import type { SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { extractApiErrorMessage } from '../../utils/apiError'
import { buildSeatSwapMockCurrentResponse, buildSeatSwapMockRegions } from './fixtures'

export interface SeatSwapPageLoaders {
  getCurrentSeatSwap: () => Promise<SeatSwapCurrentResponse>
  getTicketWatchRegions: () => Promise<TicketWatchRegion[]>
}

const DEFAULT_LOADERS: SeatSwapPageLoaders = { getCurrentSeatSwap, getTicketWatchRegions }

/** 换座页数据层：唯一调用接口的地方。 */
export function useSeatSwapPage(options: {
  loaders?: SeatSwapPageLoaders
  useMock?: boolean
  onLoaded?: () => void
} = {}) {
  const loaders = options.loaders ?? DEFAULT_LOADERS
  const loading = ref(true)
  const errorMessage = ref('')
  const currentView = ref<SeatSwapCurrentResponse | null>(null)
  const regions = ref<TicketWatchRegion[]>([])

  async function loadPage(): Promise<void> {
    loading.value = true
    errorMessage.value = ''
    try {
      if (options.useMock) {
        currentView.value = buildSeatSwapMockCurrentResponse({ candidateCount: 96, includeMyRequest: true })
        regions.value = buildSeatSwapMockRegions()
      } else {
        const [view, regionList] = await Promise.all([
          loaders.getCurrentSeatSwap(),
          loaders.getTicketWatchRegions(),
        ])
        currentView.value = view
        regions.value = regionList
      }
      options.onLoaded?.()
    } catch (err) {
      errorMessage.value = extractApiErrorMessage(err, '换座池加载失败')
    } finally {
      loading.value = false
    }
  }

  return { loading, errorMessage, currentView, regions, loadPage }
}
