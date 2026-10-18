import { describe, expect, test } from 'bun:test'
import { buildSharePath, buildShareQuery } from './shareSource'
import { captureAcquisitionSource, parseAcquisitionLaunch, readAcquisitionSource } from './acquisitionSource'

describe('share source URLs', () => {
  for (const [path, expected] of [
    ['/pages/home/index', '/pages/home/index?src=share&cid=home'],
    ['/pages/home/index?id=42', '/pages/home/index?id=42&src=share&cid=home'],
    ['/pages/home/index?', '/pages/home/index?src=share&cid=home'],
    ['/pages/home/index?id=42&', '/pages/home/index?id=42&src=share&cid=home'],
    ['/pages/home/index#part', '/pages/home/index?src=share&cid=home#part'],
    ['/pages/home/index?id=42#part?cid=old', '/pages/home/index?id=42&src=share&cid=home#part?cid=old'],
    ['', ''],
    ['#part', '#part'],
  ]) test(`preserves the page, query and fragment: ${path}`, () => {
    expect(buildSharePath(path, 'home')).toBe(expected)
  })

  for (const query of ['src=old', 'cid=recap2026', 'src=share&cid=recap2026', 'src=', 'cid', '%73rc=old'])
  test(`keeps existing campaign parameters: ${query}`, () => {
    const path = `/pages/season-recap/index?season=2026&${query}#poster`
    expect(buildSharePath(path, 'home')).toBe(path)
  })

  for (const cid of ['', 'A', 'with space', 'a&token=secret', '中文', 'a'.repeat(65)])
  test(`invalid attribution cannot break sharing: ${cid}`, () => {
    expect(buildShareQuery(cid)).toBe('')
    expect(buildSharePath('/pages/home/index?id=42#part', cid)).toBe('/pages/home/index?id=42#part')
  })

  test('accepts the shared source tracking limits and exact parameter names', () => {
    const cid = 'a'.repeat(62) + '_-'
    expect(buildShareQuery(cid)).toBe(`src=share&cid=${cid}`)
    expect(buildSharePath('/pages/home/index?description=src%3Dold&cid_extra=old', 'home'))
      .toBe('/pages/home/index?description=src%3Dold&cid_extra=old&src=share&cid=home')
  })

  test('generated paths round trip through launch capture, including a first attributed hot launch', () => {
    let stored: unknown
    const storage = { getStorageSync: () => stored, setStorageSync: (_: string, value: unknown) => { stored = value } }
    captureAcquisitionSource({ query: {} }, storage)
    expect(readAcquisitionSource(storage)).toBe(undefined)
    const query = Object.fromEntries(buildSharePath('/pages/home/index', 'home').split('?')[1]!.split('&').map(pair => pair.split('=')))
    const launch = { query, scene: 1007 }
    expect(parseAcquisitionLaunch(launch)).toEqual({ src: 'share', cid: 'home', landing_scene: 1007 })
    captureAcquisitionSource(launch, storage)
    captureAcquisitionSource({ query: { src: 'share', cid: 'matches' } }, storage)
    expect(readAcquisitionSource(storage)).toEqual({ src: 'share', cid: 'home', landing_scene: 1007 })
  })
})
