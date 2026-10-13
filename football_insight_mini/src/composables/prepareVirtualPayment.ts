import { getVirtualPaymentStatus, prepareVirtualPaymentSession } from '../api/payment'
import { useAppSession, type AppSession } from '../stores/appSession'
import { canUseVirtualPayment } from '../utils/virtualPayment'

const preparationOperations = {
  getStatus: getVirtualPaymentStatus,
  prepareSession: prepareVirtualPaymentSession,
  isSupported: canUseVirtualPayment,
  getCode: () => new Promise<string>((resolve, reject) => {
    uni.login({ provider: 'weixin', success: (result) => result.code ? resolve(result.code) : reject(new Error('支付会话准备失败，请重试')),
      fail: () => reject(new Error('支付会话准备失败，请重试')) })
  }),
}
export function prepareVirtualPayment(): Promise<void> {
  return prepareVirtualPaymentForSession(useAppSession())
}
export async function prepareVirtualPaymentForSession(
  session: Pick<AppSession, 'revision' | 'hasAccessToken'>,
  operations = preparationOperations,
): Promise<void> {
  const revision = session.revision.value
  const assertCurrent = () => {
    if (session.revision.value !== revision || !session.hasAccessToken.value) throw new Error('账号已切换，请重新购买')
  }
  assertCurrent()
  if (!operations.isSupported()) throw new Error('请使用支持该支付功能的最新版微信小程序')
  const status = await operations.getStatus()
  assertCurrent()
  if (!status.available) throw new Error(status.message || '购买服务暂未开放')
  const code = await operations.getCode()
  assertCurrent()
  await operations.prepareSession(code)
  assertCurrent()
}
