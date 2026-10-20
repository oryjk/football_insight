import { parseAcquisitionLaunch } from '../../utils/acquisitionSource'
import type { RecapMatch, RecapSummary } from '../../types/seasonRecap'
export function parseHomeDistance(value: string, required: boolean): number | null {
 const input = value.trim()
 if (!input && !required) return null
 if (!input) throw Error('请填写去主场的单边距离')
 if (!/^\d+(?:\.\d{1,2})?$/.test(input) || Number(input) > 500) throw Error('请输入 0–500 公里的数字，最多两位小数')
 return Number(input)
}
export function recapSharePath(season: number): string { return `/pages/season-recap/index?season=${season}&src=share&cid=recap${season}` }
export function resolveRecapSeason(query: Record<string, unknown>): number {
 const source = parseAcquisitionLaunch({query})
 const candidate = typeof query.season === 'string' ? query.season : source?.src === 'share' ? /^recap(\d{4})$/.exec(source.cid ?? '')?.[1] : undefined
 const year = Number(candidate)
 return Number.isInteger(year) && year >= 2026 && year <= new Date().getFullYear() ? year : 2026
}
export type RecapFilter = 'all' | 'home' | 'away'
// 距离行显示规则：客场筛选下不显示；主场筛选下始终显示（方便先填）；全部筛选下需已选主场。
export function shouldShowHomeDistance(filter: RecapFilter, hasHomeSelected: boolean): boolean {
  if (filter === 'away') return false
  if (filter === 'home') return true
  return hasHomeSelected
}
export function filterRecapMatches(matches: RecapMatch[], filter: RecapFilter): RecapMatch[] {
 if (filter === 'home') return matches.filter(match => match.is_home)
 if (filter === 'away') return matches.filter(match => !match.is_home)
 return [...matches]
}
export function recapFilterCounts(matches: RecapMatch[]): { all: number; home: number; away: number } {
 const home = matches.filter(match => match.is_home).length
 return { all: matches.length, home, away: matches.length - home }
}
export function recapSelectionStats(matches: RecapMatch[], selected: number[]): { total: number; home: number; away: number } {
 const picked = new Set(selected)
 let home = 0, away = 0
 for (const match of matches) {
  if (!picked.has(match.match_id)) continue
  if (match.is_home) home++; else away++
 }
 return { total: home + away, home, away }
}
export function isFilterFullySelected(matches: RecapMatch[], selected: number[], filter: RecapFilter): boolean {
 const ids = filterRecapMatches(matches, filter).map(match => match.match_id)
 const picked = new Set(selected)
 return ids.length > 0 && ids.every(id => picked.has(id))
}
export function toggleFilterSelection(matches: RecapMatch[], selected: number[], filter: RecapFilter): number[] {
 const ids = filterRecapMatches(matches, filter).map(match => match.match_id)
 if (isFilterFullySelected(matches, selected, filter)) {
  const clearing = new Set(ids)
  return selected.filter(id => !clearing.has(id))
 }
 return [...new Set([...selected, ...ids])]
}
export function formatRecapDate(date: string): string {
 const matched = /^\d{4}-(\d{2})-(\d{2})$/.exec(date)
 return matched ? `${matched[1]}-${matched[2]}` : date
}
// 实时预览口径与后端 calculate 一致：逐场取整后的 round_trip_km 相加，不做球面计算。
// 任一场已选比赛缺 city、或已选客场缺 round_trip_km（旧后端）时返回 null，由页面回退到已保存 summary。
export function previewRecapSummary(matches: RecapMatch[], selected: number[], distance: string): RecapSummary | null {
 const byId = new Map(matches.map(match => [match.match_id, match]))
 const ids = [...new Set(selected)].filter(id => byId.has(id))
 const cities = new Set<string>()
 let awayKm = 0, homeMatches = 0, awayMatches = 0
 for (const id of ids) {
  const match = byId.get(id)!
  if (!match.city) return null
  cities.add(match.city)
  if (match.is_home) {
   homeMatches++
  } else {
   if (typeof match.round_trip_km !== 'number') return null
   awayMatches++
   awayKm += match.round_trip_km
  }
 }
 let homeKm: number | null = null
 if (homeMatches > 0) {
  try {
   const km = parseHomeDistance(distance, true)
   if (km !== null) homeKm = Math.round(km * 2 * homeMatches)
  } catch { homeKm = null }
 }
 return {
  away_km: awayKm,
  home_km: homeKm,
  total_km: awayKm + (homeKm ?? 0),
  matches: ids.length,
  home_matches: homeMatches,
  away_matches: awayMatches,
  cities: cities.size,
 }
}
