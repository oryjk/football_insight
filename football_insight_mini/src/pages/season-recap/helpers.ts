import { parseAcquisitionLaunch } from '../../utils/acquisitionSource'
import type { RecapMatch } from '../../types/seasonRecap'
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
