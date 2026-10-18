import { expect, test } from 'bun:test'
import {
  SPLASH_MAX_MS,
  SPLASH_MIN_VISIBLE_MS,
  SPLASH_SHOW_AFTER_MS,
  decideSplash,
} from './launchSplash'

test('fast launches never show the splash', () => {
  expect(decideSplash({ elapsedMs: 0, settled: false, visibleForMs: null })).toBe('wait')
  expect(decideSplash({ elapsedMs: SPLASH_SHOW_AFTER_MS - 1, settled: false, visibleForMs: null })).toBe('wait')
  expect(decideSplash({ elapsedMs: 100, settled: true, visibleForMs: null })).toBe('skip')
})

test('a slow launch shows the splash once the grace period passes', () => {
  expect(decideSplash({ elapsedMs: SPLASH_SHOW_AFTER_MS, settled: false, visibleForMs: null })).toBe('show')
})

test('once shown it stays long enough to read, then leaves as soon as data settles', () => {
  expect(decideSplash({ elapsedMs: 400, settled: true, visibleForMs: SPLASH_MIN_VISIBLE_MS - 1 })).toBe('wait')
  expect(decideSplash({ elapsedMs: 900, settled: true, visibleForMs: SPLASH_MIN_VISIBLE_MS })).toBe('hide')
  expect(decideSplash({ elapsedMs: 900, settled: false, visibleForMs: SPLASH_MIN_VISIBLE_MS })).toBe('wait')
})

test('a hard cap guarantees the splash can never block the page', () => {
  expect(decideSplash({ elapsedMs: SPLASH_MAX_MS, settled: false, visibleForMs: 100 })).toBe('hide')
  expect(decideSplash({ elapsedMs: SPLASH_MAX_MS, settled: false, visibleForMs: null })).toBe('skip')
})
