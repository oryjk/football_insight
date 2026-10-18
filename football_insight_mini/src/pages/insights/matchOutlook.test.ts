import { expect, test } from 'bun:test'
import { percentages, outlookCopy, recordSummary } from './matchOutlook'

test('integer percentages always total100 without highlighting a favored outcome', () => {
  for (const p of [
    [1 / 3, 1 / 3, 1 / 3],
    [0.001, 0.499, 0.5],
    [0.336, 0.336, 0.328],
    [0, 0, 1],
    [0.4, 0.3, 0.3],
  ]) {
    const values = percentages(p)
    expect(values.reduce((a, b) => a + b, 0)).toBe(100)
    expect(values.every((v) => v >= 0 && Number.isInteger(v))).toBe(true)
  }
  expect(percentages([1 / 3, 1 / 3, 1 / 3])).toEqual([34, 33, 33])
  expect(() => percentages([-1, 1, 1])).toThrow()
})
test('public record states absence honestly and shows negative improvement', () => {
  expect(
    recordSummary({
      season: 2026,
      matches: 0,
      hits: 0,
      accuracy: null,
      rps_model: null,
      rps_baseline: null,
      rps_improvement: null,
    }).includes('尚无'),
  ).toBe(true)
  expect(
    recordSummary({
      season: 2026,
      matches: 10,
      hits: 3,
      accuracy: 0.3,
      rps_model: 0.3,
      rps_baseline: 0.2,
      rps_improvement: -0.1,
    }).includes('变差'),
  ).toBe(true)
})
test('rendered outlook copy and templates contain no prohibited promotional terms', async () => {
  const source = await (
    globalThis as unknown as {
      Bun: { file(url: URL): { text(): Promise<string> } }
    }
  ).Bun.file(
    new URL('./components/InsightsMatchOutlook.vue', import.meta.url),
  ).text()
  const texts = [...Object.values(outlookCopy), source.split('<script')[0]!]
  for (const text of texts)
    expect(/赔率|盘口|投注|下注|推荐|稳|必中|保证/.test(text)).toBe(false)
})
