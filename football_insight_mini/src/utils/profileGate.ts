import type { CurrentUser } from '../types/auth'
import { extractApiErrorCode } from './apiError'

export { extractApiErrorCode }

export function validateNicknameInput(raw: string): string | null {
  const trimmed = raw.trim()
  const chars = Array.from(trimmed)
  if (chars.length < 2 || chars.length > 16) {
    return null
  }
  if (/[\x00-\x1f\x7f<>]/.test(trimmed)) {
    return null
  }
  if (/^1\d{10}$/.test(trimmed)) {
    return null
  }
  if (trimmed === '微信用户' || /^wx_/i.test(trimmed)) {
    return null
  }
  return trimmed
}

export function hasValidAvatarUrl(url: string | null | undefined): boolean {
  const trimmed = typeof url === 'string' ? url.trim() : ''
  if (!trimmed) {
    return false
  }
  return (
    /^https?:\/\/\S+$/i.test(trimmed)
    || /^data:image\/(?:jpeg|jpg|png|webp);base64,[A-Za-z0-9+/=]+$/i.test(trimmed)
  )
}

export function editableNicknameInitialValue(user: CurrentUser | null): string {
  if (!user) {
    return ''
  }
  const valid = validateNicknameInput(user.display_name ?? '')
  if (!valid) {
    return ''
  }
  if (user.profile_completed === false && valid === user.account_identifier.trim()) {
    return ''
  }
  return valid
}

export function isProfileComplete(user: CurrentUser | null): boolean {
  if (!user) {
    return false
  }
  if (typeof user.profile_completed === 'boolean') {
    return user.profile_completed
  }
  const validNick = validateNicknameInput(user.display_name ?? '')
  return Boolean(
    validNick
    && validNick !== user.account_identifier.trim()
    && hasValidAvatarUrl(user.avatar_url),
  )
}
