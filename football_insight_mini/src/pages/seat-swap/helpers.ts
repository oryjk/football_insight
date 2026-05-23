import {
  formatSeatSwapSeatLabel,
  seatSwapStatusLabel,
} from '../../utils/stadiumRegions'

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
export type SeatSwapCandidateAction = 'confirm' | 'cancel_confirmation' | 'none'

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
    errors.current_region_key = '请选择当前分区'
  }

  if (!form.current_row.trim()) {
    errors.current_row = '请输入当前排号'
  }

  if (!form.current_seat_no.trim()) {
    errors.current_seat_no = '请输入当前座号'
  }

  if (!form.desired_seats.length || form.desired_seats.some((seat) => !seat.region_key.trim())) {
    errors.desired_seats = '请选择想换到的分区'
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

  if (input.candidateStatus === 'matched' || input.candidateStatus === 'display_only') {
    return 'none'
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
