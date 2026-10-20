import type { StudentOrder } from '../../types/student'

export function formatStudentPrice(cents: number): string {
  return (cents / 100).toFixed(cents % 100 === 0 ? 0 : 2)
}

export function studentOrderStatusLabel(status: StudentOrder['status']): string {
  return { pending: '待确认付款', paid: '已付款', refunded: '已退款', failed: '未完成' }[status] || '状态待确认'
}

export function formatStudentOrderDate(value: string): string {
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return ''
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`
}
