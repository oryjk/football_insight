import { ApiRequestError } from './apiError'

export function canShowOfficialWechatBindEntry(hasUser: boolean, underReview: boolean, officialBound?: boolean): boolean {
  return hasUser && !underReview && officialBound !== true
}

export function formatOfficialWechatBindCode(code: string): string {
  const normalized = code.trim().toUpperCase()
  return /^[ABCDEFGHJKLMNPQRSTUVWXYZ23456789]{6}$/.test(normalized) ? normalized : ''
}

export function getOfficialWechatBindRemainingSeconds(expiresAt: number | null, nowMs = Date.now()): number {
  return expiresAt === null ? 0 : Math.max(0, Math.ceil((expiresAt - nowMs) / 1000))
}

export function formatOfficialWechatBindCountdown(seconds: number): string {
  const remaining = Math.max(0, Math.floor(seconds))
  return `${String(Math.floor(remaining / 60)).padStart(2, '0')}:${String(remaining % 60).padStart(2, '0')}`
}

export function resolveOfficialWechatBindError(error: unknown): string {
  return error instanceof ApiRequestError && error.statusCode === 429
    ? '获取过于频繁，请稍后再试'
    : '获取绑定码失败，请稍后重试'
}
