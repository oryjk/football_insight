import { afterEach, expect, test } from 'bun:test'
import { ref } from 'vue'
import { useTicketWatchMatchId } from './useTicketWatchMatchId'
import { useTicketWatchSubscription } from './useTicketWatchSubscription'
import { useTicketWatchState } from './useTicketWatchState'
import type { AppSession } from '../../stores/appSession'
import type { CurrentUser } from '../../types/auth'
import type { PaymentAttempt } from '../../types/paymentTelemetry'
import { ApiRequestError } from '../../utils/apiError'
const testGlobal = globalThis as unknown as { uni: typeof uni }
const originalUni = testGlobal.uni
const originalTimer = globalThis.setTimeout
afterEach(() => { testGlobal.uni = originalUni; globalThis.setTimeout = originalTimer })

for (const scenario of ['match_id_unlock', 'reflux_subscription'] as const) {
  for (const outcome of ['success', 'prepare_failed', 'order_failed'] as const) {
    test(`${scenario} reports ${outcome} without changing purchasing outcome`, async () => {
      const timers: Array<() => void> = []
      const payloads: any[] = []
      const toasts: string[] = []
      globalThis.setTimeout = ((callback: () => void) => { timers.push(callback); return 1 }) as typeof setTimeout
      testGlobal.uni = {
        getStorageSync: () => 'account-a',
        request: (options: any) => { payloads.push(options.data); options.success({ statusCode: 204 }) },
        showToast: ({ title }: any) => { toasts.push(title) },
      } as unknown as typeof uni
      const state = useTicketWatchState()
      state.currentMatch.value = { match_id: 583 } as any
      state.currentUser.value = { id: 'user-a', has_wechat_binding: true } as CurrentUser
      state.canRequestWxPayment.value = true
      state.selectedTeam.value = 'chengdurongcheng'
      state.selectedRefluxSubscriptionPlanCode.value = 'once'
      state.refluxSubscriptionEmail.value = 'test@example.com'
      const session = { revision: ref(0), hasAccessToken: ref(true) } as unknown as AppSession
      let createId: string | undefined
      let paymentAttempt: PaymentAttempt | undefined
      let confirmAttempt: PaymentAttempt | undefined
      const prepare = async () => { if (outcome === 'prepare_failed') throw new Error('not available') }
      const create = async (_input: unknown, attemptId?: string) => {
        createId = attemptId
        if (outcome === 'order_failed') throw new ApiRequestError('server declined', 503)
        return { order_no: 'test-order', params: {} as any, wx_pay_params: {} as any }
      }
      const pay = async (_params: unknown, attempt?: PaymentAttempt) => { paymentAttempt = attempt }
      const confirm = async (_order: string, _current?: () => boolean, attempt?: PaymentAttempt) => { confirmAttempt = attempt; return true }
      if (scenario === 'match_id_unlock') {
        await useTicketWatchMatchId(state, () => {}, { prepareVirtualPayment: prepare, createMatchIdOrder: create, getMatchIdEntitlement: async () => ({ unlocked: true, via: 'purchase', effective_tier: 'V1' }), requestTicketWatchPayment: pay, waitForPaidOrder: confirm }, session).payForMatchId()
      } else {
        await useTicketWatchSubscription(state, () => {}, { prepareVirtualPayment: prepare, createRefluxSubscriptionOrder: create, getRefluxSubscriptionPlans: async () => ({} as any), getRefluxSubscriptionStatus: async () => ({} as any), requestTicketWatchPayment: pay, waitForPaidOrder: confirm }, session).submitRefluxSubscriptionOrder()
      }
      timers.forEach(callback => callback())
      expect(payloads.length).toBe(1)
      const payload = payloads[0]
      expect(payload.scenario).toBe(scenario)
      expect(payload.product_ref).toBe(scenario === 'match_id_unlock' ? '583' : 'once')
      if (outcome === 'prepare_failed') {
        expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'session_prepare_failed'])
        expect(payload.events[1].error_msg).toBe('not available')
        expect(createId).toBe(undefined); expect(toasts).toEqual(['not available'])
      } else if (outcome === 'order_failed') {
        expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'order_create_requested', 'order_create_failed'])
        expect(payload.events[2].detail.http_status).toBe(503)
        expect(payload.events[2].error_msg).toBe('server declined')
        expect(toasts).toEqual(['server declined'])
      } else {
        expect(payload.events.map((event: any) => event.event)).toEqual(['purchase_tapped', 'order_create_requested'])
        expect(payload.order_no).toBe('test-order')
        expect(createId).toBe(payload.attempt_id)
        expect(paymentAttempt?.attemptId).toBe(createId)
        expect(confirmAttempt).toBe(paymentAttempt)
        expect(toasts).toEqual([scenario === 'match_id_unlock' ? '已解锁' : '订阅已开通'])
      }
    })
  }
}
