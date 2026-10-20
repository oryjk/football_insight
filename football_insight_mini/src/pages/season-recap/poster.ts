import type { RecapMatch, RecapSummary, SeasonRecap } from '../../types/seasonRecap'
export const POSTER_WIDTH = 750
export const POSTER_HEIGHT = 1334
/* decorative：海报位图的酒红、香槟金与纸感纹理，不属于结构 UI token。 */
export const POSTER_PALETTE = {
 nightTop: '#611724',
 nightMid: '#350c18',
 nightBottom: '#160a10',
 rule: 'rgba(233, 196, 151, 0.22)',
 ink: '#fff5e9',
 muted: '#d7babc',
 faint: '#aa898e',
 gold: '#edcca0',
 goldDim: '#d3ae83',
 glass: 'rgba(255, 237, 219, 0.055)',
 glassBorder: 'rgba(233, 196, 151, 0.18)',
 codeCard: '#ffffff',
}
export interface RecapRouteEntry {
 city: string
 // 同城不同球队保留全部队徽；绘制时以并列圆徽展示。
 avatarUrls: string[]
}
export interface RecapPosterRoute {
 entries: RecapRouteEntry[]
 totalCities: number
}
export interface RecapPosterContent {
 kicker: string
 totalKm: number
 awayKm: number
 homeKm: number | null
 matches: number
 cities: number
 conversion: string | null
 badge: string
 route: string | null
 routeEntries: RecapPosterRoute | null
}
const EQUATOR_KM = 40075
const MARATHON_KM = 42.195
// 趣味换算：总里程达到赤道 0.1 圈时用赤道口径，否则用马拉松口径；0 不显示。
export function recapConversion(totalKm: number): string | null {
 if (!Number.isFinite(totalKm) || totalKm <= 0) return null
 if (totalKm >= EQUATOR_KM / 10) return `相当于绕赤道 ${(totalKm / EQUATOR_KM).toFixed(2)} 圈`
 return `相当于跑了 ${Math.max(1, Math.round(totalKm / MARATHON_KM))} 个马拉松`
}
// 称号优先级：远征场次 > 全勤场次 > 主场专一 > 默认同行；文案保持体面，不出现贬义。
export function recapBadge(summary: Pick<RecapSummary, 'matches' | 'away_matches'>): string {
 if (summary.away_matches >= 8) return '铁血远征军'
 if (summary.away_matches >= 3) return '远征先锋'
 if (summary.matches >= 15) return '全勤死忠'
 if (summary.matches > 0 && summary.away_matches === 0) return '主场守护者'
 return '蓉城同行者'
}
const ROUTE_MAX_CITIES = 6
// 足迹路线：已选客场按日期排序、城市去重；旧后端缺 city 时保留隐藏路线的行为。
export function recapRouteEntries(matches: RecapMatch[], selected: number[]): RecapPosterRoute | null {
 const picked = new Set(selected)
 const cities = new Map<string, RecapRouteEntry>()
 const ordered = [...matches].sort((a, b) => a.match_date.localeCompare(b.match_date) || a.match_id - b.match_id)
 for (const match of ordered) {
  if (!picked.has(match.match_id) || match.is_home) continue
  const city = match.city?.trim()
  if (!city) return null
  let entry = cities.get(city)
  if (!entry) { entry = { city, avatarUrls: [] }; cities.set(city, entry) }
  const url = match.opponent_avatar_url?.trim()
  if (url && !entry.avatarUrls.includes(url)) entry.avatarUrls.push(url)
 }
 if (!cities.size) return null
 return { entries: [...cities.values()].slice(0, ROUTE_MAX_CITIES), totalCities: cities.size }
}
export function recapRoute(matches: RecapMatch[], selected: number[]): string | null {
 const route = recapRouteEntries(matches, selected)
 if (!route) return null
 const head = ['成都', ...route.entries.map(entry => entry.city)].join(' → ')
 return route.totalCities > ROUTE_MAX_CITIES ? `${head} 等 ${route.totalCities} 城` : head
}
/** 旧 canvas 无 image 组件的 webp 开关，只在系统内置解码时尝试；未知平台保守回退。 */
export function supportsRecapWebp(platform: string, system: string): boolean {
 if (platform === 'android') return true
 if (platform === 'ios') return Number(/(?:iOS|iPadOS)\s+(\d+)/i.exec(system)?.[1]) >= 14
 return false
}
export function buildRecapPosterContent(view: SeasonRecap): RecapPosterContent {
 const { summary } = view
 return {
  kicker: `成都蓉城 · ${view.season} 赛季`,
  totalKm: summary.total_km,
  awayKm: summary.away_km,
  homeKm: summary.home_km,
  matches: summary.matches,
  cities: summary.cities,
  conversion: recapConversion(summary.total_km),
  badge: recapBadge(summary),
  route: recapRoute(view.matches, view.selected_match_ids),
  routeEntries: recapRouteEntries(view.matches, view.selected_match_ids),
 }
}
export { savePosterWithPermission as saveRecapPoster, materializePosterCode as materializeCode } from '../../utils/canvasPoster'
