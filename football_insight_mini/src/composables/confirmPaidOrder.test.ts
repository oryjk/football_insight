import { afterEach, expect, test } from 'bun:test'
import { waitForPaidOrder } from './confirmPaidOrder'
const testGlobal = globalThis as unknown as { uni: typeof uni }
const originalUni = testGlobal.uni
const originalSetTimeout = globalThis.setTimeout
afterEach(() => { testGlobal.uni = originalUni; globalThis.setTimeout = originalSetTimeout })
test('only exhausted confirmation polls report timeout, keeping optional caller behavior', async () => {
  let calls = 0
  testGlobal.uni = { getStorageSync: () => 'token', request: (options: any) => { calls++; options.success({ statusCode: 200, data: { status: 'pending' } }) } } as unknown as typeof uni
  globalThis.setTimeout = ((callback: () => void) => { callback(); return 1 }) as typeof setTimeout
  const events: string[] = []
  const attempt = { attemptId: 'id', report: (event: string) => { events.push(event) }, bindOrder: () => {} }
  expect(await waitForPaidOrder('order', () => true, attempt)).toBe(false)
  expect(calls).toBe(10); expect(events).toEqual(['confirm_timeout'])
  expect(await waitForPaidOrder('order', () => false, attempt)).toBe(false)
  expect(events).toEqual(['confirm_timeout'])
  expect(await waitForPaidOrder('order')).toBe(false)
  expect(calls).toBe(20); expect(events).toEqual(['confirm_timeout'])
})
