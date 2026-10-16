import { afterEach, expect, test } from 'bun:test'
import { createMembershipOrder, getMembershipProduct, getOrderStatus, getVirtualPaymentStatus } from './payment'
const testGlobal = globalThis as unknown as { uni: typeof uni }
const originalUni = testGlobal.uni
afterEach(() => { testGlobal.uni = originalUni })

test('catalog-v2 capability is confined to membership catalog and orders, keeping attempt correlation and bodies', async () => {
  const requests: any[] = []
  testGlobal.uni = { getStorageSync: () => 'test-token', request: (options: any) => { requests.push(options); options.success({ statusCode: 200, data: {} }) } } as unknown as typeof uni
  await getMembershipProduct()
  await createMembershipOrder('V4', 'attempt-id')
  await createMembershipOrder('V6')
  await getOrderStatus('order-1')
  await getVirtualPaymentStatus()
  expect(requests.map(options => options.header['X-Client-Capabilities'])).toEqual(['catalog-v2', 'catalog-v2', 'catalog-v2', undefined, undefined])
  expect(requests[1].header['X-Payment-Attempt-Id']).toBe('attempt-id')
  expect(requests[2].header['X-Payment-Attempt-Id']).toBe(undefined)
  expect(requests[1].data).toEqual({ target_tier: 'V4' })
  expect(requests[2].data).toEqual({ target_tier: 'V6' })
})
