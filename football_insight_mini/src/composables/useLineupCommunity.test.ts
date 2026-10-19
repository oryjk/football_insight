import { describe, expect, test } from 'bun:test'
import { ApiRequestError } from '../utils/apiError'
import { POST_LOGIN_REDIRECT_STORAGE_KEY, type PostLoginRedirectTarget } from '../utils/authStorage'
import type { PublicLineup, PublicLineupSummary, TeamLineups } from '../types/lineup'
import { createLineupList, createLineupDetail, createLineupLikes, communityDeps, publicRead } from './useLineupCommunity'

const item: PublicLineupSummary = { share_code: 'Ab12Cd34', formation: '4-3-3', author: { display_name: '球迷', avatar_url: null }, created_at: '2026-10-19T00:00:00Z', updated_at: '2026-10-19T00:00:00Z', likes_count: 2, liked_by_me: false }
function response(id: number, page = 1): TeamLineups { return { team: { team_id: id, team_name: String(id), avatar_url: null }, items: [{ ...item, share_code: page === 1 ? 'Ab12Cd34' : 'Ef56Gh78' }], total: 3, page, page_size: 20 } }
function deferred<T>() { let resolve!: (value: T) => void; let reject!: (error: unknown) => void; const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no }); return { promise, resolve, reject } }
function deps() { return communityDeps({ revision: () => 0, loggedIn: () => true, invalidate: () => {}, login: () => {} }) }

describe('public list ownership', () => {
  test('late team/sort response cannot overwrite the latest choice', async () => {
    const first = deferred<TeamLineups>(), second = deferred<TeamLineups>()
    const list = createLineupList({ ...deps(), list: (_id, sort) => sort === 'latest' ? first.promise : second.promise })
    const old = list.load(1, 'latest'), current = list.load(2, 'likes')
    second.resolve(response(2)); await current
    first.resolve(response(1)); await old
    expect(list.team.value?.team_id).toBe(2)
    expect(list.sort.value).toBe('likes')
    expect(list.loading.value).toBe(false)
  })
  test('pagination is single-flight, retains existing rows on error, and retries the same page', async () => {
    const next = deferred<TeamLineups>(); const pages: number[] = []; let fail = true
    const list = createLineupList({ ...deps(), list: async (id, _sort, page = 1) => { pages.push(page); if (page === 1) return response(id); if (fail) return next.promise; return response(id, page) } })
    await list.load(1)
    const pending = list.more(); await list.more()
    next.reject(new Error('offline')); await pending
    expect(pages).toEqual([1, 2]); expect(list.page.value).toBe(1); expect(list.items.value.length).toBe(1)
    fail = false; await list.more()
    expect(pages).toEqual([1, 2, 2]); expect(list.items.value.length).toBe(2)
  })
  test('successful likes reload server ranking, retain the current team/sort, and clear a prior failure', async () => {
    let fail = true, lists = 0
    const second = { ...item, share_code: 'Ef56Gh78', likes_count: 1 }
    const requests: Array<[number, string]> = []
    const list = createLineupList({ ...deps(),
      list: async (id, sort) => { lists++; requests.push([id, sort]); return { ...response(id), items: lists === 1 ? [item, second] : [{ ...second, likes_count: 3, liked_by_me: true }, item] } },
      like: async () => { if (fail) throw new Error('点赞失败'); return { likes_count: 3, liked_by_me: true } },
    })
    await list.load(2, 'likes'); await list.like(second, '/return')
    expect(list.error.value).toBe('点赞失败'); expect(lists).toBe(1)
    fail = false; await list.like(second, '/return')
    expect(requests).toEqual([[2, 'likes'], [2, 'likes']]); expect(list.error.value).toBe('')
    expect(list.items.value[0].share_code).toBe(second.share_code); expect(list.teamId.value).toBe(2)
  })
  test('a stale like success cannot refresh a replacement team', async () => {
    const gate = deferred<{ likes_count: number; liked_by_me: boolean }>(); const ids: number[] = []
    const list = createLineupList({ ...deps(), list: async id => { ids.push(id); return response(id) }, like: () => gate.promise })
    await list.load(1, 'likes'); const pending = list.like(list.items.value[0], '/return')
    await list.load(2, 'latest'); gate.resolve({ likes_count: 3, liked_by_me: true }); await pending
    expect(ids).toEqual([1, 2]); expect(list.teamId.value).toBe(2); expect(list.sort.value).toBe('latest')
  })
  test('unload invalidates the pending page', async () => {
    const next = deferred<TeamLineups>(), list = createLineupList({ ...deps(), list: () => next.promise })
    const pending = list.load(1); list.dispose(); next.resolve(response(1)); await pending
    expect(list.items.value).toEqual([])
  })
  test('pausing cancels a pending page, preserves cached rows, and allows pagination after resume', async () => {
    const gate = deferred<TeamLineups>(); let calls = 0
    const list = createLineupList({ ...deps(), list: async (id, _sort, page = 1) => { calls++; return calls === 2 ? gate.promise : response(id, page) } })
    await list.load(1)
    const pending = list.more(); list.invalidatePending()
    expect(list.loading.value).toBe(false); expect(list.page.value).toBe(1); expect(list.items.value.length).toBe(1)
    gate.resolve({ ...response(1, 2), total: 50 }); await pending
    expect(list.page.value).toBe(1); expect(list.total.value).toBe(3); expect(list.items.value.length).toBe(1)
    await list.more()
    expect(list.page.value).toBe(2); expect(list.items.value.length).toBe(2)
  })
})

describe('login return navigation', () => {
  for (const [url, type] of [
    ['/pages/insights/index', 'switchTab'],
    ['/pages/lineup/list?team_id=1', 'navigateTo'],
    ['/pages/lineup/view?code=Ab12Cd34', 'navigateTo'],
  ] as const) {
    test(`returns to ${url} with ${type}`, () => {
      const globals = globalThis as unknown as { uni?: unknown }, previous = globals.uni
      const stored: Array<[string, unknown]> = [], navigated: string[] = []
      globals.uni = {
        setStorageSync: (key: string, value: unknown) => stored.push([key, value]),
        showToast: () => {},
        switchTab: ({ url }: { url: string }) => navigated.push(url),
      }
      try {
        communityDeps({ revision: () => 0 }).login(url)
        expect(stored).toEqual([[POST_LOGIN_REDIRECT_STORAGE_KEY, { type, url } satisfies PostLoginRedirectTarget]])
        expect(navigated).toEqual(['/pages/user/index'])
      } finally { globals.uni = previous }
    })
  }
})

describe('likes and active auth', () => {
  test('dislike replaces a like and a second dislike cancels it using explicit intent', async () => {
    const intents: boolean[] = []
    const state = { ...item, liked_by_me: true, disliked_by_me: false }
    const flow = createLineupLikes({ ...deps(), dislike: async (_code, enabled) => {
      intents.push(enabled)
      return { likes_count: 1, liked_by_me: false, disliked_by_me: enabled }
    } })
    await flow.apply(state, '/return', () => true, result => Object.assign(state, result), 'dislike')
    expect(state.liked_by_me).toBe(false); expect(state.disliked_by_me).toBe(true); expect(state.likes_count).toBe(1)
    await flow.apply(state, '/return', () => true, result => Object.assign(state, result), 'dislike')
    expect(state.disliked_by_me).toBe(false); expect(intents).toEqual([true, false])
  })
  test('like and dislike share one pending write; failures preserve both flags and count', async () => {
    const gate = deferred<{ likes_count: number; liked_by_me: boolean; disliked_by_me: boolean }>()
    let positive = 0, publishes = 0
    const flow = createLineupLikes({ ...deps(), like: async () => { positive++; return item }, dislike: () => gate.promise })
    const pending = flow.apply(item, '/return', () => true, () => publishes++, 'dislike')
    await flow.apply(item, '/return', () => true, () => publishes++)
    expect(positive).toBe(0)
    gate.reject(new Error('offline')); expect(await pending).toBe('offline')
    expect(publishes).toBe(0); expect(flow.pending.value).toEqual([])
  })
  test('an old account dislike response cannot publish into the new identity', async () => {
    const gate = deferred<{ likes_count: number; liked_by_me: boolean; disliked_by_me: boolean }>()
    let revision = 1, publishes = 0
    const flow = createLineupLikes({ ...deps(), revision: () => revision, dislike: () => gate.promise })
    const pending = flow.apply(item, '/return', () => true, () => publishes++, 'dislike')
    revision++
    gate.resolve({ likes_count: 2, liked_by_me: false, disliked_by_me: true }); await pending
    expect(publishes).toBe(0)
  })
  test('a dislike updates the detail and a guest dislike requests login without writing', async () => {
    let writes = 0, loggedIn = true, logins = 0
    const detail: PublicLineup = { ...item, team: response(1).team, players: [], slots: [] }
    const page = createLineupDetail({ ...deps(), detail: async () => detail, loggedIn: () => loggedIn, login: () => logins++,
      dislike: async () => { writes++; return { likes_count: 2, liked_by_me: false, disliked_by_me: true } },
    })
    await page.load(item.share_code); await page.dislike('/return')
    expect(page.detail.value?.disliked_by_me).toBe(true)
    loggedIn = false; await page.dislike('/return')
    expect(writes).toBe(1); expect(logins).toBe(1)
    page.dispose()
  })
  test('duplicate taps issue one intent; failures leave prior count and liked state intact', async () => {
    const pending = deferred<{ likes_count: number; liked_by_me: boolean }>(); let calls = 0
    const flow = createLineupLikes({ ...deps(), like: () => { calls++; return pending.promise } })
    let published = { likes_count: item.likes_count, liked_by_me: item.liked_by_me }
    const first = flow.apply(item, '/return', () => true, value => { published = value })
    await flow.apply(item, '/return', () => true, value => { published = value })
    expect(calls).toBe(1); pending.reject(new Error('failed')); expect(await first).toBe('failed')
    expect(published).toEqual({ likes_count: 2, liked_by_me: false }); expect(flow.pending.value).toEqual([])
  })
  test('cancel uses explicit false and a stale response cannot update another view', async () => {
    const gate = deferred<{ likes_count: number; liked_by_me: boolean }>(); const intents: boolean[] = []; let publishes = 0
    const flow = createLineupLikes({ ...deps(), like: (_code, liked) => { intents.push(liked); return gate.promise } })
    const pending = flow.apply({ ...item, liked_by_me: true }, '/return', () => true, () => publishes++)
    flow.reset(); gate.resolve({ likes_count: 1, liked_by_me: false }); await pending
    expect(intents).toEqual([false]); expect(publishes).toBe(0)
  })
  test('guest and active-auth rejection both preserve a concrete login return URL', async () => {
    let loggedIn = false, revision = 4; const logins: string[] = []
    const source = { ...deps(), loggedIn: () => loggedIn, revision: () => revision, invalidate: (expected: number) => { if (expected === revision) { revision++; loggedIn = false } }, login: (url: string) => logins.push(url), like: async () => { throw new ApiRequestError('disabled', 401) } }
    const flow = createLineupLikes(source)
    await flow.apply(item, '/view?code=Ab12Cd34', () => true, () => {})
    loggedIn = true
    await flow.apply(item, '/view?code=Ab12Cd34', () => true, () => {})
    expect(logins).toEqual(['/view?code=Ab12Cd34', '/view?code=Ab12Cd34']); expect(loggedIn).toBe(false)
  })
  test('expired optional JWT is cleared once and the public read retries anonymously', async () => {
    let revision = 1, loggedIn = true, calls = 0
    const source = { ...deps(), loggedIn: () => loggedIn, revision: () => revision, invalidate: () => { revision++; loggedIn = false } }
    const result = await publicRead(source, async () => { calls++; if (loggedIn) throw new ApiRequestError('expired', 401); return 42 }, () => true)
    expect(result).toBe(42); expect(calls).toBe(2); expect(revision).toBe(2)
  })
  test('an old 401 retries for the new account and cannot invalidate it', async () => {
    let revision = 1, invalidates = 0, calls = 0; const pending = deferred<number>()
    const reading = publicRead({ ...deps(), revision: () => revision, invalidate: () => invalidates++ }, () => { calls++; return calls === 1 ? pending.promise : Promise.resolve(42) }, () => true)
    revision = 2; pending.reject(new ApiRequestError('old', 401)); expect(await reading).toBe(42); expect(invalidates).toBe(0); expect(calls).toBe(2)
  })
  test('cold detail retries a guest response when bootstrap publishes the signed-in session', async () => {
    let revision = 0, calls = 0
    const gate = deferred<PublicLineup>()
    const guest: PublicLineup = { ...item, team: response(1).team, players: [], slots: [] }
    const page = createLineupDetail({ ...deps(), revision: () => revision, detail: () => { calls++; return calls === 1 ? gate.promise : Promise.resolve({ ...guest, liked_by_me: true }) } })
    const loading = page.load(item.share_code); revision = 1; gate.resolve(guest); await loading
    expect(page.detail.value?.liked_by_me).toBe(true); expect(page.loading.value).toBe(false); expect(page.error.value).toBe(''); expect(calls).toBe(2)
  })
  test('cold list retries a guest page after session restoration instead of showing false empty data', async () => {
    let revision = 0, calls = 0
    const gate = deferred<TeamLineups>()
    const page = createLineupList({ ...deps(), revision: () => revision, list: async id => { calls++; return calls === 1 ? gate.promise : { ...response(id), items: [{ ...item, liked_by_me: true }] } } })
    const loading = page.load(1); revision = 1; gate.resolve(response(1)); await loading
    expect(page.items.value[0].liked_by_me).toBe(true); expect(page.page.value).toBe(1); expect(page.total.value).toBe(3); expect(page.loading.value).toBe(false); expect(calls).toBe(2)
  })
  test('repeated identity churn is bounded and presents a retryable error', async () => {
    let revision = 0, calls = 0
    const page = createLineupList({ ...deps(), revision: () => revision, list: async id => { calls++; revision++; return response(id) } })
    await page.load(1)
    expect(calls).toBe(3); expect(page.error.value).toBe('登录状态已变化，请重试加载'); expect(page.loading.value).toBe(false)
  })
})
