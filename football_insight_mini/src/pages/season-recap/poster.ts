import type { RecapSummary } from '../../types/seasonRecap'
export const POSTER_WIDTH = 750
export const POSTER_HEIGHT = 1080
// Decorative exported bitmap palette, independent of live UI tokens (design-system canvas exception).
export const POSTER_PALETTE = { paper: '#ffffff', ink: '#15161b', muted: '#6d7280', accent: '#dc2626', background: '#f6f7fb' }
export function posterLines(summary: RecapSummary): string[] {
 return [`${summary.away_km} 公里`,summary.away_matches===0?'本赛季尚未远征':'客场远征 · 城市间大圆直线往返',summary.home_km===null?'未填写主场距离':`${summary.home_km} 公里`,`${summary.matches} 场比赛 · ${summary.cities} 座城市`]
}
export { savePosterWithPermission as saveRecapPoster, materializePosterCode as materializeCode } from '../../utils/canvasPoster'
