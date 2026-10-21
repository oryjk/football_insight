import type { RecapMatch, RecapSummary, RecapSeasonTotals, SeasonRecap } from '../../types/seasonRecap'
export const POSTER_WIDTH = 750
export const POSTER_HEIGHT = 1536
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
export type RecapPosterTheme = 'dark' | 'light'
export const POSTER_PALETTES = {
 dark: { ...POSTER_PALETTE, track: 'rgba(255,237,219,0.18)', progressStart: '#ffe9ac', progressMid: '#ff9b48', progressEnd: '#d90016' },
 light: {
  ...POSTER_PALETTE, nightTop: '#fff5e5', nightMid: '#fff9f0', nightBottom: '#fff7eb',
  ink: '#6c1619', muted: '#7e4437', faint: '#855e4d', gold: '#a8000c', goldDim: '#86561e',
  rule: 'rgba(139,81,32,0.25)', glass: 'rgba(255,253,247,0.5)', glassBorder: 'rgba(197,132,66,0.35)',
  track: '#ede0cf', progressStart: '#ffda79', progressMid: '#ff963d', progressEnd: '#e90016',
 },
} as const
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
 progress: { home: number | null; away: number | null; total: number | null }
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
// 客场权重为主场的两倍；算法只用于内部称号判定，不上屏。
export function recapBadge(
 summary: Pick<RecapSummary, 'matches' | 'home_matches' | 'away_matches'>,
 totals?: RecapSeasonTotals,
): string {
 const home = summary.home_matches, away = summary.away_matches
 if (home + away === 0) return '尚未启程'
 if (totals) {
  const all = totals.home_matches + totals.away_matches
  if (all > 0 && totals.completed_matches === all
   && home === totals.home_matches && away === totals.away_matches) return '全勤死忠'
 }
 if (home >= 10 && away === 0) return '主场守护者'
 // 兼容旧后端：不把已结束的比赛误当成整个赛季，也不颁发全勤。
 if (!totals || home > totals.home_matches || away > totals.away_matches) return '蓉城同行者'
 const possible = totals.home_matches + totals.away_matches * 2
 if (possible <= 0) return '蓉城同行者'
 const earned = home + away * 2
 if (earned * 4 >= possible * 3) return '铁血死忠'
 if (earned * 2 >= possible) return '蓉城铁杆'
 if (earned * 4 >= possible) return '主场死忠'
 return '蓉城同行者'
}
export function recapProgress(summary: RecapSummary, totals?: RecapSeasonTotals): RecapPosterContent['progress'] {
 if (!totals) return { home: null, away: null, total: null }
 const ratio = (attended: number, total: number) => total > 0 ? Math.max(0, Math.min(1, attended / total)) : 0
 return {
  home: ratio(summary.home_matches, totals.home_matches),
  away: ratio(summary.away_matches, totals.away_matches),
  total: ratio(summary.matches, totals.home_matches + totals.away_matches),
 }
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
  badge: recapBadge(summary, view.season_totals),
  progress: recapProgress(summary, view.season_totals),
  route: recapRoute(view.matches, view.selected_match_ids),
  routeEntries: recapRouteEntries(view.matches, view.selected_match_ids),
 }
}
export { savePosterWithPermission as saveRecapPoster, materializePosterCode as materializeCode } from '../../utils/canvasPoster'
