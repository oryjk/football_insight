import { describe, expect, test } from 'bun:test'

import { ACTIVITY_PAGE_KEYS, buildActivityPageViewUrl } from './activity'

describe('buildActivityPageViewUrl', () => {
  test('targets the backend page-view snapshot endpoint', () => {
    expect(buildActivityPageViewUrl()).toBe('/activity/page-view')
  })
})

describe('ACTIVITY_PAGE_KEYS', () => {
  test('contains the core pages reported on show', () => {
    expect(ACTIVITY_PAGE_KEYS).toEqual([
      'home',
      'rankings',
      'insights',
      'ticket_watch',
      'user',
      'membership_purchase',
    ])
  })
})
