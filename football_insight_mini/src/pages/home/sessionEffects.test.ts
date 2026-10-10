import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick, watch } from 'vue'
import { createHomeSessionSync } from './sessionEffects'
import { createAppSession } from '../../stores/appSession'
import type * as authApi from '../../api/auth'
import type { AuthResponse, CurrentUser, MiniWechatLoginResponse } from '../../types/auth'

type AuthSource = typeof authApi

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (error: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

async function flushMicrotasks(times = 20): Promise<void> {
  for (let index = 0; index < times; index += 1) {
    await Promise.resolve()
  }
}

const userA: CurrentUser = {
  id: 'a',
  display_name: '用户a',
  account_identifier: 'account-a',
  avatar_url: null,
  has_wechat_binding: true,
  membership_tier: 'FREE',
  created_at: '2026-01-01T00:00:00Z',
}

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

describe('home session sync（后台恢复后刷新助力面板）', () => {
  test('公开数据在 code 等待期间已展示；恢复发布后助力面板自动刷新', async () => {
    const codeRequest = deferred<string>()
    const source = {
      loginWithMiniWechat: () =>
        Promise.resolve({
          status: 'authenticated',
          access_token: 'token-a',
          expires_at: '2026-02-01T00:00:00Z',
          user: userA,
        } as MiniWechatLoginResponse),
    } as unknown as AuthSource
    const session = createAppSession(source, { getMiniWechatCode: () => codeRequest.promise })

    let supportLoads = 0
    const sync = createHomeSessionSync({
      loadSupportData: async () => {
        supportLoads += 1
      },
    })
    const scope = effectScope()
    scope.run(() => {
      watch(session.currentUser, (user) => {
        sync.handleSessionUserPublished(user)
      })
    })

    // 后台启动恢复开始等待微信 code；公开首页加载不等待会话。
    const bootstrap = session.bootstrapSession({ allowSilentWechat: true })
    const publicOverview = deferred<string>()
    let publicShown = false
    void publicOverview.promise.then(() => {
      publicShown = true
    })
    publicOverview.resolve('overview')
    await flushMicrotasks()
    expect(publicShown).toBe(true)
    expect(supportLoads).toBe(0)
    expect(session.currentUser.value).toBeNull()

    // 恢复完成：code 返回并换到 token，watcher 触发助力面板刷新。
    codeRequest.resolve('code-1')
    await expect(bootstrap).resolves.toMatchObject({ id: 'a' })
    await nextTick()
    await flushMicrotasks()

    expect(supportLoads).toBe(1)
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    scope.stop()
  })

  test('退出清空用户后助力面板同样刷新（回到登录引导）', async () => {
    const source = {
      login: () =>
        Promise.resolve({
          access_token: 'token-a',
          expires_at: '2026-02-01T00:00:00Z',
          user: userA,
        } as AuthResponse),
      logout: () => Promise.resolve(),
    } as unknown as AuthSource
    const session = createAppSession(source)
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })

    let supportLoads = 0
    const sync = createHomeSessionSync({
      loadSupportData: async () => {
        supportLoads += 1
      },
    })
    const scope = effectScope()
    scope.run(() => {
      watch(session.currentUser, (user) => {
        sync.handleSessionUserPublished(user)
      })
    })

    await session.logoutSession()
    await nextTick()

    expect(supportLoads).toBe(1)
    expect(session.currentUser.value).toBeNull()
    scope.stop()
  })
})
