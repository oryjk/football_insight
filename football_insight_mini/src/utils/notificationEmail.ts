export function isValidNotificationEmail(email: string): boolean {
  const normalized = email.trim()
  return normalized.length > 0 && normalized.length <= 254 && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(normalized)
}

export function formatNotificationEmailLabel(email: string | null | undefined): string {
  const normalized = typeof email === 'string' ? email.trim() : ''
  return normalized || '未填写'
}
