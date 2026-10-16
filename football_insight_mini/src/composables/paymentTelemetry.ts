import { reportPaymentEvents } from '../api/payment'
import { MINI_PROGRAM_VERSION } from '../api/system'
import { getAccessToken } from '../utils/authStorage'
import type { ClientPaymentEvent, PaymentAttempt, PaymentEvent, PaymentEventExtra, PaymentEventsPayload, PaymentScenario } from '../types/paymentTelemetry'
export type { PaymentAttempt } from '../types/paymentTelemetry'

export function classifyPaymentFailure(failure: { errCode?: number; errMsg?: string }) {
  return {
    event: failure.errCode === -2 || /cancel|取消/i.test(failure.errMsg || '') ? 'pay_cancelled' as const : 'pay_failed' as const,
    error_code: failure.errCode === undefined ? undefined : String(failure.errCode),
    error_msg: failure.errMsg,
  }
}

/** Optional telemetry must also tolerate platform APIs that are absent or throw. */
export function safelyReportPaymentEvent(attempt: PaymentAttempt | undefined, event: ClientPaymentEvent, extra?: PaymentEventExtra): void {
  if (!attempt) return
  try { attempt.report(event, extra) } catch { console.warn('[payment telemetry] event discarded') }
}

function accessToken(): string | null {
  try { return getAccessToken() } catch { return null }
}

function jsonBytes(value: unknown): number {
  return encodeURIComponent(JSON.stringify(value)).replace(/%[A-F\d]{2}/gi, 'x').length
}

function eventDetail(common: Record<string, string | number>, elapsed: number, httpStatus?: number): Record<string, string | number> {
  const detail: Record<string, string | number> = { elapsed_ms: elapsed }
  if (httpStatus !== undefined) detail.http_status = httpStatus
  for (const [key, value] of Object.entries(common)) {
    if (jsonBytes({ ...detail, [key]: value }) <= 1024) detail[key] = value
  }
  return detail
}

function uuidV4(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (char) => {
    const random = Math.floor(Math.random() * 16)
    return (char === 'x' ? random : (random & 3) | 8).toString(16)
  })
}

export function startPaymentAttempt(scenario: PaymentScenario, productRef: string): PaymentAttempt {
  const attemptId = uuidV4()
  const startedAt = Date.now()
  const ownerToken = accessToken()
  const common: Record<string, string | number> = { mini_version: MINI_PROGRAM_VERSION }
  try {
    const info = uni.getSystemInfoSync()
    for (const [target, value] of Object.entries({ wx_lib_version: info.SDKVersion, platform: info.platform, system: info.system, brand: info.brand, model: info.model })) {
      if (typeof value === 'string' && value) common[target] = Array.from(value).slice(0, 60).join('')
    }
  } catch { /* Platform metadata is optional. */ }
  try { uni.getNetworkType({ success: ({ networkType }) => { common.network_type = networkType } }) } catch { /* Optional. */ }
  let orderNo: string | null = null
  let queue: PaymentEvent[] = []
  let scheduled = false
  const ownsSession = () => Boolean(ownerToken && ownerToken === accessToken())
  const flush = () => {
    scheduled = false
    const events = queue
    queue = []
    if (!ownsSession()) return
    const payload = (batch: PaymentEvent[]): PaymentEventsPayload => ({ attempt_id: attemptId, order_no: orderNo, scenario, product_ref: productRef, events: batch })
    const send = (batch: PaymentEvent[]) => {
      if (!batch.length) return
      try { void reportPaymentEvents(payload(batch)).catch(() => { console.warn('[payment telemetry] report failed') }) }
      catch { console.warn('[payment telemetry] report failed') }
    }
    let batch: PaymentEvent[] = []
    for (const event of events) {
      // Stay within both server limits, including multi-byte provider messages.
      const candidate = [...batch, event]
      const bytes = jsonBytes(payload(candidate))
      if (batch.length && (candidate.length > 10 || bytes > 8192)) { send(batch); batch = [] }
      batch.push(event)
    }
    send(batch)
  }
  return {
    attemptId,
    bindOrder(value) { orderNo = value },
    report(event, extra) {
      try {
        if (!ownsSession()) return
        queue.push({ event, error_code: extra?.error_code, error_msg: extra?.error_msg === undefined ? undefined : Array.from(extra.error_msg).slice(0, 300).join(''), detail: eventDetail(common, Date.now() - startedAt, extra?.detail?.http_status) })
        if (!scheduled) {
          scheduled = true
          setTimeout(() => {
            try { flush() } catch { console.warn('[payment telemetry] report discarded') }
          }, 500)
        }
      } catch { console.warn('[payment telemetry] event discarded') }
    },
  }
}
