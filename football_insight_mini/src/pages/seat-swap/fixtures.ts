/**
 * 换座页开发用 mock 数据，仅在 DEV 且 VITE_SEAT_SWAP_MOCK_LARGE_DATA=1 时使用。
 * 从 helpers.ts 原样迁出，避免与业务纯函数混放。
 */
import type { TicketWatchRegion } from '../../types/ticketWatch'
import type {
  SeatSwapCandidate,
  SeatSwapCurrentResponse,
  SeatSwapDesiredSeat,
  SeatSwapRequest,
} from '../../types/seatSwap'

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
