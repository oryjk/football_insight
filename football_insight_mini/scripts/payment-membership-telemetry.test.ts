import { afterEach, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import ts from 'typescript'
import { ref } from 'vue'
import { startPaymentAttempt } from '../src/composables/paymentTelemetry'
import { ApiRequestError, extractApiErrorMessage } from '../src/utils/apiError'

// Exercise the actual SFC handler with isolated UI/API dependencies; no Vue renderer or backend needed.
const source = readFileSync(new URL('../src/pages/membership-purchase/index.vue', import.meta.url), 'utf8').match(/<script setup lang="ts">([\s\S]*?)<\/script>/)![1]
const parsed = ts.createSourceFile('membership.ts', source, ts.ScriptTarget.Latest, true)
const handler = parsed.statements.find(statement => ts.isFunctionDeclaration(statement) && statement.name?.text === 'handlePurchase')!
const compiled = ts.transpileModule(handler.getText(parsed), { compilerOptions: { target: ts.ScriptTarget.ES2020 } }).outputText
const testGlobal = globalThis as unknown as { uni: typeof uni }
const originalUni = testGlobal.uni
const originalTimer = globalThis.setTimeout
afterEach(() => { testGlobal.uni = originalUni; globalThis.setTimeout = originalTimer })
for (const outcome of ['agreement', 'success', 'prepare_failed', 'order_failed'] as const) {
  test(`membership reports ${outcome} and preserves existing UI result`, async () => {
    const timers: Array<() => void> = [], payloads: any[] = [], toasts: string[] = []
    globalThis.setTimeout = ((callback: () => void) => { timers.push(callback); return 1 }) as typeof setTimeout
    testGlobal.uni = { getStorageSync: () => 'account-a', request: (options: any) => { payloads.push(options.data); options.success({ statusCode: 204 }) }, showToast: ({ title }: any) => { toasts.push(title) }, switchTab: (options: any) => { options.success() } } as unknown as typeof uni
    let prompts = 0
    let createdId: string | undefined, paidId: string | undefined, confirmedId: string | undefined
    const scope = {
      loading: ref(false), paymentAvailable: ref(true), selectedProduct: ref({ target_tier: 'V7' }), agreementAccepted: ref(outcome !== 'agreement'),
      session: { revision: ref(0), ensureSessionReady: async () => {} }, promptAgreement: () => { prompts++ }, startPaymentAttempt, ApiRequestError, extractApiErrorMessage,
      prepareVirtualPayment: async () => { if (outcome === 'prepare_failed') throw new Error('not available') },
      createMembershipOrder: async (_tier: string, attemptId?: string) => { createdId = attemptId; if (outcome === 'order_failed') throw new ApiRequestError('server declined', 503); return { order_no: 'test-order', params: {} } },
      requestDigitalPayment: async (_params: unknown, attempt: any) => { paidId = attempt?.attemptId },
      waitForPaidOrder: async (_order: string, _current: unknown, attempt: any) => { confirmedId = attempt?.attemptId; return true },
    }
    await new Function(...Object.keys(scope), `${compiled}; return handlePurchase()`)(...Object.values(scope))
    timers.forEach(callback => callback())
    expect(payloads.length).toBe(1)
    const payload = payloads[0]
    expect(payload.scenario).toBe('membership'); expect(payload.product_ref).toBe('V7')
    if (outcome === 'agreement') {
      expect(prompts).toBe(1); expect(payload.events.map((event: any) => event.event)).toEqual(['agreement_prompted'])
    } else if (outcome === 'prepare_failed') {
      expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'session_prepare_failed']); expect(toasts).toEqual(['not available'])
    } else if (outcome === 'order_failed') {
      expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'order_create_requested', 'order_create_failed'])
      expect(payload.events[2].detail.http_status).toBe(503); expect(toasts).toEqual(['server declined'])
    } else {
      expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'order_create_requested'])
      expect(payload.order_no).toBe('test-order'); expect(createdId).toBe(payload.attempt_id); expect(paidId).toBe(createdId); expect(confirmedId).toBe(createdId)
      expect(toasts).toEqual(['会员已生效'])
    }
    expect(scope.loading.value).toBe(false)
  })
}
