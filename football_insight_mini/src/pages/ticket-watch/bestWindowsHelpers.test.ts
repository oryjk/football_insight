import { expect, test } from 'bun:test'
import { hourFromChartX, buildBestWindowsBars, formatBestWindowsConclusion, formatBestWindow, resolveBestWindowsData, bestWindowsHourDetail } from './bestWindowsHelpers'
const hours = Array.from({ length: 24 }, (_, hour) => ({ hour, avg_events: hour === 12 ? 100 : hour === 11 ? 1 : 0, share: 0, hit_matches: hour === 12 ? 4 : 0 }))
const peak = { start_hour: 12, end_hour: 13, avg_events: 100, hit_matches: 4 }
const full = { status: 'ok' as const, access: 'full' as const, sample_size: 5, generated_at: '2026-10-16T00:00:00Z', top_windows: [peak], hours }
test('24 hours preserve zeros, scale max and retain small bars; highlight peaks and current hour', () => {
 const bars = buildBestWindowsBars(full, 12)
 expect(bars.length).toBe(24)
 expect(bars[0].heightPercent).toBe(0)
 expect(bars[11].heightPercent).toBe(4)
 expect(bars[12].heightPercent).toBe(100)
 expect(bars.filter(bar => bar.isNow).map(bar => bar.hour)).toEqual([12])
 expect(bars.filter(bar => bar.isPeak).map(bar => bar.hour)).toEqual([12])
 expect(bars.filter(bar => bar.label).map(bar => bar.label)).toEqual(['0', '6', '12', '18'])
})
test('all-zero hours have finite heights and top three alone highlight', () => {
 expect(buildBestWindowsBars({ ...full, hours: hours.map(row => ({ ...row, avg_events: 0 })) }, 25).every(row => row.heightPercent === 0 && !row.isNow)).toBe(true)
 const top = [12, 11, 14, 15].map(hour => ({ ...peak, start_hour: hour, end_hour: hour + 1 }))
 expect(buildBestWindowsBars({ ...full, top_windows: top }, 0).filter(row => row.isPeak).length).toBe(3)
})
test('formats exact copy with short peaks and midnight endpoint', () => {
 expect(formatBestWindow({ ...peak, start_hour: 23, end_hour: 24 })).toBe('23:00–24:00')
 expect(formatBestWindowsConclusion(full.top_windows)).toBe('补票最集中在 12:00–13:00')
 expect(bestWindowsHourDetail(12, 5, 4)).toBe('12 点：近 5 场里有 4 场出现补票')
})
test('preview never produces bars and paused benefits strip full details', () => {
 const preview = resolveBestWindowsData(full, true, false)!
 expect(preview.access).toBe('preview')
 expect(preview.hours).toEqual([])
 expect(preview.top_windows.length).toBe(1)
 expect(buildBestWindowsBars(preview, 12)).toEqual([])
})
test('review and unsuccessful states hide the entire panel', () => {
 expect(resolveBestWindowsData(full, false, true)).toBeNull()
 expect(resolveBestWindowsData({ ...full, status: 'insufficient' }, false, false)).toBeNull()
 expect(resolveBestWindowsData({ ...full, status: 'unavailable' }, false, false)).toBeNull()
 expect(resolveBestWindowsData(null, false, false)).toBeNull()
})

test('preview masks the hour even when the server sent it, and never leaks it into copy', () => {
 const preview = resolveBestWindowsData(full, true, false)!
 expect(preview.top_windows[0].start_hour).toBeNull()
 expect(formatBestWindow(preview.top_windows[0])).toBe('**:**–**:**')
 expect(/12|13/.test(formatBestWindowsConclusion(preview.top_windows))).toBe(false)
 const serverPreview = resolveBestWindowsData({ ...full, access: 'preview', top_windows: [{ ...peak, start_hour: null, end_hour: null }], hours: [] }, false, false)!
 expect(formatBestWindow(serverPreview.top_windows[0])).toBe('**:**–**:**')
})
test('touch x maps to the hour column and clamps outside the chart', () => {
 expect(hourFromChartX(10, 10, 240)).toBe(0)
 expect(hourFromChartX(10 + 125, 10, 240)).toBe(12)
 expect(hourFromChartX(10 + 239, 10, 240)).toBe(23)
 expect(hourFromChartX(-50, 10, 240)).toBe(0)
 expect(hourFromChartX(900, 10, 240)).toBe(23)
 expect(hourFromChartX(5, 0, 0)).toBeNull()
 expect(hourFromChartX(Number.NaN, 0, 240)).toBeNull()
})
