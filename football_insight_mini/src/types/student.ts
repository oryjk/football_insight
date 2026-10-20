export type StudentServiceCode = 'certificate_renewal' | 'tool_upgrade'

export interface StudentService {
  code: StudentServiceCode
  title: string
  description: string
  price_cents: number
  unit: '年' | '套'
  enabled: boolean
}

export interface StudentConfig {
  contact_qr_url: string
  services: StudentService[]
}

export interface StudentOrder {
  order_no: string
  code: StudentServiceCode
  title: string
  amount_cents: number
  status: 'pending' | 'paid' | 'refunded' | 'failed'
  paid_at: string | null
  created_at: string
}

export interface StudentOrderList {
  items: StudentOrder[]
  total: number
  page: number
  page_size: number
}
