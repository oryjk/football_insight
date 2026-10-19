import { computed, ref, type Ref } from 'vue'
import type { SeatSwapCandidate, SeatSwapCurrentResponse } from '../../types/seatSwap'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import { formatSeatSwapSeatLabel, seatSwapStatusLabel } from '../../utils/stadiumRegions'
import {
  countSeatSwapDesiredRegions,
  filterOutMySeatSwapRequest,
  filterSeatSwapRequestsByDesiredRegion,
  formatSeatSwapKickoffShort,
  groupSeatSwapRequestsByRegion,
  resolveDefaultExpandedSeatSwapRegionKeys,
  resolveSeatSwapMapFabLabel,
  resolveSeatSwapMapTap,
  shouldShowSeatSwapMapFab,
  type SeatSwapMapTapOutcome,
} from './helpers'

/** 换座页浏览层：筛选、分组、展开、浮窗状态。不调用接口、不碰 uni API。 */
export function useSeatSwapBrowse(input: {
  currentView: Ref<SeatSwapCurrentResponse | null>
  regions: Ref<TicketWatchRegion[]>
  sheetOpen: Ref<boolean>
}) {
  const { currentView, regions, sheetOpen } = input
  const browsingFilterKey = ref('')
  const expandedRegionKeys = ref<string[]>([])
  // 默认视为可见：observer 尚未回报前不显示浮窗。
  const mapInViewport = ref(true)

  const matchTitle = computed(() => {
    const m = currentView.value?.current_match
    return m ? `${m.home_team_name} VS ${m.away_team_name}` : '暂无当前比赛'
  })
  const matchSummary = computed(() => formatSeatSwapKickoffShort(currentView.value?.current_match?.kickoff_at || ''))
  const candidatesCount = computed(() => currentView.value?.candidates.length || 0)
  const displayCandidates = computed<SeatSwapCandidate[]>(() =>
    filterOutMySeatSwapRequest(currentView.value?.candidates || [], currentView.value?.my_request?.request_id),
  )
  const seatSwapRegionGroups = computed(() => groupSeatSwapRequestsByRegion(displayCandidates.value))
  const filteredCandidates = computed<SeatSwapCandidate[]>(() =>
    browsingFilterKey.value
      ? filterSeatSwapRequestsByDesiredRegion(displayCandidates.value, browsingFilterKey.value)
      : [],
  )

  function regionKey(region: TicketWatchRegion): string {
    return region.block_key || region.block_name
  }
  const browsingFilterName = computed(() => {
    const key = browsingFilterKey.value
    if (!key) return ''
    return regions.value.find((r) => regionKey(r) === key)?.block_name || key
  })
  const regionBadgeCounts = computed<Record<string, number>>(() => countSeatSwapDesiredRegions(displayCandidates.value))
  const myDesiredKeys = computed<string[]>(() =>
    currentView.value?.my_request?.desired_seats.map((s) => s.region_key) || [],
  )
  const myDesiredSummary = computed(() => {
    const names = currentView.value?.my_request?.desired_seats.map((s) => s.region_name) || []
    return names.length ? names.join(' / ') : '未选择'
  })
  const myRequestSeatLabel = computed(() =>
    currentView.value?.my_request ? formatSeatSwapSeatLabel(currentView.value.my_request) : '',
  )
  const myRequestStatusLabel = computed(() => seatSwapStatusLabel(currentView.value?.my_request?.status || ''))
  const mainMapMode = computed<'browse' | 'filter' | 'published'>(() => {
    if (browsingFilterKey.value) return 'filter'
    if (currentView.value?.my_request) return 'published'
    return 'browse'
  })
  const mapFabLabel = computed(() => resolveSeatSwapMapFabLabel(browsingFilterName.value))
  const mapFabVisible = computed(() => shouldShowSeatSwapMapFab({
    available: currentView.value?.available === true,
    mapInViewport: mapInViewport.value,
    sheetOpen: sheetOpen.value,
  }))

  function syncExpandedRegionKeys(): void {
    const available = seatSwapRegionGroups.value.map((g) => g.region_key)
    expandedRegionKeys.value = expandedRegionKeys.value.filter((k) => available.includes(k))
    if (!expandedRegionKeys.value.length && available.length) {
      expandedRegionKeys.value = resolveDefaultExpandedSeatSwapRegionKeys(available)
    }
  }

  function toggleRegionGroup(key: string): void {
    expandedRegionKeys.value = expandedRegionKeys.value.includes(key)
      ? expandedRegionKeys.value.filter((k) => k !== key)
      : [...expandedRegionKeys.value, key]
  }

  function clearFilter(): void {
    browsingFilterKey.value = ''
  }

  function applyMapTap(key: string, fromSheet: boolean): SeatSwapMapTapOutcome {
    const outcome = resolveSeatSwapMapTap({
      currentFilterKey: browsingFilterKey.value,
      tappedKey: key,
      hasBadge: Boolean(regionBadgeCounts.value[key]),
      fromSheet,
    })
    browsingFilterKey.value = outcome.filterKey
    return outcome
  }

  return {
    browsingFilterKey,
    expandedRegionKeys,
    mapInViewport,
    matchTitle,
    matchSummary,
    candidatesCount,
    displayCandidates,
    seatSwapRegionGroups,
    filteredCandidates,
    browsingFilterName,
    regionBadgeCounts,
    myDesiredKeys,
    myDesiredSummary,
    myRequestSeatLabel,
    myRequestStatusLabel,
    mainMapMode,
    mapFabLabel,
    mapFabVisible,
    syncExpandedRegionKeys,
    toggleRegionGroup,
    clearFilter,
    applyMapTap,
  }
}
