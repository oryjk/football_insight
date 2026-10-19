import { expect, test } from 'bun:test'
import { describePitchLayout } from './layoutDiagnostics'

const expected = [{ slot: 0, x: 140, y: 400 }, { slot: 1, x: 40, y: 300 }]
const area = { left: 12, top: 200, width: 351, height: 468 }

test('distinguishes valid model coordinates from native movers collapsed at the origin', () => {
  const report = describePitchLayout(expected, area, [
    { left: 12, top: 200, width: 58, height: 62 },
    { left: 12, top: 200, width: 58, height: 62 },
  ])
  expect(report.expectedFinite).toBe(true)
  expect(report.expectedDistinct).toBe(2)
  expect(report.actualDistinct).toBe(1)
  expect(report.actual[0].x).toBe(0)
  expect(report.actual[0].y).toBe(0)
})

test('native coordinates are relative to the measured area, including scroll offset', () => {
  const report = describePitchLayout(expected, area, expected.map(({ x, y }) => ({
    left: x + area.left, top: y + area.top, width: 58, height: 62,
  })))
  expect(report.actualDistinct).toBe(2)
  expect(report.actual.map(({ x, y }) => ({ x, y }))).toEqual(expected.map(({ x, y }) => ({ x, y })))
})

test('invalid coordinates and missing native nodes remain distinguishable', () => {
  const report = describePitchLayout([{ slot: 0, x: NaN, y: 0 }], null, [])
  expect(report.expectedFinite).toBe(false)
  expect(report.area).toBeNull()
  expect(report.actual).toEqual([])
})
