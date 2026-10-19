import { describe, expect, test } from 'bun:test'
import { buildLineupPageUrl } from './lineupLinks'

describe('buildLineupPageUrl', () => {
  test('adds teamId only when there is one', () => {
    expect(buildLineupPageUrl(77680)).toBe('/pages/lineup/index?teamId=77680')
    expect(buildLineupPageUrl()).toBe('/pages/lineup/index')
    expect(buildLineupPageUrl(0)).toBe('/pages/lineup/index')
  })
})
