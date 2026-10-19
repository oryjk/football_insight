import { describe, expect, test } from 'bun:test'
import { buildLineupPageUrl, parseLineupPageQuery } from './lineupLinks'

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
    expect(parseLineupPageQuery({ teamId: '77680', autosave: '1' })).toEqual({ teamId: 77680, autosave: true })
    expect(parseLineupPageQuery({ teamId: '77680' })).toEqual({ teamId: 77680, autosave: false })
  })

  test('ignores missing or malformed values', () => {
    expect(parseLineupPageQuery(undefined)).toEqual({ teamId: 0, autosave: false })
    expect(parseLineupPageQuery({ teamId: 'abc', autosave: 'yes' })).toEqual({ teamId: 0, autosave: false })
    expect(parseLineupPageQuery({ teamId: '-3' })).toEqual({ teamId: 0, autosave: false })
  })
})
