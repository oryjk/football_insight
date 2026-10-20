import { request } from '../utils/request'
import type { CreateOrderResponse } from './payment'
import type { StudentConfig, StudentOrderList, StudentServiceCode } from '../types/student'

export function getStudentConfig(): Promise<StudentConfig> {
  return request({ url: '/student/config', auth: true })
}

export function createStudentOrder(code: StudentServiceCode): Promise<CreateOrderResponse> {
  return request({ url: '/student/orders', method: 'POST', auth: true, data: { code } })
}

export function getStudentOrders(page = 1, pageSize = 20): Promise<StudentOrderList> {
  return request({ url: '/student/orders', auth: true, data: { page, page_size: pageSize } })
}
