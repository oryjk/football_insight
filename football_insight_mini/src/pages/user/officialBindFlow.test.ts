import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import { ApiRequestError } from '../../utils/apiError'
import { createUserOfficialBindFlow } from './officialBindFlow'
import { createAppSession } from '../../stores/appSession'
import * as authApi from '../../api/auth'
import { AUTH_TOKEN_STORAGE_KEY } from '../../utils/authStorage'

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

function setup() {
  const user = (id: string): CurrentUser => ({
    id, display_name: id, account_identifier: id, avatar_url: null,
    has_wechat_binding: true, membership_tier: 'V6', created_at: '',
  })
  const refreshes: Array<{ force?: boolean } | undefined> = []
  const session = {
    revision: ref(1), currentUser: ref<CurrentUser | null>(user('a')), hasAccessToken: ref(true),
    ensureSessionReady: async (options?: { force?: boolean }) => {
      refreshes.push(options)
      return session.currentUser.value
    },
  }
  const underReview = ref(false)
  const pending = deferred<{ code: string; expires_in_seconds: number }>()
  const copied: string[] = []
  let now = 1_000
  const flow = createUserOfficialBindFlow(session, underReview, {
    createCode: () => pending.promise,
    now: () => now,
    copy: (text) => { copied.push(text) },
  })
  return { session, underReview, pending, copied, flow, user, refreshes, setNow: (value: number) => { now = value } }
}

describe('official account bind flow', () => {
  test('opens without issuing a code, then displays and copies the command until absolute expiry', async () => {
    const { flow, pending, copied, setNow } = setup()
    flow.open()
    expect(flow.visible.value).toBe(true)
    expect(flow.code.value).toBe('')
    const request = flow.getCode()
    expect(flow.loading.value).toBe(true)
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    expect(flow.code.value).toBe('ABC234')
    expect(flow.expiresAt.value).toBe(601_000)
    flow.copyCode()
    expect(copied).toEqual(['绑定 ABC234'])
    setNow(601_000)
    flow.copyCode()
    expect(copied).toEqual(['绑定 ABC234'])
    flow.refreshCountdown()
    expect(flow.remainingSeconds.value).toBe(0)
  })

  test('drops old-account success immediately when actor changes even before a Vue update', async () => {
    const { flow, session, pending, user, copied } = setup()
    flow.open()
    expect(flow.visible.value).toBe(true)
    const request = flow.getCode()
    session.currentUser.value = user('b')
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    expect(flow.visible.value).toBe(false)
    expect(flow.code.value).toBe('')
    expect(flow.loading.value).toBe(false)
    flow.copyCode()
    expect(copied).toEqual([])
  })

  test('clears an already visible code on session revision change', async () => {
    const { flow, session, pending, copied } = setup()
    flow.open()
    const request = flow.getCode()
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    expect(flow.code.value).toBe('ABC234')
    session.revision.value += 1
    expect(flow.visible.value).toBe(false)
    expect(flow.code.value).toBe('')
    flow.copyCode()
    expect(copied).toEqual([])
  })

  test('drops late failure after close and reopening the sheet', async () => {
    const { flow, pending } = setup()
    flow.open()
    const request = flow.getCode()
    flow.close()
    flow.open()
    expect(flow.visible.value).toBe(true)
    pending.reject(new ApiRequestError('rate limited', 429))
    await request
    expect(flow.error.value).toBe('')
    expect(flow.loading.value).toBe(false)
    expect(flow.code.value).toBe('')
  })

  test('shows 429 feedback and stops loading after failure', async () => {
    const { flow, pending } = setup()
    flow.open()
    const request = flow.getCode()
    pending.reject(new ApiRequestError('server', 429))
    await request
    expect(flow.error.value).toBe('获取过于频繁，请稍后再试')
    expect(flow.loading.value).toBe(false)
  })

  test('closes and clears codes when review mode hides the entry', () => {
    const { flow, underReview } = setup()
    flow.open()
    expect(flow.visible.value).toBe(true)
    underReview.value = true
    expect(flow.visible.value).toBe(false)
    flow.open()
    expect(flow.visible.value).toBe(false)
  })

  test('removes a displayed code when a user refresh confirms official binding', async () => {
    const { flow, session, pending, copied } = setup()
    flow.open()
    const request = flow.getCode()
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    expect(flow.code.value).toBe('ABC234')
    session.currentUser.value = { ...session.currentUser.value!, official_wechat_bound: true }
    expect(flow.visible.value).toBe(false)
    expect(flow.code.value).toBe('')
    flow.copyCode()
    expect(copied).toEqual([])
  })

  test('old success and finally cannot modify a new account request', async () => {
    const session = { revision: ref(1), currentUser: ref<CurrentUser | null>({
      id: 'a', display_name: 'A', account_identifier: 'a', avatar_url: null,
      has_wechat_binding: true, membership_tier: 'V6', created_at: '',
    }), hasAccessToken: ref(true), ensureSessionReady: async () => null }
    const oldResponse = deferred<{ code: string; expires_in_seconds: number }>()
    const newResponse = deferred<{ code: string; expires_in_seconds: number }>()
    let next = oldResponse
    const flow = createUserOfficialBindFlow(session, ref(false), {
      createCode: () => next.promise, now: () => 1_000, copy: () => {},
    })
    flow.open()
    const oldRequest = flow.getCode()
    session.revision.value += 1
    next = newResponse
    flow.open()
    const newRequest = flow.getCode()
    oldResponse.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await oldRequest
    expect(flow.code.value).toBe('')
    expect(flow.loading.value).toBe(true)
    newResponse.resolve({ code: 'XYZ789', expires_in_seconds: 600 })
    await newRequest
    expect(flow.code.value).toBe('XYZ789')
    expect(flow.loading.value).toBe(false)
  })
})

describe('return from official account binding', () => {
  test('closing without a code refreshes once; internal cleanup does not refresh', async () => {
    const { flow, refreshes, session } = setup()
    flow.open()
    expect(refreshes).toEqual([])
    await flow.close()
    expect(flow.visible.value).toBe(false)
    expect(refreshes).toEqual([{ force: true }])
    await flow.close()
    flow.open()
    session.revision.value += 1
    flow.open()
    flow.dispose()
    expect(refreshes).toEqual([{ force: true }])
  })

  test('a failed close refresh keeps the sheet closed and existing user data', async () => {
    const { flow, session } = setup()
    const before = session.currentUser.value
    session.ensureSessionReady = async () => { throw new Error('offline') }
    flow.open()
    await flow.close()
    expect(flow.visible.value).toBe(false)
    expect(session.currentUser.value).toBe(before)
  })

  test('does not force ordinary page shows or merely opening the sheet', async () => {
    const { flow, refreshes } = setup()
    await flow.handlePageShow()
    flow.open()
    flow.handlePageHide()
    await flow.handlePageShow()
    expect(refreshes).toEqual([])
  })

  test('refreshes on close and again after leaving with an issued code', async () => {
    const { flow, pending, refreshes } = setup()
    flow.open()
    const request = flow.getCode()
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    await flow.handlePageShow()
    expect(refreshes).toEqual([])
    await flow.close()
    flow.handlePageHide()
    await flow.handlePageShow()
    flow.handlePageHide()
    await flow.handlePageShow()
    expect(refreshes).toEqual([{ force: true }, { force: true }])
  })

  test('cannot carry a pending return refresh to a replacement session', async () => {
    const { flow, session, pending, refreshes } = setup()
    flow.open()
    const request = flow.getCode()
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    flow.handlePageHide()
    session.revision.value += 1
    await flow.handlePageShow()
    expect(refreshes).toEqual([])
  })

  test('includes a code response delivered while the page was hidden', async () => {
    const { flow, pending, refreshes } = setup()
    flow.open()
    const request = flow.getCode()
    flow.handlePageHide()
    pending.resolve({ code: 'ABC234', expires_in_seconds: 600 })
    await request
    await flow.handlePageShow()
    expect(refreshes).toEqual([{ force: true }])
  })

  for (const trigger of ['return', 'close']) {
    test(`bypasses real appSession cache on binding ${trigger} and removes the entry`, async () => {
      const globalWithUni = globalThis as unknown as { uni: unknown }
      const previousUni = globalWithUni.uni
      const storage = new Map<string, unknown>([[AUTH_TOKEN_STORAGE_KEY, 'test-token']])
      globalWithUni.uni = {
        getStorageSync: (key: string) => storage.get(key),
        setStorageSync: (key: string, value: unknown) => storage.set(key, value),
        removeStorageSync: (key: string) => storage.delete(key),
      }
      try {
        const initial: CurrentUser = {
          id: 'a', display_name: 'A', account_identifier: 'a', avatar_url: null,
          has_wechat_binding: true, membership_tier: 'V6', created_at: '', official_wechat_bound: false,
        }
        let serverUser = initial
        let meCalls = 0
        const session = createAppSession({
          ...authApi,
          getCurrentUser: async () => { meCalls += 1; return serverUser },
        })
        await session.ensureSessionReady()
        const flow = createUserOfficialBindFlow(session, ref(false), {
          createCode: async () => ({ code: 'ABC234', expires_in_seconds: 600 }),
          now: () => Date.now(), copy: () => {},
        })
        flow.open()
        await flow.getCode()
        serverUser = { ...initial, official_wechat_bound: true }
        // A normal page refresh still has a fresh cached user.
        await session.ensureSessionReady()
        expect(meCalls).toBe(1)
        expect(flow.canOpen.value).toBe(true)
        if (trigger === 'close') {
          await flow.close()
        } else {
          flow.handlePageHide()
          await flow.handlePageShow()
        }
        expect(meCalls).toBe(2)
        expect(session.currentUser.value?.official_wechat_bound).toBe(true)
        expect(flow.canOpen.value).toBe(false)
        expect(flow.visible.value).toBe(false)
        await flow.handlePageShow()
        expect(meCalls).toBe(2)
        flow.dispose()
      } finally {
        globalWithUni.uni = previousUni
      }
    })
  }

  for (const trigger of ['return', 'close']) {
    test(`real appSession discards a binding ${trigger} response after another account logs in`, async () => {
      const globalWithUni = globalThis as unknown as { uni: unknown }
      const previousUni = globalWithUni.uni
      const storage = new Map<string, unknown>([[AUTH_TOKEN_STORAGE_KEY, 'a-token']])
      globalWithUni.uni = {
        getStorageSync: (key: string) => storage.get(key),
        setStorageSync: (key: string, value: unknown) => storage.set(key, value),
        removeStorageSync: (key: string) => storage.delete(key),
      }
      try {
        const a: CurrentUser = {
          id: 'a', display_name: 'A', account_identifier: 'a', avatar_url: null,
          has_wechat_binding: true, membership_tier: 'V6', created_at: '', official_wechat_bound: false,
        }
        const b: CurrentUser = { ...a, id: 'b', display_name: 'B', account_identifier: 'b' }
        const lateRefresh = deferred<CurrentUser | null>()
        let meCalls = 0
        const session = createAppSession({
          ...authApi,
          getCurrentUser: () => ++meCalls === 1 ? Promise.resolve(a) : lateRefresh.promise,
          login: async () => ({ user: b, access_token: 'b-token', expires_at: '2099-01-01T00:00:00Z' }),
        })
        await session.ensureSessionReady()
        const flow = createUserOfficialBindFlow(session, ref(false), {
          createCode: async () => ({ code: 'ABC234', expires_in_seconds: 600 }),
          now: () => Date.now(), copy: () => {},
        })
        flow.open()
        await flow.getCode()
        if (trigger === 'return') flow.handlePageHide()
        const returning = trigger === 'close' ? flow.close() : flow.handlePageShow()
        expect(meCalls).toBe(2)
        await session.loginWithPassword({ account_identifier: 'b', password: 'test' })
        lateRefresh.resolve({ ...a, official_wechat_bound: true })
        await returning
        expect(session.currentUser.value?.id).toBe('b')
        expect(session.currentUser.value?.official_wechat_bound).toBe(false)
        expect(flow.code.value).toBe('')
        expect(flow.visible.value).toBe(false)
        flow.handlePageHide()
        await flow.handlePageShow()
        expect(meCalls).toBe(2)
        flow.dispose()
      } finally {
        globalWithUni.uni = previousUni
      }
    })
  }
})
