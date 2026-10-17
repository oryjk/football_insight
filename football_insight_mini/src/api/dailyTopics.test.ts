import { afterEach, describe, expect, test } from 'bun:test'

import { AUTH_TOKEN_STORAGE_KEY } from '../utils/authStorage'
import { recordTopicVisit } from './dailyTopics'

type FakeUni = {
  getStorageSync: (key: string) => unknown
  request: (options: { header: Record<string, string>, success: (response: { statusCode: number, data: unknown }) => void }) => void
}

const globals = globalThis as unknown as { uni?: FakeUni }

afterEach(() => {
  delete globals.uni
})

function captureVisitHeaders(token: string | null): Record<string, string> {
  let captured: Record<string, string> = {}
  globals.uni = {
    getStorageSync: key => (key === AUTH_TOKEN_STORAGE_KEY ? token ?? '' : ''),
    request: (options) => {
      captured = options.header
      options.success({ statusCode: 200, data: { heat: 1 } })
    },
  }
  void recordTopicVisit(77680, 'topic', 'visitor')
  return captured
}

describe('recordTopicVisit', () => {
  // The admin "话题读者" list is built from the logged-in token on this request.
  test('sends the login token so the reader is attributed to the account', () => {
    expect(captureVisitHeaders('jwt-token').Authorization).toBe('Bearer jwt-token')
  })

  test('stays anonymous when nobody is logged in', () => {
    expect(captureVisitHeaders(null).Authorization === undefined).toBe(true)
  })
})
