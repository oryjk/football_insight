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

export function previousSeatSwapStep(step: SeatSwapSelectionStep, hasPresetCandidate = false): SeatSwapSelectionStep {
  if (step === 'ready_to_publish') return hasPresetCandidate ? 'select_current' : 'select_desired'
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
