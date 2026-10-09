import { getOrderStatus } from '../../api/payment'

export async function waitForPaidOrder(orderNo: string): Promise<boolean> {
  for (let attempt = 0; attempt < 10; attempt += 1) {
    const status = await getOrderStatus(orderNo)
    if (status.status === 'paid') {
      return true
    }
    await new Promise((resolve) => setTimeout(resolve, 1200))
  }

  return false
}

export function requestTicketWatchPayment(params: {
  timeStamp: string
  nonceStr: string
  package: string
  signType: string
  paySign: string
}): Promise<void> {
  return new Promise((resolve, reject) => {
    uni.requestPayment({
      provider: 'wxpay',
      ...params,
      success: () => resolve(),
      fail: (error) => {
        const message = error.errMsg || '支付取消'
        reject(
          new Error(
            message.includes('cancel') ||
            message.includes('关闭') ||
            message.includes('fail')
              ? '支付已取消'
              : '支付失败',
          ),
        )
      },
    })
  })
}
