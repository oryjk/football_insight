import { resolveSeatSwapRegionLayout } from '../../utils/stadiumRegions'

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

export interface SeatSwapMapFabLabel {
  text: string
  clearable: boolean
}

/** 浮窗文案：无筛选显示"地图"；有筛选显示分区名并允许清除。 */
export function resolveSeatSwapMapFabLabel(filterName: string): SeatSwapMapFabLabel {
  const name = filterName.trim()
  return name ? { text: name, clearable: true } : { text: '地图', clearable: false }
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
  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}">`,
    '  <g fill="#dde1e8">',
    ...rects.map((r) => `  ${r}`),
    '  </g>',
    '</svg>',
    '',
  ].join('\n')
}
