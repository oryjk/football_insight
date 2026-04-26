import { request } from '../utils/request'

export interface MembershipProduct {
  price_cents: number
  title: string
  subtitle: string
  description: string
  products?: MembershipProductOption[]
}

export interface MembershipProductOption {
  target_tier: string
  price_cents: number
  original_price_cents?: number
  pay_price_cents?: number
  upgrade_fee_cents?: number
  title: string
  subtitle: string
  description: string
}

export interface CreateOrderResponse {
  order_no: string
  params: {
    timeStamp: string
    nonceStr: string
    package: string
    signType: string
    paySign: string
  }
}

export interface OrderStatusResponse {
  order_no: string
  status: string
  amount_cents: number
  paid_at: string | null
}

export function getMembershipProduct(): Promise<MembershipProduct> {
  return request<MembershipProduct>({ url: '/payment/membership-product', auth: true })
}

export function createMembershipOrder(targetTier: string): Promise<CreateOrderResponse> {
  return request<CreateOrderResponse>({
    url: '/payment/membership/order',
    method: 'POST',
    auth: true,
    data: {
      target_tier: targetTier,
    },
  })
}

export function getOrderStatus(orderNo: string): Promise<OrderStatusResponse> {
  return request<OrderStatusResponse>({
    url: `/payment/order/${orderNo}`,
    auth: true,
  })
}
