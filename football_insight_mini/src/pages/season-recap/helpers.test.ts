import { describe, expect, test } from 'bun:test'
import { filterRecapMatches, formatRecapDate, isFilterFullySelected, parseHomeDistance, previewRecapSummary, recapFilterCounts, recapSelectionStats, recapSharePath, resolveRecapSeason, toggleFilterSelection } from './helpers'
import { parseAcquisitionLaunch } from '../../utils/acquisitionSource'
import type { RecapMatch } from '../../types/seasonRecap'
describe('season recap inputs and attribution', () => {
 test('distance allows 0..500 km, rejects coercion, excessive precision and missing required value', () => {
  for (const input of ['-1', '500.01', 'abc', 'NaN', 'Infinity', '1e2', '0x10', '12.345']) expect(() => parseHomeDistance(input, false)).toThrow()
  expect(parseHomeDistance('', false)).toBe(null)
  expect(() => parseHomeDistance(' ', true)).toThrow()
  expect(parseHomeDistance('0', true)).toBe(0)
  expect(parseHomeDistance('500', true)).toBe(500)
  expect(parseHomeDistance('12.25', true)).toBe(12.25)
 })
 test('direct share and mini code scene both attribute to share and retain season', () => {
  expect(recapSharePath(2026)).toBe('/pages/season-recap/index?season=2026&src=share&cid=recap2026')
  const scene = 's%3Dshare%26c%3Drecap2026'
  expect(parseAcquisitionLaunch({ query: { scene } })).toEqual({src:'share',cid:'recap2026'})
  expect(resolveRecapSeason({scene})).toBe(2026)
  expect(resolveRecapSeason({season:'2026'})).toBe(2026)
  for (const input of [{}, {season:'NaN'}, {scene:'%'}, {season:'99999'}, {scene:'s=bad&c=recap2026'}]) expect(resolveRecapSeason(input)).toBe(2026)
 })
})

test('new recap route must not replace the existing default launch page',async()=>{
 const pages=(await import('../../pages.json')).default.pages
 expect(pages[0]?.path).toBe('pages/home/index')
 expect(pages.some(page=>page.path==='pages/season-recap/index')).toBe(true)
})

describe('recap filter and selection helpers', () => {
 const matches: RecapMatch[] = [
  { match_id: 1, match_date: '2026-03-01', opponent: '北京国安', is_home: true },
  { match_id: 2, match_date: '2026-03-08', opponent: '上海申花', is_home: false },
  { match_id: 3, match_date: '2026-03-15', opponent: '山东泰山', is_home: true },
 ]
 test('filterRecapMatches splits home/away and keeps full order for all', () => {
  expect(filterRecapMatches(matches, 'all').map(m => m.match_id)).toEqual([1, 2, 3])
  expect(filterRecapMatches(matches, 'home').map(m => m.match_id)).toEqual([1, 3])
  expect(filterRecapMatches(matches, 'away').map(m => m.match_id)).toEqual([2])
 })
 test('recapFilterCounts counts each bucket', () => {
  expect(recapFilterCounts(matches)).toEqual({ all: 3, home: 2, away: 1 })
  expect(recapFilterCounts([])).toEqual({ all: 0, home: 0, away: 0 })
 })
 test('recapSelectionStats only counts ids that exist in matches', () => {
  expect(recapSelectionStats(matches, [1, 2, 99, 2])).toEqual({ total: 2, home: 1, away: 1 })
  expect(recapSelectionStats(matches, [])).toEqual({ total: 0, home: 0, away: 0 })
 })
 test('isFilterFullySelected requires a non-empty fully picked filter', () => {
  expect(isFilterFullySelected(matches, [1, 3], 'home')).toBe(true)
  expect(isFilterFullySelected(matches, [1], 'home')).toBe(false)
  expect(isFilterFullySelected(matches, [1, 2, 3], 'all')).toBe(true)
  expect(isFilterFullySelected([], [], 'home')).toBe(false)
 })
 test('toggleFilterSelection fills then clears the current filter without touching others', () => {
  const filled = toggleFilterSelection(matches, [2], 'home')
  expect(filled).toEqual([2, 1, 3])
  expect(toggleFilterSelection(matches, filled, 'home')).toEqual([2])
  expect(toggleFilterSelection(matches, [], 'all')).toEqual([1, 2, 3])
  expect(toggleFilterSelection(matches, [1, 2, 3], 'all')).toEqual([])
 })
 test('formatRecapDate trims ISO dates to MM-DD and keeps malformed input as-is', () => {
  expect(formatRecapDate('2026-03-01')).toBe('03-01')
  expect(formatRecapDate('2026-12-31')).toBe('12-31')
  for (const input of ['2026/03/01', '03-01', '2026-3-1', '']) expect(formatRecapDate(input)).toBe(input)
 })
})

describe('previewRecapSummary live preview', () => {
 const matches: RecapMatch[] = [
  { match_id: 1, match_date: '2026-03-01', opponent: '北京国安', is_home: true, city: '成都', round_trip_km: null },
  { match_id: 2, match_date: '2026-03-08', opponent: '上海申花', is_home: false, city: '上海', round_trip_km: 3340 },
  { match_id: 3, match_date: '2026-03-15', opponent: '上海海港', is_home: false, city: '上海', round_trip_km: 3340 },
  { match_id: 4, match_date: '2026-03-22', opponent: '山东泰山', is_home: true, city: '成都', round_trip_km: null },
 ]
 test('sums per-match round trips, dedupes selection and unknown ids', () => {
  const summary = previewRecapSummary(matches, [1, 2, 2, 3, 99], '')
  expect(summary).toEqual({ away_km: 6680, home_km: null, total_km: 6680, matches: 3, home_matches: 1, away_matches: 2, cities: 2 })
 })
 test('home distance multiplies per home match and rounds the total', () => {
  const summary = previewRecapSummary(matches, [1, 4], '12.25')
  expect(summary).toEqual({ away_km: 0, home_km: 49, total_km: 49, matches: 2, home_matches: 2, away_matches: 0, cities: 1 })
 })
 test('empty or invalid distance leaves home_km null but keeps away mileage', () => {
  for (const distance of ['', ' ', '-1', '500.01', 'abc', '12.345']) {
   const summary = previewRecapSummary(matches, [1, 2], distance)
   expect(summary?.home_km).toBe(null)
   expect(summary?.away_km).toBe(3340)
   expect(summary?.total_km).toBe(3340)
  }
  expect(previewRecapSummary(matches, [2], '')?.home_km).toBe(null)
 })
 test('legacy backend without city or round_trip_km falls back to null', () => {
  const legacy: RecapMatch[] = [
   { match_id: 1, match_date: '2026-03-01', opponent: '北京国安', is_home: true },
   { match_id: 2, match_date: '2026-03-08', opponent: '上海申花', is_home: false },
  ]
  expect(previewRecapSummary(legacy, [1], '10')).toBe(null)
  expect(previewRecapSummary(legacy, [2], '')).toBe(null)
  expect(previewRecapSummary(legacy, [], '')?.matches).toBe(0)
  const partial: RecapMatch[] = [{ ...matches[1]!, round_trip_km: null }]
  expect(previewRecapSummary(partial, [2], '')).toBe(null)
 })
 test('no selection previews an empty summary', () => {
  expect(previewRecapSummary(matches, [], '')).toEqual({ away_km: 0, home_km: null, total_km: 0, matches: 0, home_matches: 0, away_matches: 0, cities: 0 })
 })
})
