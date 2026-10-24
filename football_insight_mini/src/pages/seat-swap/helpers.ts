import {
  formatSeatSwapDesiredSeats,
  formatSeatSwapSeatLabel,
  resolveSeatSwapRegionLayout,
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
export type SeatSwapCandidateAction = 'confirm' | 'view' | 'cancel_confirmation' | 'none'

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

  const phoneError = validateSeatSwapPhoneNumber(phone)
  if (phoneError) {
    errors.phone_number = phoneError
  }

  return errors
}

/** 手机号可不填（微信号二选一）；填了就必须是 11 位、1 开头、第二位 3-9 的大陆手机号。 */
export function validateSeatSwapPhoneNumber(value: string): string | undefined {
  const phone = value.trim()
  if (!phone) return undefined
  if (!/^\d+$/.test(phone)) return '手机号只能是数字'
  if (phone.length !== 11) return `手机号应为 11 位，当前 ${phone.length} 位`
  if (!/^1[3-9]\d{9}$/.test(phone)) return '手机号格式不正确'
  return undefined
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

export interface SeatSwapMapFabLabel {
  text: string
  clearable: boolean
}

/** 浮窗文案：无筛选显示"球场图"；有筛选显示分区名并允许清除。 */
export function resolveSeatSwapMapFabLabel(filterName: string): SeatSwapMapFabLabel {
  const name = filterName.trim()
  return name ? { text: name, clearable: true } : { text: '球场图', clearable: false }
}

/** 列表逐张入场的延迟：每张错开 step 毫秒，最多错开 maxSteps 张。 */
export function resolveSeatSwapStaggerDelayMs(index: number, step = 40, maxSteps = 5): number {
  if (!Number.isFinite(index) || index <= 0) return 0
  return Math.min(Math.floor(index), maxSteps) * step
}

/** 地图划出视口、本场开放、且没有任何弹层打开时才显示浮窗。 */
export function shouldShowSeatSwapMapFab(input: {
  available: boolean
  mapInViewport: boolean
  sheetOpen: boolean
}): boolean {
  return input.available && !input.mapInViewport && !input.sheetOpen
}

/** 地图面板首行用的紧凑开球时间：MM/DD HH:mm（本地时间）；无效输入返回空串。 */
export function formatSeatSwapKickoffShort(iso: string): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return ''
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/** 换座池外层锚点：筛选态与分组态都存在，用作地图点击后的滚动目标。 */
export const SEAT_SWAP_POOL_ANCHOR_ID = 'seat-swap-pool-anchor'

/**
 * 根据地图包裹节点的位置判断是否仍有部分露在导航下方的视口内。
 * rect 未知时按可见处理（不显示浮窗），避免误弹。
 */
export function isSeatSwapMapRectInViewport(
  rect: { top: number; bottom: number } | null,
  navigationHeight: number,
  windowHeight: number,
): boolean {
  if (!rect) return true
  return rect.bottom > navigationHeight && rect.top < windowHeight
}

export interface SeatSwapMapTapOutcome {
  filterKey: string
  closeSheet: boolean
  scrollTo: string | null
}

/**
 * 首屏地图与地图弹层共用的点击规则：
 * 有意向的分区被选中 → 滚动到换座池（来自弹层时先关闭弹层）；其余情况只切换筛选。
 */
export function resolveSeatSwapMapTap(input: {
  currentFilterKey: string
  tappedKey: string
  hasBadge: boolean
  fromSheet: boolean
}): SeatSwapMapTapOutcome {
  const filterKey = resolveSeatSwapBrowseFilterKey(input.currentFilterKey, input.tappedKey)
  const jump = input.hasBadge && filterKey !== ''
  return {
    filterKey,
    closeSheet: jump && input.fromSheet,
    scrollTo: jump ? SEAT_SWAP_POOL_ANCHOR_ID : null,
  }
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

  // 对方已和别人匹配：第三方没有任何操作。
  if (input.candidateStatus === 'taken') {
    return 'none'
  }

  // 已匹配 / 对方已邀请我：列表里只给「查看」，联系方式和确认换座都放进弹层里。
  if (input.candidateStatus === 'matched' || input.candidateStatus === 'peer_confirmed_me') {
    return 'view'
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

/** 已确认想和我换座、等我确认的人数（候选状态 peer_confirmed_me）。 */
export function countSeatSwapPeerConfirmations(candidates: Array<{ status: string }>): number {
  return candidates.filter((candidate) => candidate.status === 'peer_confirmed_me').length
}

/** 角标数字超过 99 显示 99+。 */
export function formatSeatSwapBadgeCount(count: number): string {
  return count > 99 ? '99+' : String(count)
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

/** 球场全部分区：100 层 101–132、500 层 501–536、VIP1–3（与 resolveSeatSwapRegionLayout 覆盖范围一致）。 */
export const SEAT_SWAP_STADIUM_REGION_NAMES: string[] = [
  ...Array.from({ length: 32 }, (_, i) => String(101 + i)),
  ...Array.from({ length: 36 }, (_, i) => String(501 + i)),
  'VIP1',
  'VIP2',
  'VIP3',
]

export interface SeatSwapThumbHighlight {
  key: string
  kind: 'current' | 'desired'
  /** 相对球场区域的百分比定位，与 StadiumMap 同一套坐标。 */
  style: string
}

/** 换座卡缩略图的高亮块：先目标后当前（当前叠在最上层）；未知分区跳过，当前与目标重合时只算当前。 */
export function resolveSeatSwapThumbHighlights(currentKey: string, desiredKeys: string[]): SeatSwapThumbHighlight[] {
  const toStyle = (key: string): string | null => {
    const layout = resolveSeatSwapRegionLayout(key)
    return layout
      ? `left:${layout.left}%;top:${layout.top}%;width:${layout.width}%;height:${layout.height}%;`
      : null
  }
  const highlights: SeatSwapThumbHighlight[] = []
  const seen = new Set<string>([currentKey])
  for (const key of desiredKeys) {
    if (seen.has(key)) continue
    seen.add(key)
    const style = toStyle(key)
    if (style) highlights.push({ key, kind: 'desired', style })
  }
  const currentStyle = currentKey ? toStyle(currentKey) : null
  if (currentStyle) highlights.push({ key: currentKey, kind: 'current', style: currentStyle })
  return highlights
}

export interface SeatSwapPitchBox {
  left: number
  top: number
  width: number
  height: number
}

/**
 * 球场草坪位置（百分比，与分区同一坐标系）：取 100 层内圈四边分区围出的空白，再向内留 3% 边距。
 * 大图与缩略图共用，帮助判断看台朝向与远近。
 */
export function resolveSeatSwapPitchBox(): SeatSwapPitchBox {
  let left = 0
  let right = 100
  let top = 0
  let bottom = 100
  for (let n = 101; n <= 132; n += 1) {
    const r = resolveSeatSwapRegionLayout(String(n))
    if (!r) continue
    if (r.side === 'left') left = Math.max(left, r.left + r.width)
    if (r.side === 'right') right = Math.min(right, r.left)
    if (r.side === 'top') top = Math.max(top, r.top + r.height)
    if (r.side === 'bottom') bottom = Math.min(bottom, r.top)
  }
  const pad = 3
  const round = (n: number) => Math.round(n * 100) / 100
  return {
    left: round(left + pad),
    top: round(top + pad),
    width: round(right - left - pad * 2),
    height: round(bottom - top - pad * 2),
  }
}

/** 缩略图底图的画布比例与大图球场区域一致（750×560rpx）。 */
export const SEAT_SWAP_THUMB_VIEWBOX = { width: 750, height: 560 }

/**
 * 生成缩略图底图 SVG：每个分区一块浅灰圆角矩形，不含编号、价格图例与票价配色。
 * 由 scripts/generate-seat-swap-thumb.ts 写入 static/seat-swap/stadium-thumb.svg。
 */
export function buildSeatSwapThumbSvg(): string {
  const { width, height } = SEAT_SWAP_THUMB_VIEWBOX
  const round = (n: number) => Math.round(n * 100) / 100
  const rects = SEAT_SWAP_STADIUM_REGION_NAMES.flatMap((name) => {
    const layout = resolveSeatSwapRegionLayout(name)
    if (!layout) return []
    const x = round((layout.left / 100) * width)
    const y = round((layout.top / 100) * height)
    const w = round((layout.width / 100) * width)
    const h = round((layout.height / 100) * height)
    return [`  <rect x="${x}" y="${y}" width="${w}" height="${h}" rx="6" />`]
  })
  const pitch = resolveSeatSwapPitchBox()
  const pitchRect = `  <rect class="pitch" x="${round((pitch.left / 100) * width)}" y="${round((pitch.top / 100) * height)}" width="${round((pitch.width / 100) * width)}" height="${round((pitch.height / 100) * height)}" rx="10" fill="#e3f1e7" />`
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">`,
    pitchRect,
    '  <g fill="#dde1e8">',
    ...rects.map((r) => `  ${r}`),
    '  </g>',
    '</svg>',
    '',
  ].join('\n')
}

interface SeatSwapPricedRegion {
  block_key?: string
  block_name: string
  price: string
}

/** 分区票价：取分区接口里的 price（如 "180.00" → "180元"）；找不到或无效时返回空串。 */
export function formatSeatSwapRegionPrice(regions: SeatSwapPricedRegion[], key: string): string {
  const region = regions.find((item) => (item.block_key || item.block_name) === key)
  const value = Number.parseFloat(region?.price ?? '')
  if (!Number.isFinite(value)) return ''
  return `${Number(value.toFixed(2))}元`
}

export interface SeatSwapSeatPriceRow {
  kind: 'current' | 'desired'
  label: string
  price: string
}

/** 座位大图下方的文字：当前座位一行，目标座位每个分区一行，各自带票价。 */
export function buildSeatSwapSeatPriceRows(
  request: {
    current_region_key: string
    current_region_name: string
    current_row: string
    current_seat_no: string
    desired_seats: Array<{
      region_key: string
      region_name: string
      desired_row?: string | null
      desired_seat_no?: string | null
    }>
  },
  regions: SeatSwapPricedRegion[],
): SeatSwapSeatPriceRow[] {
  return [
    {
      kind: 'current',
      label: formatSeatSwapSeatLabel(request),
      price: formatSeatSwapRegionPrice(regions, request.current_region_key),
    },
    ...request.desired_seats.map((seat) => ({
      kind: 'desired' as const,
      label: formatSeatSwapDesiredSeats([seat]),
      price: formatSeatSwapRegionPrice(regions, seat.region_key),
    })),
  ]
}

export type SeatSwapPriceDeltaKind = 'cheaper' | 'pricier' | 'same' | 'unknown'

export interface SeatSwapSwapSummary {
  current: { region: string; seat: string; price: string }
  desired: Array<{
    key: string
    region: string
    seat: string
    price: string
    delta: string
    deltaKind: SeatSwapPriceDeltaKind
  }>
}

function parseSeatSwapRegionPrice(regions: SeatSwapPricedRegion[], key: string): number | null {
  const region = regions.find((item) => (item.block_key || item.block_name) === key)
  const value = Number.parseFloat(region?.price ?? '')
  return Number.isFinite(value) ? value : null
}

function formatSeatSwapRowSeat(row?: string | null, seatNo?: string | null): string {
  return [row?.trim() ? `${row.trim()}排` : '', seatNo?.trim() ? `${seatNo.trim()}号` : ''].filter(Boolean).join(' ')
}

/** 座位大图下方"现在 → 想换到"：当前座位票价，以及每个目标分区票价与差价（省/贵/同价）。 */
export function buildSeatSwapSwapSummary(
  request: Parameters<typeof buildSeatSwapSeatPriceRows>[0],
  regions: SeatSwapPricedRegion[],
): SeatSwapSwapSummary {
  const currentPrice = parseSeatSwapRegionPrice(regions, request.current_region_key)
  return {
    current: {
      region: request.current_region_name,
      seat: formatSeatSwapRowSeat(request.current_row, request.current_seat_no),
      price: formatSeatSwapRegionPrice(regions, request.current_region_key),
    },
    desired: request.desired_seats.map((seat) => {
      const price = parseSeatSwapRegionPrice(regions, seat.region_key)
      let delta = ''
      let deltaKind: SeatSwapPriceDeltaKind = 'unknown'
      if (price !== null && currentPrice !== null) {
        const diff = Number((price - currentPrice).toFixed(2))
        if (diff < 0) {
          delta = `省${-diff}元`
          deltaKind = 'cheaper'
        } else if (diff > 0) {
          delta = `贵${diff}元`
          deltaKind = 'pricier'
        } else {
          delta = '同价'
          deltaKind = 'same'
        }
      }
      return {
        key: seat.region_key,
        region: seat.region_name,
        seat: formatSeatSwapRowSeat(seat.desired_row, seat.desired_seat_no),
        price: formatSeatSwapRegionPrice(regions, seat.region_key),
        delta,
        deltaKind,
      }
    }),
  }
}

export type SeatSwapMatchLevel = 'perfect' | 'high' | 'partial' | 'low'

export interface SeatSwapMatch {
  /** 0-100，按 10 取整，共 10 档（0/20/30/40/50/60/70/80/90/100）。 */
  score: number
  level: SeatSwapMatchLevel
  /** 对方的座位在我想要的分区里（含邻近）。 */
  theyHaveMine: boolean
  /** 我的座位在对方想要的分区里（含邻近）。 */
  iHaveTheirs: boolean
  label: string
  hint: string
}

interface SeatSwapMatchRegion {
  region_key: string
  region_name: string
}

// 看台是一圈：内环 101-132、外环 501-536，同环按圈上的间隔算远近（含首尾相接）。
function seatSwapRegionRing(name: string): { base: number; size: number; index: number } | null {
  const number = Number.parseInt(name, 10)
  if (Number.isNaN(number)) return null
  if (number >= 101 && number <= 132) return { base: 101, size: 32, index: number - 101 }
  if (number >= 501 && number <= 536) return { base: 501, size: 36, index: number - 501 }
  return null
}

/** 同环两个分区之间隔了几格；不同环、VIP 等没有编号的返回 null。 */
export function seatSwapRegionDistance(nameA: string, nameB: string): number | null {
  const a = seatSwapRegionRing(nameA)
  const b = seatSwapRegionRing(nameB)
  if (!a || !b || a.base !== b.base) return null
  const gap = Math.abs(a.index - b.index)
  return Math.min(gap, a.size - gap)
}

export function areSeatSwapRegionsAdjacent(nameA: string, nameB: string): boolean {
  return seatSwapRegionDistance(nameA, nameB) === 1
}

/** 某个分区对一组目标分区的满足度：命中 1，隔 1 格 0.6，隔 2 格 0.3，其余 0。 */
function seatSwapRegionFit(region: SeatSwapMatchRegion, wanted: SeatSwapMatchRegion[]): number {
  let best = 0
  for (const item of wanted) {
    if (item.region_key === region.region_key) return 1
    const distance = seatSwapRegionDistance(item.region_name, region.region_name)
    if (distance === 1) best = Math.max(best, 0.6)
    else if (distance === 2) best = Math.max(best, 0.3)
  }
  return best
}

/**
 * 我和对方的换座匹配度：
 * - 看两边各自的满足度：对方的座位是不是我想去的、我的座位是不是对方想去的；
 * - 较好的一边占 70%，较差的一边占 30%：一边命中另一边完全不沾边是 70%，两边都命中是 100%；
 * - 邻近分区按距离打折（隔 1 格 0.6、隔 2 格 0.3），结果按 10 取整。
 */
export function computeSeatSwapMatch(
  mine: { current_region_key: string; current_region_name: string; desired_seats: SeatSwapMatchRegion[] },
  peer: { current_region_key: string; current_region_name: string; desired_seats: SeatSwapMatchRegion[] },
): SeatSwapMatch {
  const theyHave = seatSwapRegionFit(
    { region_key: peer.current_region_key, region_name: peer.current_region_name },
    mine.desired_seats,
  )
  const iHave = seatSwapRegionFit(
    { region_key: mine.current_region_key, region_name: mine.current_region_name },
    peer.desired_seats,
  )
  const strong = Math.max(theyHave, iHave)
  const weak = Math.min(theyHave, iHave)
  const score = Math.round((strong * 70 + weak * 30) / 10) * 10
  const level: SeatSwapMatchLevel =
    score >= 100 ? 'perfect' : score >= 70 ? 'high' : score >= 40 ? 'partial' : 'low'
  const label =
    level === 'perfect' ? '双向匹配' : level === 'high' ? '高度匹配' : level === 'partial' ? '部分匹配' : '匹配度低'
  let hint = '你们的座位和彼此的目标都不在同一片区域'
  if (theyHave === 1 && iHave === 1) hint = '对方的座位正是你想去的，你的座位也正是对方想去的'
  else if (theyHave === 1) hint = '对方的座位是你想去的，你的座位不完全在对方的目标里'
  else if (iHave === 1) hint = '你的座位在对方的目标里，对方的座位不完全是你想去的'
  else if (score > 0) hint = '座位在彼此目标的附近分区，可以沟通看看'
  return { score, level, theyHaveMine: theyHave > 0, iHaveTheirs: iHave > 0, label, hint }
}
