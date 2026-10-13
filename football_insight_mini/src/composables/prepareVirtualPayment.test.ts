import { expect, test } from 'bun:test'
import { ref } from 'vue'
import { prepareVirtualPaymentForSession } from './prepareVirtualPayment'

test('switching accounts while obtaining a WeChat code never prepares payment for the new account', async () => {
  const session = { revision: ref(1), hasAccessToken: ref(true) }
  let resolveCode!: (code: string) => void
  const code = new Promise<string>((resolve) => { resolveCode = resolve })
  let prepared = 0
  const operation = prepareVirtualPaymentForSession(session, {
    isSupported: () => true,
    getStatus: async () => ({ available: true, message: '' }),
    getCode: () => code,
    prepareSession: async () => { prepared += 1; return { ready: true } },
  })
  await Promise.resolve()
  session.revision.value += 1
  resolveCode('old-account-code')
  const error = await operation.catch((error: Error) => error)
  expect((error as Error).message).toBe('账号已切换，请重新购买')
  expect(prepared).toBe(0)
})

test('unconfigured payment does not request a login code or create a payment session', async () => {
  let codes = 0
  let prepared = 0
  const operation = prepareVirtualPaymentForSession({ revision: ref(1), hasAccessToken: ref(true) }, {
    isSupported: () => true,
    getStatus: async () => ({ available: false, message: '购买暂未开放' }),
    getCode: async () => { codes += 1; return 'code' },
    prepareSession: async () => { prepared += 1; return { ready: true } },
  })
  const error = await operation.catch((error: Error) => error)
  expect((error as Error).message).toBe('购买暂未开放')
  expect(codes).toBe(0)
  expect(prepared).toBe(0)
})
