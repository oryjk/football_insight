import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import {
  AUTH_TOKEN_STORAGE_KEY,
  MANUAL_LOGOUT_STORAGE_KEY,
  clearManualLogout,
  getAccessToken,
  hasManualLogout,
  setAccessToken,
  setManualLogout,
} from './authStorage'

const globalWithUni = globalThis as unknown as { uni?: unknown }
const previousUni = globalWithUni.uni

let storage: Record<string, unknown>

beforeEach(() => {
  storage = {}
  globalWithUni.uni = {
    getStorageSync: (key: string) => storage[key] ?? '',
    setStorageSync: (key: string, value: unknown) => {
      storage[key] = value
    },
    removeStorageSync: (key: string) => {
      delete storage[key]
    },
  }
})

afterEach(() => {
  globalWithUni.uni = previousUni
})

describe('manual logout flag storage', () => {
  test('默认无标记；set 后跨读取生效；clear 后移除', () => {
    expect(hasManualLogout()).toBe(false)

    setManualLogout()
    expect(hasManualLogout()).toBe(true)
    expect(storage[MANUAL_LOGOUT_STORAGE_KEY]).toBe('true')

    clearManualLogout()
    expect(hasManualLogout()).toBe(false)
    expect(MANUAL_LOGOUT_STORAGE_KEY in storage).toBe(false)
  })

  test('标记读写不影响其他本地数据（token 等）', () => {
    setAccessToken('token-a')
    storage['unrelated_key'] = 'keep-me'

    setManualLogout()
    clearManualLogout()

    expect(getAccessToken()).toBe('token-a')
    expect(storage['unrelated_key']).toBe('keep-me')
    expect(storage[AUTH_TOKEN_STORAGE_KEY]).toBe('token-a')
  })
})
