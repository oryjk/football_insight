import type { DigitalPaymentParams, VirtualPayParams } from '../types/payment'

type VirtualPaymentFailure = { errCode?: number; errMsg?: string }
type VirtualPaymentApi = {
  requestVirtualPayment?: (options: VirtualPayParams & { success: () => void; fail: (error: VirtualPaymentFailure) => void }) => void
}

export function canUseVirtualPayment(): boolean {
  let supported = false
  // #ifdef MP-WEIXIN
  supported = typeof (uni as unknown as VirtualPaymentApi).requestVirtualPayment === 'function'
    && typeof uni.canIUse === 'function' && uni.canIUse('requestVirtualPayment')
  // #endif
  return supported
}

export function requestDigitalPayment(params: DigitalPaymentParams): Promise<void> {
  if (!('mode' in params) || params.mode !== 'short_series_goods' || !params.signData || !params.paySig || !params.signature) {
    return Promise.reject(new Error('支付配置已更新，请更新小程序后重试'))
  }
  if (!canUseVirtualPayment()) return Promise.reject(new Error('请使用支持该支付功能的最新版微信小程序'))
  return new Promise((resolve, reject) => {
    // #ifdef MP-WEIXIN
    // uni-app proxies unknown WeChat APIs to the underlying platform.
    const api = uni as unknown as VirtualPaymentApi
    api.requestVirtualPayment!({
      mode: params.mode, signData: params.signData, paySig: params.paySig, signature: params.signature,
      success: () => resolve(),
      fail: (error) => reject(new Error(error.errCode === -2 || /cancel|取消/i.test(error.errMsg || '')
        ? '支付已取消' : error.errCode === -15007 ? '支付会话已过期，请重新购买' : '支付未完成，请稍后重试')),
    })
    return
    // #endif
    // #ifndef MP-WEIXIN
    reject(new Error('请在微信小程序内完成购买'))
    // #endif
  })
}
