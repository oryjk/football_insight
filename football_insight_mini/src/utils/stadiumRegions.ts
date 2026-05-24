export interface SeatSwapRegionLayout {
  ring: 'inner' | 'outer' | 'vip'
  side: 'right' | 'top' | 'left' | 'bottom' | 'vip'
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
  | 'vip'
  | 'muted'

const regionColorGroups: Record<Exclude<SeatSwapRegionColorGroup, 'muted'>, number[]> = {
  blue: [511, 512, 516, 517, 529, 530, 534, 535],
  green: [102, 103, 104, 105, 106, 107, 108, 116, 117, 123, 513, 514, 515, 531, 532, 533],
  purple: [504, 505, 506, 507, 521, 522, 523, 524, 525],
  yellow: [109, 110, 111, 112, 113, 125, 126, 130, 131],
  navy: [118, 508, 509, 510, 518, 519, 520, 526, 527, 528],
  red: [127, 129],
  vip: [],
}

export function resolveSeatSwapRegionColorGroup(name: string): SeatSwapRegionColorGroup {
  if (name.trim().toUpperCase().startsWith('VIP')) {
    return 'vip'
  }

  const number = Number.parseInt(name, 10)
  for (const [group, regions] of Object.entries(regionColorGroups)) {
    if (regions.includes(number)) {
      return group as SeatSwapRegionColorGroup
    }
  }
  return 'muted'
}

export function resolveSeatSwapRegionLayout(name: string): SeatSwapRegionLayout | null {
  const vipLayout = resolveVipLayout(name)
  if (vipLayout) {
    return vipLayout
  }

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

interface RingMetrics {
  ring: SeatSwapRegionLayout['ring']
  leftColumnX: number
  rightColumnX: number
  topLeftX: number
  topRightX: number
  bottomLeftX: number
  bottomRightX: number
  topY: number
  bottomY: number
  sideTopY: number
  sideBottomY: number
  horizontalItemW: number
  verticalItemW: number
  itemH: number
  cornerY: number
}

interface RingSegment {
  start: number
  count: number
  side: SeatSwapRegionLayout['side']
}

const innerRingMetrics: RingMetrics = {
  ring: 'inner',
  leftColumnX: 15,
  rightColumnX: 85,
  topLeftX: 21,
  topRightX: 79,
  bottomLeftX: 21,
  bottomRightX: 79,
  topY: 16.5,
  bottomY: 73.5,
  sideTopY: 25.5,
  sideBottomY: 64.5,
  horizontalItemW: 6.8,
  verticalItemW: 6.2,
  itemH: 5,
  cornerY: 2.8,
}

const outerRingMetrics: RingMetrics = {
  ring: 'outer',
  leftColumnX: 6,
  rightColumnX: 94,
  topLeftX: 14,
  topRightX: 86,
  bottomLeftX: 14,
  bottomRightX: 86,
  topY: 8,
  bottomY: 88,
  sideTopY: 19,
  sideBottomY: 73,
  horizontalItemW: 8.2,
  verticalItemW: 6.2,
  itemH: 5,
  cornerY: 3,
}

const innerRingSegments: RingSegment[] = [
  { start: 101, count: 7, side: 'right' },
  { start: 108, count: 9, side: 'top' },
  { start: 117, count: 7, side: 'left' },
  { start: 124, count: 9, side: 'bottom' },
]

const outerRingSegments: RingSegment[] = [
  { start: 501, count: 9, side: 'right' },
  { start: 510, count: 9, side: 'top' },
  { start: 519, count: 9, side: 'left' },
  { start: 528, count: 9, side: 'bottom' },
]

const vipLayouts: Record<string, SeatSwapRegionLayout> = {
  VIP1: { ring: 'vip', side: 'vip', left: 37.2, top: 77.3, width: 8.2, height: 3.8 },
  VIP2: { ring: 'vip', side: 'vip', left: 45.9, top: 77.3, width: 8.2, height: 3.8 },
  VIP3: { ring: 'vip', side: 'vip', left: 54.6, top: 77.3, width: 8.2, height: 3.8 },
}

function resolveCounterclockwiseRing(
  index: number,
  total: number,
  ring: SeatSwapRegionLayout['ring'],
): SeatSwapRegionLayout {
  const number = ring === 'inner' ? 101 + index : 501 + index
  const metrics = ring === 'inner' ? innerRingMetrics : outerRingMetrics
  const segments = ring === 'inner' ? innerRingSegments : outerRingSegments
  const segment = segments.find((item) => number >= item.start && number < item.start + item.count)

  if (!segment) {
    throw new Error(`Unsupported seat swap ${ring} ring index ${index} of ${total}`)
  }

  const offset = number - segment.start
  const progress = segment.count === 1 ? 0 : offset / (segment.count - 1)
  const cornerWeight = resolveCornerWeight(progress)
  const itemW = segment.side === 'top' || segment.side === 'bottom'
    ? metrics.horizontalItemW
    : metrics.verticalItemW
  let centerX = metrics.rightColumnX
  let centerY = metrics.sideBottomY

  if (segment.side === 'right') {
    centerX = metrics.rightColumnX
    centerY = interpolate(metrics.sideBottomY, metrics.sideTopY, progress)
  } else if (segment.side === 'top') {
    centerX = interpolate(metrics.topRightX, metrics.topLeftX, progress)
    centerY = metrics.topY + metrics.cornerY * cornerWeight
  } else if (segment.side === 'left') {
    centerX = metrics.leftColumnX
    centerY = interpolate(metrics.sideTopY, metrics.sideBottomY, progress)
  } else {
    centerX = interpolate(metrics.bottomLeftX, metrics.bottomRightX, progress)
    centerY = metrics.bottomY - metrics.cornerY * cornerWeight
  }

  return {
    ring,
    side: segment.side,
    left: centerX - itemW / 2,
    top: centerY - metrics.itemH / 2,
    width: itemW,
    height: metrics.itemH,
  }
}

function interpolate(start: number, end: number, progress: number): number {
  return start + (end - start) * progress
}

function resolveCornerWeight(progress: number): number {
  return Math.pow(Math.abs(progress - 0.5) * 2, 1.4)
}

function resolveVipLayout(name: string): SeatSwapRegionLayout | null {
  return vipLayouts[name.trim().toUpperCase()] || null
}
