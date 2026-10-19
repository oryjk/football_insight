import { describe, expect, test } from 'bun:test'
import { DEFAULT_FORMATION, FORMATION_CODES, getFormation } from './formations'

const DOT_RATIO = 0.12 // 圆点直径约占球场宽度的 12%，任意两点中心距不得小于它

describe('formations', () => {
  for (const code of FORMATION_CODES) {
    test(`${code}: 11 slots, one goalkeeper, unique indices, coords in the unit square`, () => {
      const slots = getFormation(code)
      expect(slots.length).toBe(11)
      expect(slots.filter((s) => s.position === 'GK').length).toBe(1)
      expect(new Set(slots.map((s) => s.slot)).size).toBe(11)
      expect(slots.map((s) => s.slot).sort((a, b) => a - b)).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
      for (const s of slots) {
        expect(s.x >= 0 && s.x <= 1 && s.y >= 0 && s.y <= 1).toBe(true)
        expect(s.label.length > 0).toBe(true)
      }
      const gk = slots.find((s) => s.position === 'GK')!
      expect([gk.x, gk.y]).toEqual([0.5, 0.9])
    })

    test(`${code}: no two dots overlap`, () => {
      const slots = getFormation(code)
      for (let i = 0; i < slots.length; i += 1) {
        for (let j = i + 1; j < slots.length; j += 1) {
          const d = Math.hypot(slots[i].x - slots[j].x, (slots[i].y - slots[j].y) * 0.75)
          expect(d >= DOT_RATIO).toBe(true)
        }
      }
    })

    test(`${code}: position counts match the code (defenders-midfielders-forwards)`, () => {
      const slots = getFormation(code)
      const lines = code.split('-').map(Number)
      const df = slots.filter((s) => s.position === 'DF').length
      const fw = slots.filter((s) => s.position === 'FW').length
      const mf = slots.filter((s) => s.position === 'MF').length
      expect(df).toBe(lines[0])
      expect(fw).toBe(lines[lines.length - 1])
      expect(mf).toBe(lines.slice(1, -1).reduce((a, b) => a + b, 0))
    })

    test(`${code}: slots are ordered GK, then defenders, midfielders, forwards`, () => {
      const order = { GK: 0, DF: 1, MF: 2, FW: 3 }
      const ranks = getFormation(code).map((s) => order[s.position])
      expect(ranks).toEqual([...ranks].sort((a, b) => a - b))
    })
  }

  test('unknown code falls back to the default formation', () => {
    expect(getFormation('9-9-9')).toEqual(getFormation(DEFAULT_FORMATION))
  })
})
