export interface SeatSwapRegionLayout {
  ring: 'inner' | 'outer'
  side: 'right' | 'top' | 'left' | 'bottom'
  left: number
  top: number
  width: number
  height: number
}

export type SeatSwapRegionColorGroup =
  | 'blue'
  | 'green'
  | 'purple'
  | 'yellow'
  | 'navy'
  | 'red'
  | 'muted'

const regionColorGroups: Record<Exclude<SeatSwapRegionColorGroup, 'muted'>, number[]> = {
  blue: [511, 512, 516, 517, 529, 530, 534, 535],
  green: [102, 103, 104, 105, 106, 107, 108, 116, 117, 123, 513, 514, 515, 531, 532, 533],
  purple: [504, 505, 506, 507, 521, 522, 523, 524, 525],
  yellow: [109, 110, 111, 112, 113, 125, 126, 130, 131],
  navy: [508, 509, 510, 518, 519, 520, 526, 527, 528],
  red: [127, 129],
}

export function resolveSeatSwapRegionColorGroup(name: string): SeatSwapRegionColorGroup {
  const number = Number.parseInt(name, 10)
  for (const [group, regions] of Object.entries(regionColorGroups)) {
    if (regions.includes(number)) {
      return group as SeatSwapRegionColorGroup
    }
  }
  return 'muted'
}

export function resolveSeatSwapRegionLayout(name: string): SeatSwapRegionLayout | null {
  const number = Number.parseInt(name, 10)
  if (Number.isNaN(number)) {
    return null
  }

  if (number >= 101 && number <= 132) {
    return resolveCounterclockwiseRing(number - 101, 32, 'inner')
  }

  if (number >= 501 && number <= 536) {
    return resolveCounterclockwiseRing(number - 501, 36, 'outer')
  }

  return null
}

export function formatSeatSwapSeatLabel(input: {
  current_region_name: string
  current_row: string
  current_seat_no: string
}): string {
  const extra = [
    input.current_row.trim() ? `${input.current_row.trim()}排` : '',
    input.current_seat_no.trim() ? `${input.current_seat_no.trim()}号` : '',
  ]
    .filter(Boolean)
    .join(' ')
  return extra ? `${input.current_region_name} ${extra}` : input.current_region_name
}

export function formatSeatSwapDesiredSeats(
  seats: Array<{
    region_name: string
    desired_row?: string | null
    desired_seat_no?: string | null
  }>,
): string {
  return seats
    .map((seat) => {
      const extra = [
        seat.desired_row ? `${seat.desired_row}排` : '',
        seat.desired_seat_no ? `${seat.desired_seat_no}号` : '',
      ]
        .filter(Boolean)
        .join(' ')
      return extra ? `${seat.region_name} ${extra}` : seat.region_name
    })
    .join('、')
}

export function seatSwapStatusLabel(status: string): string {
  switch (status) {
    case 'communicable':
      return '可沟通'
    case 'waiting_peer_confirmation':
      return '等待对方确认'
    case 'peer_confirmed_me':
      return '对方已确认你'
    case 'matched':
      return '已匹配成功'
    default:
      return ''
  }
}

function resolveCounterclockwiseRing(
  index: number,
  total: number,
  ring: SeatSwapRegionLayout['ring'],
): SeatSwapRegionLayout {
  const isOuter = ring === 'outer'
  const metrics = isOuter
    ? { left: 6, right: 94, top: 8, bottom: 84, itemW: 8.6, itemH: 8.4, cornerX: 7, cornerY: 8 }
    : { left: 15, right: 85, top: 18, bottom: 73, itemW: 7.2, itemH: 8.2, cornerX: 6, cornerY: 7 }
  const sideCount = total / 4
  const sideIndex = Math.floor(index / sideCount)
  const offset = index % sideCount
  const progress = (offset + 1) / (sideCount + 1)
  const cornerProgress = Math.abs(progress - 0.5) * 2
  const cornerCurve = Math.pow(cornerProgress, 2.2)
  const side = resolveRoundedTrackSide(sideIndex)

  let centerX = metrics.right
  let centerY = metrics.bottom - progress * (metrics.bottom - metrics.top)

  if (side === 'right') {
    centerX = metrics.right - metrics.cornerX * cornerCurve
  } else if (side === 'top') {
    centerX = metrics.right - progress * (metrics.right - metrics.left)
    centerY = metrics.top + metrics.cornerY * cornerCurve
  } else if (side === 'left') {
    centerX = metrics.left + metrics.cornerX * cornerCurve
    centerY = metrics.top + progress * (metrics.bottom - metrics.top)
  } else {
    centerX = metrics.left + progress * (metrics.right - metrics.left)
    centerY = metrics.bottom - metrics.cornerY * cornerCurve
  }

  return {
    ring,
    side,
    left: centerX - metrics.itemW / 2,
    top: centerY - metrics.itemH / 2,
    width: metrics.itemW,
    height: metrics.itemH,
  }
}

function resolveRoundedTrackSide(sideIndex: number): SeatSwapRegionLayout['side'] {
  if (sideIndex === 0) return 'right'
  if (sideIndex === 1) return 'top'
  if (sideIndex === 2) return 'left'
  return 'bottom'
}
