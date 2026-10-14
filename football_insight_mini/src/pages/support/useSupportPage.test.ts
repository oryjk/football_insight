import { afterEach, describe, expect, test } from 'bun:test'
import { effectScope, ref } from 'vue'
import { useSupportPage, type SupportPageDeps } from './useSupportPage'
import type { SupportMatchDetail } from '../../types/support'
import { ApiRequestError } from '../../utils/apiError'

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

const NOW = Date.parse('2026-10-14T12:00:00Z')
const pages: Array<{ stopCountdown: () => void }> = []

afterEach(() => {
  pages.splice(0).forEach(page => page.stopCountdown())
})

function matchDetail(matchId: number, overrides: Partial<SupportMatchDetail['viewer']> = {}): SupportMatchDetail {
  return {
    match_id: matchId,
    season: 2026,
    round_number: 20,
    match_date: '2026-10-15',
    match_time: '19:35',
    kickoff_at: '2026-10-15T11:35:00Z',
    status: 'scheduled',
    support_window_status: 'open',
    countdown_seconds: 0,
    total_support_count: 10,
    home_team: { team_id: 1, team_name: '主队', avatar_storage_url: null, score: '-', support_count: 6, support_share_pct: 60, season_support_rank: 1 },
    away_team: { team_id: 2, team_name: '客队', avatar_storage_url: null, score: '-', support_count: 4, support_share_pct: 40, season_support_rank: 2 },
    viewer: { favorite_team_id: null, supported_team_id: null, has_supported: false, can_support: true, ...overrides },
  } as SupportMatchDetail
}

function setup(overrides: Partial<SupportPageDeps> = {}) {
  const session = { revision: ref(0), hasAccessToken: ref(true) }
  let invalidations = 0
  const scope = effectScope()
  const page = scope.run(() => useSupportPage({
    session,
    matchId: ref<number | null>(null),
    now: () => NOW,
    onDetailInvalidated: () => { invalidations += 1 },
    getMatchSupportDetail: () => Promise.resolve(matchDetail(1)),
    castSupportVote: () => Promise.resolve(matchDetail(1, { has_supported: true, supported_team_id: 1 })),
    cancelSupportVote: () => Promise.resolve(matchDetail(1)),
    ...overrides,
  }))!
  page.setMatchId('1')
  pages.push({ stopCountdown: () => { page.stopCountdown(); scope.stop() } })
  return { page, session, invalidations: () => invalidations }
}

describe('useSupportPage 加载', () => {
  test('缺少比赛参数时直接报错，不发请求', async () => {
    let calls = 0
    const { page } = setup({ getMatchSupportDetail: () => { calls += 1; return Promise.resolve(matchDetail(1)) } })
    page.setMatchId(undefined)

    await page.loadPage()

    expect(calls).toBe(0)
    expect(page.errorMessage.value).toBe('缺少比赛参数，无法打开助力页。')
    expect(page.loading.value).toBe(false)
  })

  test('重叠加载时迟到的旧响应不覆盖新响应', async () => {
    const first = deferred<SupportMatchDetail>()
    const second = deferred<SupportMatchDetail>()
    const queue = [first, second]
    const { page } = setup({ getMatchSupportDetail: () => queue.shift()!.promise })

    const firstLoad = page.loadPage()
    const secondLoad = page.loadPage()
    second.resolve(matchDetail(1, { has_supported: true, supported_team_id: 2 }))
    await secondLoad
    first.resolve(matchDetail(1))
    await firstLoad

    expect(page.detail.value?.viewer.supported_team_id).toBe(2)
    expect(page.loading.value).toBe(false)
  })

  test('冷启动登录完成后重新加载，旧错误不覆盖新账号详情', async () => {
    const pending = deferred<SupportMatchDetail>()
    const fresh = deferred<SupportMatchDetail>()
    const queue = [pending, fresh]
    const { page, session } = setup({ getMatchSupportDetail: () => queue.shift()!.promise })

    const load = page.loadPage()
    session.revision.value += 1
    pending.reject(new ApiRequestError('旧账号失败', 500, false))
    await load

    expect(page.detail.value).toBeNull()
    expect(page.errorMessage.value).toBe('')
    expect(page.loading.value).toBe(true)
    fresh.resolve(matchDetail(1, { has_supported: true, supported_team_id: 2 }))
    await fresh.promise
    await Promise.resolve()
    expect(page.detail.value?.viewer.supported_team_id).toBe(2)
    expect(page.loading.value).toBe(false)
  })

  test('页面隐藏后会话变化不刷新，返回时读取新账号详情', async () => {
    let calls = 0
    const { page, session } = setup({ getMatchSupportDetail: () => {
      calls += 1
      return Promise.resolve(matchDetail(1))
    } })
    await page.loadPage()
    page.pausePage()
    session.revision.value += 1
    expect(calls).toBe(1)
    expect(page.detail.value).toBeNull()
    await page.loadPage()
    expect(calls).toBe(2)
    expect(page.detail.value?.match_id).toBe(1)
  })

  test('离开页面后迟到的加载不发布详情或重启倒计时', async () => {
    const pending = deferred<SupportMatchDetail>()
    const { page } = setup({ getMatchSupportDetail: () => pending.promise })
    const load = page.loadPage()
    page.pausePage()
    pending.resolve(matchDetail(1))
    await load
    expect(page.detail.value).toBeNull()
    expect(page.liveCountdownSeconds.value).toBe(0)
    expect(page.loading.value).toBe(false)
  })

  test('加载会让海报失效', async () => {
    const { page, invalidations } = setup()
    await page.loadPage()
    expect(invalidations()).toBe(1)
  })
})

describe('useSupportPage 助力', () => {
  test('未登录时返回 login-required，不发请求', async () => {
    let calls = 0
    const { page, session } = setup({ castSupportVote: () => { calls += 1; return Promise.resolve(matchDetail(1)) } })
    await page.loadPage()
    session.hasAccessToken.value = false

    expect(await page.submitVote(1)).toEqual({ kind: 'login-required', matchId: 1 })
    expect(calls).toBe(0)
  })

  test('点击已助力球队走取消接口', async () => {
    let canceled = 0
    const { page } = setup({
      getMatchSupportDetail: () => Promise.resolve(matchDetail(1, { has_supported: true, supported_team_id: 1 })),
      cancelSupportVote: () => { canceled += 1; return Promise.resolve(matchDetail(1)) },
    })
    await page.loadPage()

    expect(await page.submitVote(1)).toEqual({ kind: 'canceled' })
    expect(canceled).toBe(1)
    expect(page.detail.value?.viewer.has_supported).toBe(false)
  })

  test('助力途中页面重新加载，旧助力结果被丢弃且提交态复位', async () => {
    const vote = deferred<SupportMatchDetail>()
    const { page } = setup({ castSupportVote: () => vote.promise })
    await page.loadPage()

    const submit = page.submitVote(1)
    expect(page.voteSubmittingTeamId.value).toBe(1)
    await page.loadPage()
    vote.resolve(matchDetail(1, { has_supported: true, supported_team_id: 1 }))

    expect(await submit).toEqual({ kind: 'stale' })
    expect(page.detail.value?.viewer.has_supported).toBe(false)
    expect(page.voteSubmittingTeamId.value).toBeNull()
  })

  test('助力失败返回可展示的错误文案', async () => {
    const { page } = setup({ castSupportVote: () => Promise.reject(new ApiRequestError('本场已截止', 409, false)) })
    await page.loadPage()

    expect(await page.submitVote(2)).toEqual({ kind: 'failed', message: '本场已截止' })
    expect(page.voteSubmittingTeamId.value).toBeNull()
  })
})

describe('useSupportPage 倒计时', () => {
  test('开球时间已过则同步关闭助力窗口', async () => {
    const { page } = setup({
      now: () => Date.parse('2026-10-15T12:00:00Z'),
    })
    await page.loadPage()
    page.stopCountdown()

    expect(page.liveCountdownSeconds.value).toBe(0)
    expect(page.detail.value?.support_window_status).toBe('closed')
    expect(page.detail.value?.viewer.can_support).toBe(false)
  })
})
