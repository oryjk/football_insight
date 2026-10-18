import { beforeEach, afterEach, describe, expect, test } from 'bun:test'
import { effectScope, nextTick, ref, watch } from 'vue'
import { createInsightsSessionSync } from './sessionEffects'
import { useInsightsData, type InsightsData } from './useInsightsData'
import { createAppSession, type AppSession } from '../../stores/appSession'
import type * as authApi from '../../api/auth'
import type { AuthResponse, CurrentUser } from '../../types/auth'
import type { TeamInsight, TeamInsightsResponse } from '../../types/insight'
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
  membership_tier: 'V3',
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

describe('insights session sync（恢复发布后刷新会员洞察）', () => {
  /** 与页面相同的接线，走真实 Vue watch。 */
  function wireSync(options: { underReview: boolean }) {
    const currentUser = ref<CurrentUser | null>(null)
    let insightLoads = 0
    const sync = createInsightsSessionSync({
      isUnderReview: () => options.underReview,
      loadInsights: async () => {
        insightLoads += 1
      },
      resetSessionScopedState: () => {},
    })
    const scope = effectScope()
    scope.run(() => {
      watch(currentUser, (user) => {
        sync.handleSessionUserPublished(user)
      })
    })
    return {
      currentUser,
      get insightLoads() {
        return insightLoads
      },
      stop: () => scope.stop(),
    }
  }

  test('后台恢复发布已认证用户：自动加载洞察，无需切 Tab', async () => {
    const fixture = wireSync({ underReview: false })

    fixture.currentUser.value = userA
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(1)
    fixture.stop()
  })

  test('退出清空用户：不加载洞察（模板已切游客视图）', async () => {
    const fixture = wireSync({ underReview: false })
    fixture.currentUser.value = userA
    await nextTick()

    fixture.currentUser.value = null
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(1)
    fixture.stop()
  })

  test('审核中即使发布用户也不拉取会员数据', async () => {
    const fixture = wireSync({ underReview: true })

    fixture.currentUser.value = userA
    await nextTick()
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(fixture.insightLoads).toBe(0)
    fixture.stop()
  })
})

function buildTeamInsightsResponse(teamId: number): TeamInsightsResponse {
  return {
    view_kind: 'live',
    round_number: null,
    current_season: 2026,
    teams: [{ team_id: teamId, team_name: `球队${teamId}` } as TeamInsightsResponse['teams'][number]],
    insights: [{ team_id: teamId, team_name: `球队${teamId}` } as TeamInsight],
  }
}

interface InsightsFixture {
  session: AppSession
  data: InsightsData
  insightRequests: Array<Deferred<TeamInsightsResponse>>
  stop: () => void
}

/**
 * 与页面相同的真实接线：真实 createAppSession + 真实 useInsightsData 加载编排 +
 * 真实 sessionEffects + 真实 Vue watch（revision 变化先清缓存再刷新）。
 */
function wireInsightsFixture(options: {
  source: Partial<AuthSource>
}): InsightsFixture {
  const session = createAppSession(options.source as unknown as AuthSource)
  const insightRequests: Array<Deferred<TeamInsightsResponse>> = []
  const data = useInsightsData({
    session,
    isUnderReview: () => false,
    isMembershipLocked: () => false,
    getLiveTeamInsights: () => {
      const request = deferred<TeamInsightsResponse>()
      insightRequests.push(request)
      return request.promise
    },
  })
  const sync = createInsightsSessionSync({
    isUnderReview: () => false,
    loadInsights: data.loadInsights,
    resetSessionScopedState: data.resetSessionScopedState,
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
    data,
    insightRequests,
    stop: () => scope.stop(),
  }
}

describe('insights 私有加载编排（真实 session + watch + 加载入口）', () => {
  test('会员贡献资料迟到时保留免费倾向面板中已选的有效球队', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const fixture = wireInsightsFixture({ source: { getCurrentUser: async () => userA } })
    await fixture.data.loadUser()
    const pending = fixture.data.loadInsights()
    fixture.data.selectedTeamId.value = 2
    const response = buildTeamInsightsResponse(1)
    response.teams.push(buildTeamInsightsResponse(2).teams[0]!)
    fixture.insightRequests[0]!.resolve(response)
    await pending
    expect(fixture.data.selectedTeamId.value).toBe(2)
    fixture.stop()
  })
  test('onShow（loadUser→loadInsights）与恢复 watcher 同会话只发一次私有请求', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    const fixture = wireInsightsFixture({
      source: { getCurrentUser: () => meRequest.promise },
    })
    const { session, data, insightRequests } = fixture

    // 页面 onShow 的实际组合：先确认用户再加载洞察；me 延迟返回（H5 冷刷新）。
    const shown = (async () => {
      const stillCurrent = await data.loadUser()
      if (!stillCurrent) {
        return
      }
      await data.loadInsights()
    })()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(0)

    meRequest.resolve(userA)
    await nextTick()
    await flushMicrotasks()

    // currentUser watcher 与 onShow 同 revision 合并：只有 1 次 team-insights 请求。
    expect(insightRequests.length).toBe(1)

    insightRequests[0].resolve(buildTeamInsightsResponse(2))
    await shown
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(2)
    expect(data.errorMessage.value).toBe('')
    expect(data.loading.value).toBe(false)
    fixture.stop()
  })

  test('切换账号：A 缓存同步清空，旧 A 迟到成功不改 B 的 loading/缓存', async () => {
    const fixture = wireInsightsFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, data, insightRequests } = fixture

    // 登录 A 并加载出缓存。
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(1)
    insightRequests[0].resolve(buildTeamInsightsResponse(1))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(1)
    expect(data.loading.value).toBe(false)

    // A 的正常刷新在途：同账号刷新保留缓存。
    const oldARefresh = data.loadInsights()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(2)
    expect(data.selectedTeamId.value).toBe(1)

    // 切换登录 B：会话身份变化同步清掉 A 缓存，B 的私有请求在途。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(data.insights.value).toEqual([])
    expect(data.selectedTeamId.value).toBeNull()
    expect(insightRequests.length).toBe(3)
    expect(data.loading.value).toBe(true)

    // 旧 A 刷新迟到成功：不发布旧缓存，也不清掉 B 请求的 loading。
    insightRequests[1].resolve(buildTeamInsightsResponse(1))
    await oldARefresh
    await flushMicrotasks()
    expect(data.loading.value).toBe(true)
    expect(data.selectedTeamId.value).toBeNull()

    // B 成功：发布 B 数据并结束 loading。
    insightRequests[2].resolve(buildTeamInsightsResponse(2))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(2)
    expect(data.loading.value).toBe(false)
    expect(data.errorMessage.value).toBe('')
    fixture.stop()
  })

  test('切换账号：旧 A 迟到失败不写错误、不清 B 的 loading', async () => {
    const fixture = wireInsightsFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, data, insightRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    insightRequests[0].resolve(buildTeamInsightsResponse(1))
    await flushMicrotasks()

    const oldARefresh = data.loadInsights()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(2)

    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(3)
    expect(data.loading.value).toBe(true)

    // 旧 A 刷新迟到失败：错误不发布，loading 仍属于 B。
    insightRequests[1].reject(new ApiRequestError('old request fail', 500))
    await oldARefresh
    await flushMicrotasks()
    expect(data.errorMessage.value).toBe('')
    expect(data.loading.value).toBe(true)

    insightRequests[2].resolve(buildTeamInsightsResponse(2))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(2)
    expect(data.loading.value).toBe(false)
    fixture.stop()
  })

  test('旧 onShow 等待 me 期间账号切换：失效 continuation 不再读取/遮盖新账号数据', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    const fixture = wireInsightsFixture({
      source: {
        getCurrentUser: () => meRequest.promise,
        login: () =>
          Promise.resolve({
            access_token: 'token-b',
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: 'b' },
          } as AuthResponse),
      },
    })
    const { session, data, insightRequests } = fixture

    // 与页面 onShow 相同：确认用户后才加载洞察；A 的 me 在途。
    const shown = (async () => {
      const stillCurrent = await data.loadUser()
      if (!stillCurrent) {
        return
      }
      await data.loadInsights()
    })()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(0)

    // me 等待期间用户登录 B：组合 watcher 发布 B 并成功加载 B 的洞察。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(1)
    insightRequests[0].resolve(buildTeamInsightsResponse(2))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(2)

    // A 的旧 me 迟到：被 session 判 superseded，loadUser 返回 false，
    // 旧 onShow 必须停止后续阶段——不再发第二次请求（其失败会遮盖 B 已成功的数据）。
    meRequest.resolve(userA)
    await shown
    await flushMicrotasks()

    expect(insightRequests.length).toBe(1)
    expect(session.currentUser.value?.id).toBe('b')
    expect(data.selectedTeamId.value).toBe(2)
    expect(data.errorMessage.value).toBe('')
    expect(data.loading.value).toBe(false)
    fixture.stop()
  })

  test('审核查询等待期间账号切换：审核 false 照常发布，旧 onShow 不续私有请求', async () => {
    const fixture = wireInsightsFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, data, insightRequests } = fixture

    // A 登录并加载出缓存。
    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(1)
    insightRequests[0].resolve(buildTeamInsightsResponse(1))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(1)

    // 与页面 onShow 相同的实际组合：操作身份在第一个 await（审核查询）之前建立；
    // 审核结果与账号无关照常发布，私有链路在审核/loadUser 每个 await 之后核对身份。
    const reviewFlag = ref(false)
    const auditRequest = deferred<boolean>()
    const shown = (async () => {
      const shownRevision = session.revision.value
      const underReview = await auditRequest.promise
      reviewFlag.value = underReview
      if (underReview) {
        data.applyUnderReviewView()
        return
      }
      if (session.revision.value !== shownRevision) {
        return
      }
      const stillCurrent = await data.loadUser()
      if (!stillCurrent) {
        return
      }
      await data.loadInsights()
    })()
    await flushMicrotasks()

    // 审核查询在途期间用户登录 B：组合 watcher 发布 B 并成功加载 B 的洞察。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(2)
    insightRequests[1].resolve(buildTeamInsightsResponse(2))
    await flushMicrotasks()
    expect(data.selectedTeamId.value).toBe(2)

    // 审核查询迟到返回 false：审核 gate 照常发布；旧 onShow 已失效——
    // 不再 loadUser/loadInsights 重读 B，后续读取失败也不会遮盖 B 已加载成功的数据。
    auditRequest.resolve(false)
    await shown
    await flushMicrotasks()

    expect(reviewFlag.value).toBe(false)
    expect(insightRequests.length).toBe(2)
    expect(session.currentUser.value?.id).toBe('b')
    expect(data.selectedTeamId.value).toBe(2)
    expect(data.errorMessage.value).toBe('')
    expect(data.loading.value).toBe(false)
    fixture.stop()
  })

  test('审核查询等待期间账号切换且审核中：仍发布审核 gate 并使新账号在途读取失效', async () => {
    const fixture = wireInsightsFixture({
      source: {
        login: (payload: { account_identifier: string }) =>
          Promise.resolve({
            access_token: `token-${payload.account_identifier}`,
            expires_at: '2026-02-01T00:00:00Z',
            user: { ...userA, id: payload.account_identifier },
          } as AuthResponse),
      },
    })
    const { session, data, insightRequests } = fixture

    await session.loginWithPassword({ account_identifier: 'a', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(1)
    insightRequests[0].resolve(buildTeamInsightsResponse(1))
    await flushMicrotasks()

    // 与页面 onShow 相同的实际组合。
    const reviewFlag = ref(false)
    const auditRequest = deferred<boolean>()
    const shown = (async () => {
      const shownRevision = session.revision.value
      const underReview = await auditRequest.promise
      reviewFlag.value = underReview
      if (underReview) {
        data.applyUnderReviewView()
        return
      }
      if (session.revision.value !== shownRevision) {
        return
      }
      const stillCurrent = await data.loadUser()
      if (!stillCurrent) {
        return
      }
      await data.loadInsights()
    })()
    await flushMicrotasks()

    // 审核查询在途期间用户登录 B：watcher 发起 B 的读取（仍在途）。
    await session.loginWithPassword({ account_identifier: 'b', password: 'x' })
    await nextTick()
    await flushMicrotasks()
    expect(insightRequests.length).toBe(2)
    expect(data.loading.value).toBe(true)

    // 审核查询迟到返回 true：审核限制与账号无关，必须保留——发布审核 gate、
    // 清掉会员数据并使 B 的在途读取 owner 失效；不再 loadUser/loadInsights。
    auditRequest.resolve(true)
    await shown
    await flushMicrotasks()

    expect(reviewFlag.value).toBe(true)
    expect(insightRequests.length).toBe(2)
    expect(data.teams.value).toEqual([])
    expect(data.selectedTeamId.value).toBeNull()
    expect(data.loading.value).toBe(false)

    // B 的在途读取迟到成功：owner 已被审核视图失效，不再发布会员数据。
    insightRequests[1].resolve(buildTeamInsightsResponse(2))
    await flushMicrotasks()
    expect(data.teams.value).toEqual([])
    expect(data.selectedTeamId.value).toBeNull()
    expect(data.errorMessage.value).toBe('')
    fixture.stop()
  })

  test('ensure 等待期间用户退出：迟到的 ensure 回调不再读取/发布', async () => {
    storage[AUTH_TOKEN_STORAGE_KEY] = 'token-a'
    const meRequest = deferred<CurrentUser | null>()
    const fixture = wireInsightsFixture({
      source: {
        getCurrentUser: () => meRequest.promise,
        logout: () => Promise.resolve(),
      },
    })
    const { session, data, insightRequests } = fixture

    const shown = (async () => {
      const stillCurrent = await data.loadUser()
      if (!stillCurrent) {
        return
      }
      await data.loadInsights()
    })()
    await flushMicrotasks()
    await session.logoutSession()

    // 退出后迟到的 me 成功：ensure 已被替代，不再发私有请求。
    meRequest.resolve(userA)
    await shown
    await nextTick()
    await flushMicrotasks()

    expect(insightRequests.length).toBe(0)
    expect(data.userLoading.value).toBe(false)
    expect(data.loading.value).toBe(false)
    expect(data.errorMessage.value).toBe('')
    fixture.stop()
  })
})
