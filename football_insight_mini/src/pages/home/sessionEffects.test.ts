import { afterEach, beforeEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick, watch } from 'vue'
import { createHomeSessionSync } from './sessionEffects'
import { useHomeSupportData, type HomeSupportData } from './useHomeSupportData'
import { createAppSession, type AppSession } from '../../stores/appSession'
import type * as authApi from '../../api/auth'
import type { AuthResponse, CurrentUser, MiniWechatLoginResponse } from '../../types/auth'
import type { SupportProfile, SupportTeam } from '../../types/support'
import { ApiRequestError } from '../../utils/apiError'
import { AUTH_TOKEN_STORAGE_KEY } from '../../utils/authStorage'

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
      resetSessionScopedState: () => {},
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
      resetSessionScopedState: () => {},
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

function buildSupportProfile(teamId: number, teamName: string): SupportProfile {
  return {
    favorite_team: {
      team_id: teamId,
      team_name: teamName,
      avatar_storage_url: null,
      rank_no: null,
    },
    next_match: null,
  }
}

interface HomeSupportFixture {
  session: AppSession
  support: HomeSupportData
  profileRequests: Array<Deferred<SupportProfile>>
  meCalls: () => number
  stop: () => void
}

/**
 * 与页面相同的真实接线：真实 createAppSession + 真实 useHomeSupportData 加载编排 +
 * 真实 sessionEffects + 真实 Vue watch（revision 先清缓存，currentUser 再刷新）。
 */
function wireHomeSupportFixture(options: {
  source: Partial<AuthSource>
  getMiniWechatCode?: () => Promise<string>
  setFavoriteTeam?: (payload: { team_id: number }) => Promise<SupportTeam>
}): HomeSupportFixture {
  let meCalls = 0
  const source = { ...options.source } as Record<string, unknown>
  if (options.source.getCurrentUser) {
    const getCurrentUser = options.source.getCurrentUser
    source.getCurrentUser = () => {
      meCalls += 1
      return getCurrentUser()
    }
  }
  const session = createAppSession(source as unknown as AuthSource, {
    getMiniWechatCode: options.getMiniWechatCode,
  })
  const profileRequests: Array<Deferred<SupportProfile>> = []
  const support = useHomeSupportData({
    session,
    getSupportProfile: () => {
      const request = deferred<SupportProfile>()
      profileRequests.push(request)
      return request.promise
    },
    listSupportTeams: () => Promise.resolve([]),
    setFavoriteTeam: options.setFavoriteTeam,
  })
  const sync = createHomeSessionSync({
    loadSupportData: support.loadSupportData,
    resetSessionScopedState: support.resetSessionScopedState,
  })
  const scope = effectScope()
  scope.run(() => {
    // 与页面相同的真实组合 watch：revision 变化先清缓存；只有 user 引用变化才触发私有读取
    // （beginAuthIntent 只递增 revision、currentUser 仍是旧账号，此时不能发旧 token 的请求）。
    watch([session.revision, session.currentUser], ([revision, user], [previousRevision, previousUser]) => {
      if (revision !== previousRevision) {
        sync.handleSessionRevisionChange()
      }
      if (user !== previousUser) {
        sync.handleSessionUserPublished(user)
      }
    })
  })
  return {
    session,
    support,
    profileRequests,
    meCalls: () => meCalls,
    stop: () => scope.stop(),
  }
}

describe('home 私有加载编排（真实 session + watch + 加载入口）', () => {
  test('冷启动旧 token：me 确认前不发私有请求；me 401 后仍走微信恢复并加载新会话资料', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-expired'
    const meRequest = deferred<CurrentUser | null>()
    let codeCalls = 0
    const fixture = wireHomeSupportFixture({
      source: {
        getCurrentUser: () => meRequest.promise,
        loginWithMiniWechat: () =>
          Promise.resolve({
            status: 'authenticated',
            access_token: 'token-bound',
            expires_at: '2026-02-01T00:00:00Z',
            user: userA,
          } as MiniWechatLoginResponse),
      },
      getMiniWechatCode: () => {
        codeCalls += 1
        return Promise.resolve('code-1')
      },
    })
    const { session, support, profileRequests } = fixture

    // 与生产一致：App bootstrap 等待 /auth/me，页面 onShow 同时发起私有读取。
    const bootstrap = session.bootstrapSession({ allowSilentWechat: true })
    void support.loadSupportData()
    await flushMicrotasks()

    // me 未确认前：私有读取等待共享 ensure，不抢发 profile 请求（也就不会提前 401 取消恢复）。
    expect(profileRequests.length).toBe(0)
    expect(session.currentUser.value).toBeNull()

    // 旧 token 的 /auth/me 返回 401：恢复不被取消，继续微信探测并发布已绑定用户。
    meRequest.resolve(null)
    await expect(bootstrap).resolves.toMatchObject({ id: 'a' })
    await nextTick()
    await flushMicrotasks()

    expect(codeCalls).toBe(1)
    expect(session.currentUser.value).toMatchObject({ id: 'a' })
    // 恢复发布后 watcher 触发恰好一次私有请求。
    expect(profileRequests.length).toBe(1)

    profileRequests[0].resolve(buildSupportProfile(7, '绑定账号主队'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_id).toBe(7)
    expect(support.supportLoading.value).toBe(false)
    expect(support.loginPromptVisible.value).toBe(false)
    fixture.stop()
  })

  test('有效 token 冷启动：onShow 与恢复 watcher 同会话合并为一次私有请求', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    const fixture = wireHomeSupportFixture({
      source: { getCurrentUser: () => meRequest.promise },
    })
    const { session, support, profileRequests } = fixture

    // H5 冷刷新场景：bootstrap 与页面 onShow 并发，me 延迟返回。
    const bootstrap = session.bootstrapSession({ allowSilentWechat: false })
    void support.loadSupportData()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(0)

    meRequest.resolve(userA)
    await expect(bootstrap).resolves.toMatchObject({ id: 'a' })
    await nextTick()
    await flushMicrotasks()

    // 共享 ensure 合并 me；onShow 与 currentUser watcher 同 revision 合并私有请求。
    expect(fixture.meCalls()).toBe(1)
    expect(profileRequests.length).toBe(1)

    profileRequests[0].resolve(buildSupportProfile(1, 'A-private'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_name).toBe('A-private')
    expect(support.supportLoading.value).toBe(false)
    fixture.stop()
  })

  test('切换账号：A 缓存同步清空，旧 A 迟到成功不改 B 的 loading/缓存', async () => {
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, support, profileRequests } = fixture

    // 登录 A 并加载出缓存。
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(1)
    profileRequests[0].resolve(buildSupportProfile(1, 'A-private'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_name).toBe('A-private')
    expect(support.supportLoading.value).toBe(false)

    // A 的正常刷新在途：同账号刷新保留缓存。
    const oldARefresh = support.loadSupportData()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)
    expect(support.supportProfile.value?.favorite_team?.team_name).toBe('A-private')

    // 切换登录 B：会话身份变化同步清掉 A 缓存，B 的私有请求在途。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(support.supportProfile.value).toBeNull()
    expect(profileRequests.length).toBe(3)
    expect(support.supportLoading.value).toBe(true)

    // 旧 A 刷新迟到成功：不发布旧缓存，也不清掉 B 请求的 loading。
    profileRequests[1].resolve(buildSupportProfile(1, 'A-private'))
    await oldARefresh
    await flushMicrotasks()
    expect(support.supportProfile.value).toBeNull()
    expect(support.supportLoading.value).toBe(true)

    // B 成功：发布 B 数据并结束 loading。
    profileRequests[2].resolve(buildSupportProfile(2, 'B-private'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_name).toBe('B-private')
    expect(support.supportLoading.value).toBe(false)
    fixture.stop()
  })

  test('切换账号：旧 A 迟到失败不写错误、不清 B 的 loading', async () => {
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, support, profileRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    profileRequests[0].resolve(buildSupportProfile(1, 'A-private'))
    await flushMicrotasks()

    const oldARefresh = support.loadSupportData()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)

    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(3)
    expect(support.supportLoading.value).toBe(true)

    // 旧 A 刷新迟到失败：错误不发布，loading 仍属于 B。
    profileRequests[1].reject(new ApiRequestError('请求失败（500）', 500))
    await oldARefresh
    await flushMicrotasks()
    expect(support.supportErrorMessage.value).toBe('')
    expect(support.supportLoading.value).toBe(true)
    expect(support.supportProfile.value).toBeNull()

    profileRequests[2].resolve(buildSupportProfile(2, 'B-private'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_name).toBe('B-private')
    expect(support.supportLoading.value).toBe(false)
    fixture.stop()
  })

  test('写后强制刷新：不与写前在途 GET 合并，旧 GET 迟到成功不覆盖新选择', async () => {
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, support, profileRequests } = fixture

    // 登录 A 并加载出缓存（当前主队 team1）。
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(1)
    profileRequests[0].resolve(buildSupportProfile(1, 'team1'))
    await flushMicrotasks()
    expect(support.selectedFavoriteTeamId.value).toBe(1)

    // 写前普通读取在途（快照仍为 team1）。
    const preWriteRead = support.loadSupportData()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)

    // 设置主队写成功后的写后刷新（页面 handleConfirmFavoriteTeam 的实际入口）：
    // 不与写前在途 GET 合并，直接发新请求。
    const postWriteRefresh = support.loadSupportData({ force: true })
    await flushMicrotasks()
    expect(profileRequests.length).toBe(3)
    expect(support.supportLoading.value).toBe(true)

    // 写前旧 GET 迟到成功（旧主队 team1）：不覆盖，也不清写后请求的 loading。
    profileRequests[1].resolve(buildSupportProfile(1, 'team1'))
    await preWriteRead
    await flushMicrotasks()
    expect(support.supportLoading.value).toBe(true)
    expect(support.supportProfile.value?.favorite_team?.team_id).toBe(1)

    // 写后 GET 返回新主队：正常发布。
    profileRequests[2].resolve(buildSupportProfile(2, 'team2'))
    await postWriteRefresh
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_id).toBe(2)
    expect(support.selectedFavoriteTeamId.value).toBe(2)
    expect(support.supportLoading.value).toBe(false)
    fixture.stop()
  })

  test('写后强制刷新：写前旧 GET 迟到失败不写错误、不清写后请求的 loading', async () => {
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, support, profileRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    profileRequests[0].resolve(buildSupportProfile(1, 'team1'))
    await flushMicrotasks()

    const preWriteRead = support.loadSupportData()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)

    const postWriteRefresh = support.loadSupportData({ force: true })
    await flushMicrotasks()
    expect(profileRequests.length).toBe(3)

    // 写前旧 GET 迟到失败：错误不发布，loading 仍属于写后请求。
    profileRequests[1].reject(new ApiRequestError('stale read fail', 500))
    await preWriteRead
    await flushMicrotasks()
    expect(support.supportErrorMessage.value).toBe('')
    expect(support.supportLoading.value).toBe(true)

    profileRequests[2].resolve(buildSupportProfile(2, 'team2'))
    await postWriteRefresh
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_id).toBe(2)
    expect(support.supportLoading.value).toBe(false)
    fixture.stop()
  })

  test('同账号设置主队成功：submit 返回 true 且提交状态复位', async () => {
    const putRequests: Array<Deferred<SupportTeam>> = []
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
      setFavoriteTeam: () => {
        const request = deferred<SupportTeam>()
        putRequests.push(request)
        return request.promise
      },
    })
    const { session, support, profileRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    profileRequests[0].resolve(buildSupportProfile(1, 'team1'))
    await flushMicrotasks()

    const submit = support.submitFavoriteTeam(2)
    await flushMicrotasks()
    expect(putRequests.length).toBe(1)
    expect(support.favoriteSubmitting.value).toBe(true)

    putRequests[0].resolve({ team_id: 2, team_name: 'team2', avatar_storage_url: null, rank_no: null })
    // true：页面侧继续关闭 sheet/toast，并接 loadSupportData({ force: true }) 写后刷新。
    await expect(submit).resolves.toBe(true)
    expect(support.favoriteSubmitting.value).toBe(false)
    fixture.stop()
  })

  test('设置主队写等待期间切换账号：旧 PUT 成功不关闭新账号 sheet、不强制刷新其读取', async () => {
    const putRequests: Array<Deferred<SupportTeam>> = []
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
      setFavoriteTeam: () => {
        const request = deferred<SupportTeam>()
        putRequests.push(request)
        return request.promise
      },
    })
    const { session, support, profileRequests } = fixture

    // 登录 A 并加载缓存。
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(1)
    profileRequests[0].resolve(buildSupportProfile(1, 'team1'))
    await flushMicrotasks()

    // A 发起设置主队写（PUT 在途）。
    const submit = support.submitFavoriteTeam(2)
    await flushMicrotasks()
    expect(putRequests.length).toBe(1)
    expect(support.favoriteSubmitting.value).toBe(true)

    // 写等待期间登录 B：缓存/提交状态同步复位，B 的 profile GET 在途；B 打开了自己的主队 sheet。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)
    expect(support.favoriteSubmitting.value).toBe(false)
    support.favoriteTeamSheetVisible.value = true

    // A 的旧 PUT 迟到成功：提交判为失效（false），真实 handler 凭 false 不关闭 B 的
    // sheet、不 toast、不 force 刷新替换 B 在途的 GET。
    putRequests[0].resolve({ team_id: 2, team_name: 'team2', avatar_storage_url: null, rank_no: null })
    await expect(submit).resolves.toBe(false)
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)
    expect(support.favoriteTeamSheetVisible.value).toBe(true)
    expect(support.favoriteSubmitting.value).toBe(false)

    // B 的读取正常发布、不被旧写干扰。
    profileRequests[1].resolve(buildSupportProfile(3, 'B-team'))
    await flushMicrotasks()
    expect(support.supportProfile.value?.favorite_team?.team_id).toBe(3)
    expect(support.supportLoading.value).toBe(false)
    expect(support.supportErrorMessage.value).toBe('')
    fixture.stop()
  })

  test('设置主队写等待期间切换账号：旧 PUT 迟到失败不抛出，finally 不清新提交的 submitting', async () => {
    const putRequests: Array<Deferred<SupportTeam>> = []
    const fixture = wireHomeSupportFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
      setFavoriteTeam: () => {
        const request = deferred<SupportTeam>()
        putRequests.push(request)
        return request.promise
      },
    })
    const { session, support, profileRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    profileRequests[0].resolve(buildSupportProfile(1, 'team1'))
    await flushMicrotasks()

    // A 的写在途 → 切换 B → B 加载完成后发起自己的新写（新 owner）。
    const submit = support.submitFavoriteTeam(2)
    await flushMicrotasks()
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(profileRequests.length).toBe(2)
    profileRequests[1].resolve(buildSupportProfile(3, 'B-team'))
    await flushMicrotasks()

    const submitB = support.submitFavoriteTeam(4)
    await flushMicrotasks()
    expect(putRequests.length).toBe(2)
    expect(support.favoriteSubmitting.value).toBe(true)

    // A 的旧 PUT 迟到失败：返回 false 不抛出（页面侧不 toast）；旧 finally 不清 B 提交的 submitting。
    putRequests[0].reject(new ApiRequestError('old write fail', 500))
    await expect(submit).resolves.toBe(false)
    await flushMicrotasks()
    expect(support.favoriteSubmitting.value).toBe(true)
    expect(support.supportErrorMessage.value).toBe('')

    // B 的写成功：返回 true，finally 正常复位。
    putRequests[1].resolve({ team_id: 4, team_name: 'team4', avatar_storage_url: null, rank_no: null })
    await expect(submitB).resolves.toBe(true)
    await flushMicrotasks()
    expect(support.favoriteSubmitting.value).toBe(false)
    fixture.stop()
  })

  test('ensure 等待期间用户退出：迟到的 ensure 回调不再发私有请求', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    const fixture = wireHomeSupportFixture({
      source: {
        getCurrentUser: () => meRequest.promise,
        logout: () => Promise.resolve(),
      },
    })
    const { session, support, profileRequests } = fixture

    void support.loadSupportData()
    await flushMicrotasks()
    await session.logoutSession()

    // 退出后迟到的 me 成功：ensure 已被替代，不能再发私有请求。
    meRequest.resolve(userA)
    await nextTick()
    await flushMicrotasks()

    expect(profileRequests.length).toBe(0)
    expect(support.supportProfile.value).toBeNull()
    expect(support.supportLoading.value).toBe(false)
    expect(support.loginPromptVisible.value).toBe(true)
    fixture.stop()
  })
})
