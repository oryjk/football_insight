export interface SeatSwapCurrentMatch {
  match_id: number
  home_team_name: string
  away_team_name: string
  kickoff_at: string
}

export interface SeatSwapDesiredSeat {
  region_key: string
  region_name: string
  desired_row?: string | null
  desired_seat_no?: string | null
}

export interface SeatSwapContact {
  wechat_id?: string | null
  phone_number?: string | null
}

export interface SeatSwapRequest {
  request_id: string
  user_id: string
  display_name: string
  current_region_key: string
  current_region_name: string
  current_row: string
  current_seat_no: string
  desired_seats: SeatSwapDesiredSeat[]
  contact?: SeatSwapContact | null
  status: string
  created_at: string
}

export interface SeatSwapCandidate extends SeatSwapRequest {
  status: 'communicable' | 'waiting_peer_confirmation' | 'peer_confirmed_me' | 'matched' | 'display_only'
}

export interface SeatSwapCurrentResponse {
  available: boolean
  current_match?: SeatSwapCurrentMatch | null
  my_request?: SeatSwapRequest | null
  candidates: SeatSwapCandidate[]
}

export interface UpsertSeatSwapRequestPayload {
  current_region_key: string
  current_region_name: string
  current_row: string
  current_seat_no: string
  wechat_id?: string | null
  phone_number?: string | null
  desired_seats: SeatSwapDesiredSeat[]
}

export interface CancelMatchedSeatSwapPayload {
  reason: string
  evidence_file_name: string
  evidence_content_type: string
  evidence_base64: string
}
