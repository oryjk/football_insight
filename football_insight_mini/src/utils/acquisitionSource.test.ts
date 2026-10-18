import { describe, expect, test } from 'bun:test'
import { captureAcquisitionSource, parseAcquisitionLaunch, readAcquisitionSource } from './acquisitionSource'

describe('acquisition launch source', () => {
  test('accepts direct labels, keeps numeric landing scene and enforces exact bounds', () => {
    expect(parseAcquisitionLaunch({ query: { src: 'douyin', cid: 'test1' }, scene: 1047 }))
      .toEqual({ src: 'douyin', cid: 'test1', landing_scene: 1047 })
    expect(parseAcquisitionLaunch({ query: { src: 'a'.repeat(32), cid: 'b'.repeat(64) } }))
      .toEqual({ src: 'a'.repeat(32), cid: 'b'.repeat(64) })
    expect(parseAcquisitionLaunch({ query: { src: 'douyin', cid: 'x'.repeat(65) } }))
      .toEqual({ src: 'douyin' })
  })
  test('discards malformed or absent channels without throwing', () => {
    for (const src of ['', 'Douyin', 'x/y', ' a', '抖音', '<script>', 'a'.repeat(33), 1, [], {}]) {
      expect(parseAcquisitionLaunch({ query: { src, cid: 'test' } })).toBe(undefined)
    }
    for (const value of [undefined, null, [], 1, {}, { scene: 1047 }, { query: { cid: 'only' } }]) {
      expect(parseAcquisitionLaunch(value)).toBe(undefined)
    }
  })
  test('decodes short code scene and never mixes direct source with another content', () => {
    for (const scene of ['s=douyin&c=0418a', 's%3Ddouyin%26c%3D0418a']) {
      expect(parseAcquisitionLaunch({ query: { scene }, scene: 1047 }))
        .toEqual({ src: 'douyin', cid: '0418a', landing_scene: 1047 })
    }
    expect(parseAcquisitionLaunch({ query: { src: 'share', scene: 's=douyin&c=0418a' } }))
      .toEqual({ src: 'share' })
    for (const scene of ['%', 's=douyin&s=xhs', 's=douyin&c=one&c=two', `s=douyin&c=${'x'.repeat(30)}`]) {
      expect(parseAcquisitionLaunch({ query: { scene } })).toBe(undefined)
    }
  })
  test('persists first valid source across later entries and ignores corrupt storage', () => {
    const items = new Map<string, unknown>()
    const storage = { getStorageSync: (key: string) => items.get(key), setStorageSync: (key: string, value: unknown) => { items.set(key, value) } }
    captureAcquisitionSource({ query: {} }, storage)
    expect(items.size).toBe(0)
    captureAcquisitionSource({ query: { src: 'douyin', cid: 'first' } }, storage)
    captureAcquisitionSource({ query: { src: 'xhs', cid: 'second' } }, storage)
    expect(readAcquisitionSource(storage)).toEqual({ src: 'douyin', cid: 'first' })
    const key = [...items.keys()][0]!
    items.set(key, { src: '<script>', cid: 'bad' })
    captureAcquisitionSource({ query: { src: 'share' } }, storage)
    expect(readAcquisitionSource(storage)).toEqual({ src: 'share' })
  })
  test('unavailable storage and launch APIs cannot prevent authentication', () => {
    const storage = { getStorageSync: () => { throw Error('denied') }, setStorageSync: () => { throw Error('denied') } }
    captureAcquisitionSource({ query: { src: 'douyin' } }, storage)
    expect(readAcquisitionSource(storage)).toBe(undefined)
  })
})
