import { describe, expect, test } from 'bun:test'
import { formatFreshness, formatRefreshInterval, resolveRefreshState, type TicketRefreshState } from './presentation'

describe('ticket refresh presentation', () => {
  const intervals: Array<[number, string]> = [
    [1, '1 秒'], [15, '15 秒'], [59, '59 秒'], [60, '1 分钟'],
    [90, '1 分 30 秒'], [120, '2 分钟'], [600, '10 分钟'],
    [1.9, '1 秒'], [0, ''], [-1, ''], [0.9, ''],
    [Number.NaN, ''], [Number.POSITIVE_INFINITY, ''], [Number.NEGATIVE_INFINITY, ''],
  ]
  for (const [seconds, expected] of intervals) {
    test(`formats a ${seconds} second interval as ${expected}`, () => {
      expect(formatRefreshInterval(seconds)).toBe(expected)
    })
  }

  const ages: Array<[number | null, string]> = [
    [null, '尚未更新'], [-1000, '刚刚'], [0, '刚刚'], [4999, '刚刚'],
    [5000, '5 秒前'], [59999, '59 秒前'], [60000, '1 分钟前'],
    [3599999, '59 分钟前'], [3600000, '1 小时前'], [7200000, '2 小时前'],
  ]
  for (const [age, expected] of ages) {
    test(`formats client retrieval age ${age} as ${expected}`, () => {
      const now = 10_000_000
      expect(formatFreshness(now, age === null ? null : now - age)).toBe(expected)
    })
  }

  test('uses no-data copy for invalid timestamps', () => {
    expect(formatFreshness(Number.NaN, 1)).toBe('尚未更新')
    expect(formatFreshness(1, Number.POSITIVE_INFINITY)).toBe('尚未更新')
  })

  const states: Array<[boolean, boolean, boolean, TicketRefreshState]> = [
    [false, false, false, 'idle'], [false, false, true, 'idle'],
    [false, true, false, 'idle'], [false, true, true, 'idle'],
    [true, true, false, 'refreshing'], [true, true, true, 'refreshing'],
    [true, false, true, 'failing'], [true, false, false, 'live'],
  ]
  for (const [monitoring, refreshing, pollFailing, expected] of states) {
    test(`prioritizes monitoring=${monitoring} refreshing=${refreshing} failing=${pollFailing} as ${expected}`, () => {
      expect(resolveRefreshState({ monitoring, refreshing, pollFailing })).toBe(expected)
    })
  }
})
