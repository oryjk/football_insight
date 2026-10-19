import { describe, expect, test } from 'bun:test'
import { buildLineupPageUrl, parseLineupPageQuery, buildLineupViewUrl, parseLineupShareCode, buildLineupListUrl, parseLineupListQuery } from './lineupLinks'

describe('buildLineupPageUrl', () => {
  test('adds teamId only when there is one', () => {
    expect(buildLineupPageUrl(77680)).toBe('/pages/lineup/index?teamId=77680')
    expect(buildLineupPageUrl()).toBe('/pages/lineup/index')
    expect(buildLineupPageUrl(0)).toBe('/pages/lineup/index')
  })

  test('the post-login link carries a one-shot autosave marker', () => {
    expect(buildLineupPageUrl(77680, { autosave: true })).toBe('/pages/lineup/index?teamId=77680&autosave=1')
    expect(buildLineupPageUrl(undefined, { autosave: true })).toBe('/pages/lineup/index?autosave=1')
    expect(buildLineupPageUrl(77680, { autosave: false })).toBe('/pages/lineup/index?teamId=77680')
  })
})

describe('parseLineupPageQuery', () => {
  test('reads teamId and autosave from page options', () => {
    expect(parseLineupPageQuery({ teamId: '77680', autosave: '1' })).toEqual({ teamId: 77680, autosave: true, resumeAction: 'draft' })
    expect(parseLineupPageQuery({ teamId: '77680' })).toEqual({ teamId: 77680, autosave: false, resumeAction: null })
  })

  test('explicit post-login actions remain distinct and old autosave cannot publish', () => {
    expect(buildLineupPageUrl(1, { resumeAction: 'draft' })).toBe('/pages/lineup/index?teamId=1&resumeAction=draft')
    expect(buildLineupPageUrl(1, { resumeAction: 'publish' })).toBe('/pages/lineup/index?teamId=1&resumeAction=publish')
    expect(parseLineupPageQuery({ teamId: '1', resumeAction: 'publish' }).resumeAction).toBe('publish')
    expect(parseLineupPageQuery({ autosave: '1' }).resumeAction).toBe('draft')
    expect(parseLineupPageQuery({ resumeAction: 'bad' }).resumeAction).toBe(null)
  })

  test('ignores missing or malformed values', () => {
    expect(parseLineupPageQuery(undefined)).toEqual({ teamId: 0, autosave: false, resumeAction: null })
    expect(parseLineupPageQuery({ teamId: 'abc', autosave: 'yes' })).toEqual({ teamId: 0, autosave: false, resumeAction: null })
    expect(parseLineupPageQuery({ teamId: '-3' })).toEqual({ teamId: 0, autosave: false, resumeAction: null })
  })
})


describe('public lineup navigation', () => {
  test('shares only a validated readonly target, with attribution', () => {
    expect(buildLineupViewUrl('Ab12Cd34', true)).toBe('/pages/lineup/view?code=Ab12Cd34&src=share&cid=lineup')
    expect(buildLineupViewUrl('Ab12Cd34')).toBe('/pages/lineup/view?code=Ab12Cd34')
    expect(buildLineupViewUrl('../mine')).toBe('')
    expect(buildLineupListUrl(21)).toBe('/pages/lineup/list?teamId=21')
    expect(buildLineupListUrl(21, 'likes')).toBe('/pages/lineup/list?teamId=21&sort=likes')
    expect(parseLineupListQuery({ teamId: '21', sort: 'likes' })).toEqual({ teamId: 21, sort: 'likes' })
    expect(parseLineupListQuery({ teamId: '21', sort: 'bad' })).toEqual({ teamId: 21, sort: 'latest' })
  })
  test('reads encoded wxacode l parameter, never accepts another campaign or bad code', () => {
    expect(parseLineupShareCode({ scene: encodeURIComponent('s=share&c=lineup&l=Ab12Cd34') })).toBe('Ab12Cd34')
    expect(parseLineupShareCode({ code: 'Ab12Cd34' })).toBe('Ab12Cd34')
    for (const scene of ['%', 's=share&c=ticket&l=Ab12Cd34', 's=share&c=lineup&l=bad', 'l=Ab12Cd34']) expect(parseLineupShareCode({ scene })).toBe('')
  })
})
