import type { RecapMatch, RecapSummary, SeasonRecap } from '../../types/seasonRecap'
export const POSTER_WIDTH = 750
export const POSTER_HEIGHT = 1334
/* 海报位图配色（夜场远征主题）：插画装饰色例外，不走 UI token。 */
export const POSTER_PALETTE = {
 nightTop: '#0c0f24',
 nightMid: '#1c1030',
 nightBottom: '#42090f',
 glow: 'rgba(220, 38, 38, 0.55)',
 beam: 'rgba(255, 255, 255, 0.10)',
 pitchLine: 'rgba(255, 255, 255, 0.05)',
 ink: '#f5f6fa',
 muted: 'rgba(235, 238, 248, 0.6)',
 faint: 'rgba(235, 238, 248, 0.38)',
 gold: '#e9c477',
 goldDim: 'rgba(233, 196, 119, 0.75)',
 glass: 'rgba(255, 255, 255, 0.08)',
 glassBorder: 'rgba(255, 255, 255, 0.16)',
 codeCard: '#ffffff',
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
// 足迹路线：按比赛日期排序后取已选客场城市去重（首次出现顺序）；旧后端缺 city 或无客场时不显示。
export function recapRoute(matches: RecapMatch[], selected: number[]): string | null {
 const picked = new Set(selected)
 const cities: string[] = []
 const ordered = [...matches].sort((a, b) => a.match_date.localeCompare(b.match_date))
 for (const match of ordered) {
  if (!picked.has(match.match_id) || match.is_home) continue
  if (!match.city) return null
  if (!cities.includes(match.city)) cities.push(match.city)
 }
 if (!cities.length) return null
 const head = ['成都', ...cities.slice(0, ROUTE_MAX_CITIES)].join(' → ')
 return cities.length > ROUTE_MAX_CITIES ? `${head} 等 ${cities.length} 城` : head
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
 }
}
export { savePosterWithPermission as saveRecapPoster, materializePosterCode as materializeCode } from '../../utils/canvasPoster'
