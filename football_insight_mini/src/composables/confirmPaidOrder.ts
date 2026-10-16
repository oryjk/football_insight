import type { PaymentAttempt } from '../types/paymentTelemetry'
import { safelyReportPaymentEvent } from './paymentTelemetry'
import { getOrderStatus } from '../api/payment'

export async function waitForPaidOrder(orderNo: string, isCurrent: () => boolean = () => true, paymentAttempt?: PaymentAttempt): Promise<boolean> {
  for (let attempt = 0; attempt < 10; attempt += 1) {
    if (!isCurrent()) return false
    const status = await getOrderStatus(orderNo)
    if (!isCurrent()) return false
    if (status.status === 'paid') return true
    if (status.status === 'refunded' || status.status === 'failed') throw new Error('订单未完成，请重新购买')
    await new Promise((resolve) => setTimeout(resolve, 1200))
  }
  safelyReportPaymentEvent(paymentAttempt, 'confirm_timeout')
  return false
}
