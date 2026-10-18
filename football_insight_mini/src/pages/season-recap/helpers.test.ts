import { describe, expect, test } from 'bun:test'
import { parseHomeDistance, recapSharePath, resolveRecapSeason } from './helpers'
import { parseAcquisitionLaunch } from '../../utils/acquisitionSource'
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
