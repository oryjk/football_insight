import { afterEach, expect, test } from 'bun:test'
import { createMembershipOrder } from './payment'
import { createMatchIdOrder } from './matchIdUnlock'
import { createRefluxSubscriptionOrder } from './ticketWatch'
const testGlobal = globalThis as unknown as { uni: typeof uni }
const previousUni = testGlobal.uni
afterEach(() => { testGlobal.uni = previousUni })
test('all three order APIs send optional attempt header without changing bodies or old callers', async () => {
  const requests: any[] = []
  testGlobal.uni = { getStorageSync: () => 'test-token', request: (options: any) => { requests.push(options); options.success({ statusCode: 200, data: {} }) } } as unknown as typeof uni
  const input = { plan_code: 'once', team_code: 'cdu', match_id: 583, email: 'test@example.com' }
  await createMembershipOrder('V7', 'attempt-id'); await createMembershipOrder('V7')
  await createMatchIdOrder(583, 'attempt-id'); await createMatchIdOrder(583)
  await createRefluxSubscriptionOrder(input, 'attempt-id'); await createRefluxSubscriptionOrder(input)
  for (const index of [0, 2, 4]) expect(requests[index].header['X-Payment-Attempt-Id']).toBe('attempt-id')
  for (const index of [1, 3, 5]) expect(requests[index].header['X-Payment-Attempt-Id']).toBe(undefined)
  expect(requests.map(request => request.data)).toEqual([{ target_tier: 'V7' }, { target_tier: 'V7' }, { match_id: 583 }, { match_id: 583 }, input, input])
})
