import {
  formatSeatSwapSeatLabel,
  seatSwapStatusLabel,
} from '../../utils/stadiumRegions'
import type { TicketWatchRegion } from '../../types/ticketWatch'
import type {
  SeatSwapCandidate,
  SeatSwapCurrentResponse,
  SeatSwapDesiredSeat,
  SeatSwapRequest,
} from '../../types/seatSwap'

export interface SeatSwapDesiredSeatFormState {
  region_key: string
  region_name: string
  desired_row: string
  desired_seat_no: string
}

export interface SeatSwapFormState {
  current_region_key: string
  current_region_name: string
  current_row: string
  current_seat_no: string
  wechat_id: string
  phone_number: string
  desired_seats: SeatSwapDesiredSeatFormState[]
}

export interface SeatSwapFormErrors {
  current_region_key?: string
  current_row?: string
  current_seat_no?: string
  desired_seats?: string
  contact?: string
  phone_number?: string
}

export type SeatSwapSelectionStep = 'select_current' | 'select_desired' | 'ready_to_publish'
export type SeatSwapCandidateAction = 'confirm' | 'cancel_confirmation' | 'matched_cancel' | 'none'

export interface SeatSwapRegionGroup<TRequest extends SeatSwapRegionGroupItem> {
  region_key: string
  region_name: string
  requests: TRequest[]
}

export interface SeatSwapRegionGroupItem {
  request_id: string
  current_region_key: string
  current_region_name: string
  created_at: string
}

export interface SeatSwapDesiredRegionItem {
  desired_seats: Array<{
    region_key: string
  }>
}

const DEFAULT_SEAT_SWAP_MOCK_REGIONS = [
  '101', '102', '103', '104', '105', '106', '107', '108',
  '109', '110', '111', '112', '113', '114', '115', '116',
  '117', '118', '119', '120', '121', '122', '123', '124',
  '125', '126', '127', '128', '129', '130',
  '131', '132', '501', '502', '503', '504', '505', '506',
  '507', '508', '509', '510', '511', '512', '513', '514',
  '515', '516', '517', '518', '519', '520', '521', '522',
  '523', '524', '525', '526', '527', '528', '529', '530',
  '531', '532', '533', '534', '535', '536', 'VIP1', 'VIP2', 'VIP3',
]

export function buildSeatSwapMockRegions(): TicketWatchRegion[] {
  return DEFAULT_SEAT_SWAP_MOCK_REGIONS.map((regionKey, index) => ({
    block_key: regionKey,
    block_name: regionKey,
    price:
      regionKey.startsWith('VIP')
        ? '1288'
        : regionKey.startsWith('5')
          ? ['100', '120', '150', '180'][index % 4]
          : ['180', '220', '400'][index % 3],
    usable_count: (index % 5) + 1,
    estate: 1,
  }))
}

export function buildSeatSwapMockCurrentResponse(input?: {
  candidateCount?: number
  includeMyRequest?: boolean
}): SeatSwapCurrentResponse {
  const candidateCount = input?.candidateCount ?? 72
  const includeMyRequest = input?.includeMyRequest ?? true
  const regions = DEFAULT_SEAT_SWAP_MOCK_REGIONS
  const now = Date.parse('2026-05-23T18:00:00+08:00')

  const myRequest = includeMyRequest
    ? buildSeatSwapMockRequest({
        id: 'mock-my-request',
        userId: 'mock-user-me',
        name: '我的换座',
        avatarUrl: 'https://img.example.com/mock-me.png',
        currentRegion: '131',
        currentRow: '1',
        currentSeatNo: '1',
        desiredRegionKeys: ['531', '532', '533', '534', '536'],
        createdAt: new Date(now).toISOString(),
        status: 'active',
      })
    : null

  const candidates: SeatSwapCandidate[] = Array.from({ length: candidateCount }, (_, index) => {
    const currentRegion = regions[index % regions.length]
    const desiredRegionKeys = [
      regions[(index + 7) % regions.length],
      regions[(index + 13) % regions.length],
      regions[(index + 23) % regions.length],
    ].filter((regionKey, regionIndex, list) => regionKey !== currentRegion && list.indexOf(regionKey) === regionIndex)

    const status =
      index % 12 === 0
        ? 'matched'
        : index % 9 === 0
          ? 'peer_confirmed_me'
          : index % 7 === 0
            ? 'waiting_peer_confirmation'
            : 'communicable'

    return {
      ...buildSeatSwapMockRequest({
        id: `mock-request-${index + 1}`,
        userId: `mock-user-${index + 1}`,
        name: `球迷${String(index + 1).padStart(2, '0')}`,
        avatarUrl: index % 5 === 0 ? null : `https://img.example.com/mock-${(index % 9) + 1}.png`,
        currentRegion,
        currentRow: String((index % 20) + 1),
        currentSeatNo: String((index % 28) + 1),
        desiredRegionKeys,
        createdAt: new Date(now - index * 60_000).toISOString(),
        status,
        includeContact: index % 6 !== 0,
      }),
      status,
    }
  })

  return {
    available: true,
    current_match: {
      match_id: 574,
      home_team_name: '成都蓉城',
      away_team_name: '上海申花',
      kickoff_at: '2026-05-23T19:35:00+08:00',
    },
    my_request: myRequest,
    candidates,
  }
}

export function filterOutMySeatSwapRequest<TRequest extends { request_id: string }>(
  requests: TRequest[],
  myRequestId?: string | null,
): TRequest[] {
  if (!myRequestId) return requests
  return requests.filter((request) => request.request_id !== myRequestId)
}

export function validateSeatSwapForm(form: SeatSwapFormState): SeatSwapFormErrors {
  const errors: SeatSwapFormErrors = {}

  if (!form.current_region_key.trim()) {
    errors.current_region_key = '请选择当前座位分区'
  }

  if (!form.current_row.trim()) {
    errors.current_row = '请输入当前排号'
  }

  if (!form.current_seat_no.trim()) {
    errors.current_seat_no = '请输入当前座号'
  }

  if (!form.desired_seats.length || form.desired_seats.some((seat) => !seat.region_key.trim())) {
    errors.desired_seats = '请选择目标座位分区'
  }

  const phone = form.phone_number.trim()
  const wechat = form.wechat_id.trim()
  if (!phone && !wechat) {
    errors.contact = '请至少填写微信号或手机号'
  }

  if (phone && !/^1\d{10}$/.test(phone)) {
    errors.phone_number = '请输入 11 位手机号'
  }

  return errors
}

export function canConfirmCurrentSeatRegion(
  regionKey: string,
  currentRow: string,
  currentSeatNo: string,
): boolean {
  return !!regionKey.trim() && !!currentRow.trim() && !!currentSeatNo.trim()
}

export function canConfirmDesiredSeatRegions(desiredSeats: SeatSwapDesiredSeatFormState[]): boolean {
  return desiredSeats.some((seat) => seat.region_key.trim().length > 0)
}

export function canPublishSeatSwapRequest(step: SeatSwapSelectionStep): boolean {
  return step === 'ready_to_publish'
}

export function shouldShowSeatSwapStickyMap(scrollTop: number, threshold: number): boolean {
  return scrollTop >= threshold
}

export function canInteractSeatSwapMap(mode: 'browse' | 'filter' | 'published' | 'select-current' | 'select-desired' | 'review'): boolean {
  return mode !== 'review'
}

export function buildSeatSwapRegionAnchorId(regionKey: string): string {
  return `seat-swap-group-${regionKey}`
}

export function resolveSeatSwapBrowseFilterKey(currentKey: string, tappedKey: string): string {
  return currentKey === tappedKey ? '' : tappedKey
}

export function resolveDefaultExpandedSeatSwapRegionKeys(availableKeys: string[]): string[] {
  return [...availableKeys]
}

export function shouldDimSeatSwapRegion(input: {
  mode: 'browse' | 'filter' | 'published' | 'select-current' | 'select-desired' | 'review'
  hasFilterKey: boolean
  isFilterMatched: boolean
  hasStagedCurrentKey: boolean
  isStagedCurrent: boolean
  hasStagedDesiredKeys: boolean
  isStagedDesired: boolean
  isCurrent: boolean
  isDesired: boolean
}): boolean {
  if (input.mode === 'published') {
    return !input.isCurrent && !input.isDesired
  }

  if (input.mode === 'select-current') {
    return input.hasStagedCurrentKey && !input.isStagedCurrent
  }

  if (input.mode === 'select-desired') {
    return !input.isStagedCurrent && !input.isStagedDesired && input.hasStagedDesiredKeys
  }

  return false
}

export function readSeatSwapViewportScrollTop(value: unknown): number | null {
  if (!value || typeof value !== 'object') {
    return null
  }

  const scrollTop = (value as { scrollTop?: unknown }).scrollTop
  return typeof scrollTop === 'number' ? scrollTop : null
}

export function resolveSeatSwapStickyMapThreshold(input: {
  mapTop: number | null
  viewportScrollTop: number | null
  topOffsetPx: number
  fallbackThreshold: number
}): number {
  if (input.mapTop === null || input.viewportScrollTop === null) {
    return input.fallbackThreshold
  }

  return Math.max(
    0,
    Math.round(input.mapTop + input.viewportScrollTop - input.topOffsetPx),
  )
}

export function resolveSeatSwapCandidateAction(input: {
  candidateStatus: string
  candidateRequestId: string
  myRequestId?: string | null
  isLoggedIn: boolean
}): SeatSwapCandidateAction {
  if (!input.isLoggedIn || input.candidateRequestId === input.myRequestId) {
    return 'none'
  }

  if (input.candidateStatus === 'waiting_peer_confirmation') {
    return 'cancel_confirmation'
  }

  if (input.candidateStatus === 'matched') {
    return 'matched_cancel'
  }

  return 'confirm'
}

export function previousSeatSwapStep(step: SeatSwapSelectionStep): SeatSwapSelectionStep {
  if (step === 'ready_to_publish') return 'select_desired'
  if (step === 'select_desired') return 'select_current'
  return 'select_current'
}

export function toggleDesiredSeatRegion(
  selectedSeats: SeatSwapDesiredSeatFormState[],
  region: Pick<SeatSwapDesiredSeatFormState, 'region_key' | 'region_name'>,
): SeatSwapDesiredSeatFormState[] {
  const exists = selectedSeats.some((seat) => seat.region_key === region.region_key)
  if (exists) {
    return selectedSeats.filter((seat) => seat.region_key !== region.region_key)
  }

  return [
    ...selectedSeats,
    {
      region_key: region.region_key,
      region_name: region.region_name,
      desired_row: '',
      desired_seat_no: '',
    },
  ]
}

export function groupSeatSwapRequestsByRegion<TRequest extends SeatSwapRegionGroupItem>(
  requests: TRequest[],
): SeatSwapRegionGroup<TRequest>[] {
  const groups = new Map<string, SeatSwapRegionGroup<TRequest>>()

  for (const request of requests) {
    const key = request.current_region_key
    const existing = groups.get(key)
    if (existing) {
      existing.requests.push(request)
      continue
    }

    groups.set(key, {
      region_key: request.current_region_key,
      region_name: request.current_region_name,
      requests: [request],
    })
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      requests: [...group.requests].sort((a, b) => b.created_at.localeCompare(a.created_at)),
    }))
    .sort((a, b) => a.region_name.localeCompare(b.region_name, 'zh-Hans-CN', { numeric: true }))
}

export function countSeatSwapDesiredRegions<TRequest extends SeatSwapDesiredRegionItem>(
  requests: TRequest[],
): Record<string, number> {
  const counts: Record<string, number> = {}
  for (const request of requests) {
    for (const seat of request.desired_seats) {
      if (!seat.region_key) continue
      counts[seat.region_key] = (counts[seat.region_key] || 0) + 1
    }
  }
  return counts
}

export function filterSeatSwapRequestsByDesiredRegion<TRequest extends SeatSwapDesiredRegionItem>(
  requests: TRequest[],
  regionKey: string,
): TRequest[] {
  if (!regionKey) return []
  return requests.filter((request) => request.desired_seats.some((seat) => seat.region_key === regionKey))
}

export function hasSeatSwapFormErrors(errors: SeatSwapFormErrors): boolean {
  return Object.keys(errors).length > 0
}

export function formatSeatLabel(input: {
  current_region_name: string
  current_row: string
  current_seat_no: string
}): string {
  return formatSeatSwapSeatLabel(input)
}

export function statusLabel(status: string): string {
  return seatSwapStatusLabel(status)
}

function buildSeatSwapMockRequest(input: {
  id: string
  userId: string
  name: string
  avatarUrl?: string | null
  currentRegion: string
  currentRow: string
  currentSeatNo: string
  desiredRegionKeys: string[]
  createdAt: string
  status: SeatSwapRequest['status'] | SeatSwapCandidate['status']
  includeContact?: boolean
}): SeatSwapRequest {
  const desiredSeats: SeatSwapDesiredSeat[] = input.desiredRegionKeys.map((regionKey, index) => ({
    region_key: regionKey,
    region_name: regionKey,
    desired_row: index === 0 ? '1' : null,
    desired_seat_no: index === 0 ? String(index + 1) : null,
  }))

  return {
    request_id: input.id,
    user_id: input.userId,
    display_name: input.name,
    avatar_url: input.avatarUrl ?? null,
    current_region_key: input.currentRegion,
    current_region_name: input.currentRegion,
    current_row: input.currentRow,
    current_seat_no: input.currentSeatNo,
    desired_seats: desiredSeats,
    contact: input.includeContact === false
      ? null
      : {
          wechat_id: `wx_${input.id.slice(-4)}`,
          phone_number: Number.parseInt(input.currentSeatNo, 10) % 3 === 0 ? `1380000${input.currentSeatNo.padStart(4, '0')}` : null,
        },
    status: input.status,
    created_at: input.createdAt,
    seat_swap_notice_enabled: true,
  }
}
