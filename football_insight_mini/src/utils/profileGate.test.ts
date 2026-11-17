import { describe, expect, test } from 'bun:test'
import type { CurrentUser } from '../types/auth'
import {
  editableNicknameInitialValue,
  extractApiErrorCode,
  isProfileComplete,
  validateNicknameInput,
} from './profileGate'

function makeUser(overrides: Partial<CurrentUser> = {}): CurrentUser {
  return {
    id: 'u-1',
    display_name: '13800138000',
    account_identifier: '13800138000',
    avatar_url: null,
    has_wechat_binding: false,
    membership_tier: 'V3',
    created_at: '2026-10-01T00:00:00Z',
    ...overrides,
  }
}

describe('profileGate', () => {
  test('validates custom nicknames and blocks phones/defaults/control chars', () => {
    expect(validateNicknameInput('  蓉城老铁  ')).toBe('蓉城老铁')
    expect(validateNicknameInput('a')).toBeNull()
    expect(validateNicknameInput('球'.repeat(17))).toBeNull()
    expect(validateNicknameInput('13800138000')).toBeNull()
    expect(validateNicknameInput('微信用户')).toBeNull()
    expect(validateNicknameInput('wx_abc')).toBeNull()
    expect(validateNicknameInput('老铁\n换行')).toBeNull()
    expect(validateNicknameInput('<b>abc</b>')).toBeNull()
  })

  test('never prefills phone number or default username when profile incomplete', () => {
    expect(editableNicknameInitialValue(makeUser())).toBe('')
    expect(
      editableNicknameInitialValue(
        makeUser({
          display_name: 'footballfan',
          account_identifier: 'footballfan',
          profile_completed: false,
        }),
      ),
    ).toBe('')
    expect(
      editableNicknameInitialValue(
        makeUser({
          display_name: '凤凰山南看台',
          profile_completed: true,
        }),
      ),
    ).toBe('凤凰山南看台')
  })

  test('detects profile completion from explicit flag or fallback checks', () => {
    expect(isProfileComplete(null)).toBe(false)
    expect(isProfileComplete(makeUser({ profile_completed: false }))).toBe(false)
    expect(isProfileComplete(makeUser({ profile_completed: true }))).toBe(true)
    expect(
      isProfileComplete(
        makeUser({
          display_name: '13800138000',
          avatar_url: 'https://oryjk.cn:82/minio/football-insight/a.jpg',
        }),
      ),
    ).toBe(false)
    expect(
      isProfileComplete(
        makeUser({
          display_name: '蓉城老铁',
          avatar_url: 'https://oryjk.cn:82/minio/football-insight/a.jpg',
        }),
      ),
    ).toBe(true)
  })

  test('extracts machine-readable error code from ApiRequestError-like object', () => {
    expect(extractApiErrorCode({ body: { code: 'PROFILE_INCOMPLETE' } })).toBe('PROFILE_INCOMPLETE')
    expect(extractApiErrorCode({ body: null })).toBeNull()
  })
})
