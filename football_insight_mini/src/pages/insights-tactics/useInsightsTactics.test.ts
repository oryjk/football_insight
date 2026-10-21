import { describe, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { CurrentUser } from '../../types/auth'
import type { LineupEditorResponse, LineupSort, PublicLineupSummary, TeamLineups } from '../../types/lineup'
import type { SessionStatus } from '../../stores/appSession'
import { ApiRequestError } from '../../utils/apiError'
import { useInsightsTactics, type InsightsTacticsOptions } from './useInsightsTactics'

const card: PublicLineupSummary = {
  share_code: 'Ab12Cd34', formation: '4-3-3', author: { display_name: '球迷', avatar_url: null },
  created_at: '', updated_at: '', likes_count: 2, liked_by_me: true,
}
const saved: LineupEditorResponse = {
  draft: { formation: '4-4-2', slots: [], updated_at: 'draft' },
  lineup: { formation: '4-3-3', slots: [], share_code: card.share_code, updated_at: 'published' },
}
function user(id: string): CurrentUser {
  return { id, display_name: id, account_identifier: id, avatar_url: null, has_wechat_binding: false,
    membership_tier: 'none', membership_expires_at: null, created_at: '' }
}
function session(authenticated = true) {
  const currentUser = ref<CurrentUser | null>(authenticated ? user('A') : null)
  const hasAccessToken = ref(authenticated), status = ref<SessionStatus>(authenticated ? 'authenticated' : 'guest')
  const revision = ref(0), invalidations: number[] = []
  return { currentUser, hasAccessToken, status, revision, invalidations,
    invalidateSession(expected: number) {
      invalidations.push(expected)
      if (revision.value !== expected) return
      revision.value++; hasAccessToken.value = false; currentUser.value = null; status.value = 'guest'
    },
  }
}
function response(id: number, page = 1): TeamLineups {
  return { team: { team_id: id, team_name: `${id}`, avatar_url: null },
    items: [{ ...card, share_code: `${id}-${page}` }], total: 2, page, page_size: 20 }
}
function deferred<T>() {
  let resolve!: (value: T) => void, reject!: (cause: unknown) => void
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}
async function flush() { await nextTick(); await Promise.resolve(); await Promise.resolve() }
function setup(overrides: Partial<InsightsTacticsOptions> = {}) {
  const auth = session(), selectedTeamId = ref<number | null>(1), active = ref(true)
  const lists: Array<[number, LineupSort, number]> = [], mines: number[] = [], logins: string[] = []
  const api = useInsightsTactics({ session: auth, selectedTeamId, active, ...overrides,
    deps: {
      mine: async id => { mines.push(id); return saved },
      ...overrides.deps,
      community: {
        list: async (id, sort, page = 1) => { lists.push([id, sort, page]); return response(id, page) },
        like: async () => ({ likes_count: 3, liked_by_me: false }),
        login: url => logins.push(url),
        ...overrides.deps?.community,
      },
    },
  })
  return { api, auth, selectedTeamId, active, lists, mines, logins }
}

describe('insights tactics loading and identity ownership', () => {
  test('dislike publishes both flags and logout removes the personalized dislike', async () => {
    let disliked = false
    const { api, auth } = setup({ deps: { community: {
      list: async id => ({ ...response(id), items: [{ ...card, liked_by_me: !disliked, disliked_by_me: disliked }] }),
      dislike: async (_code, enabled) => { disliked = enabled; return { likes_count: 1, liked_by_me: false, disliked_by_me: enabled } },
    } } })
    await flush(); await api.dislike(api.community.items.value[0])
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    expect(api.community.items.value[0].disliked_by_me).toBe(true)
    auth.currentUser.value = null; auth.hasAccessToken.value = false; auth.status.value = 'guest'; auth.revision.value++
    await flush()
    expect(api.community.items.value[0].disliked_by_me).toBe(false)
    api.dispose()
  })
  test('waits for an active valid team; guests read public tactics without private requests', async () => {
    const selectedTeamId = ref<number | null>(null), active = ref(false)
    const { api, lists, mines } = setup({ session: session(false), selectedTeamId, active })
    selectedTeamId.value = 1; await flush()
    expect(lists).toEqual([])
    active.value = true; await flush()
    expect(lists).toEqual([[1, 'latest', 1]])
    expect(mines).toEqual([])
    expect(api.mineDraft.value).toBeNull()
    api.dispose()
  })

  test('a slow cloud draft never blocks the public list; completed tab content is reused', async () => {
    const pending = deferred<LineupEditorResponse>()
    const { api, active, lists } = setup({ deps: { mine: () => pending.promise } })
    await flush()
    expect(api.mineLoading.value).toBe(true)
    expect(api.community.items.value.length).toBe(1)
    pending.resolve(saved); await flush()
    active.value = false; await flush(); active.value = true; await flush()
    expect(api.mineDraft.value?.formation).toBe('4-4-2')
    expect(api.minePublished.value?.formation).toBe('4-3-3')
    expect(lists.length).toBe(1)
    await api.like(api.community.items.value[0])
    expect(api.community.items.value[0].likes_count).toBe(3)
    api.dispose()
  })

  test('switching teams clears private data and old results/finally cannot settle the new request', async () => {
    const old = deferred<LineupEditorResponse>(), current = deferred<LineupEditorResponse>()
    const { api, selectedTeamId } = setup({ deps: { mine: id => id === 1 ? old.promise : current.promise } })
    await flush()
    selectedTeamId.value = 2; await flush()
    old.resolve(saved); await flush()
    expect(api.mineDraft.value).toBeNull()
    expect(api.mineLoading.value).toBe(true)
    expect(api.community.team.value?.team_id).toBe(2)
    current.resolve({ ...saved, draft: { ...saved.draft!, formation: '3-5-2' } }); await flush()
    expect(api.mineDraft.value?.formation).toBe('3-5-2')
    expect(api.mineLoading.value).toBe(false)
    api.dispose()
  })

  test('authentication loading hides the previous draft and liked flags without reading its old token', async () => {
    const auth = session()
    let likes = 0
    const { api, mines } = setup({ session: auth, deps: { community: {
      like: async () => { likes++; return { likes_count: 3, liked_by_me: true } },
    } } })
    await flush()
    expect(Boolean(api.mineDraft.value)).toBe(true)
    auth.status.value = 'loading'; auth.revision.value++; await flush()
    expect(api.mineDraft.value).toBeNull()
    expect(api.minePublished.value).toBeNull()
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    expect(mines).toEqual([1])
    await api.like(api.community.items.value[0])
    expect(likes).toBe(0)
    auth.currentUser.value = user('B'); auth.status.value = 'authenticated'; auth.revision.value++; await flush()
    expect(mines).toEqual([1, 1])
    expect(Boolean(api.mineDraft.value)).toBe(true)
    api.dispose()
  })

  test('a stale account 401 cannot invalidate a replacement account or clear its loading state', async () => {
    const auth = session(), old = deferred<LineupEditorResponse>(), current = deferred<LineupEditorResponse>()
    let count = 0
    const { api } = setup({ session: auth, deps: { mine: () => ++count === 1 ? old.promise : current.promise } })
    await flush()
    auth.currentUser.value = user('B'); auth.revision.value++; await flush()
    old.reject(new ApiRequestError('old account', 401)); await flush()
    expect(auth.invalidations).toEqual([])
    expect(api.mineLoading.value).toBe(true)
    current.resolve(saved); await flush()
    expect(api.mineLoading.value).toBe(false)
    api.dispose()
  })

  test('public 401 during authentication cannot invalidate the active login intent', async () => {
    const auth = session()
    auth.status.value = 'loading'
    const { api, mines } = setup({ session: auth, deps: { community: {
      list: async () => { throw new ApiRequestError('previous token', 401) },
    } } })
    await flush()
    expect(auth.invalidations).toEqual([])
    expect(auth.status.value).toBe('loading')
    expect(mines).toEqual([])
    api.dispose()
  })

  test('a failed session refresh retains the resolved identity, cloud card, and personalized likes', async () => {
    const auth = session()
    const { api, mines, lists } = setup({ session: auth })
    await flush()
    auth.status.value = 'error'; await flush()
    expect(api.authenticated.value).toBe(true)
    expect(api.mineDraft.value?.updated_at).toBe('draft')
    expect(api.minePublished.value?.updated_at).toBe('published')
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    expect(mines).toEqual([1])
    expect(lists.length).toBe(1)
    await api.refresh()
    expect(mines).toEqual([1, 1])
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    await api.like(api.community.items.value[0])
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    auth.status.value = 'loading'; await flush()
    expect(api.authenticated.value).toBe(false)
    expect(api.mineDraft.value).toBeNull()
    api.dispose()
  })

  test('identity changes replace an in-flight guest page and ignore its late response', async () => {
    const auth = session(false), old = deferred<TeamLineups>(); let count = 0
    const { api, mines } = setup({ session: auth, deps: { community: {
      list: async id => ++count === 1 ? old.promise : response(id),
    } } })
    auth.currentUser.value = user('A'); auth.hasAccessToken.value = true
    auth.status.value = 'authenticated'; auth.revision.value++; await flush()
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    expect(mines).toEqual([1])
    old.resolve({ ...response(1), items: [] }); await flush()
    expect(api.community.items.value.length).toBe(1)
    expect(api.community.loading.value).toBe(false)
    api.dispose()
  })

  test('failed cloud reads can retry and onShow refresh updates both independent resources for a free account', async () => {
    let attempts = 0
    const { api, lists } = setup({ deps: { mine: async () => {
      if (++attempts === 1) throw new Error('offline')
      return { ...saved, draft: { ...saved.draft!, updated_at: `${attempts}` } }
    } } })
    await flush()
    expect(api.mineError.value).toBe('offline')
    expect(api.community.items.value.length).toBe(1)
    await api.retryMine()
    expect(api.mineError.value).toBe('')
    expect(api.mineDraft.value?.updated_at).toBe('2')
    await api.refresh()
    expect(api.mineDraft.value?.updated_at).toBe('3')
    expect(lists.length).toBe(2)
    api.dispose()
  })

  test('sort, paging, and guest like keep the insights return URL', async () => {
    const { api, lists, logins } = setup({ session: session(false) })
    await flush(); await api.changeSort('likes'); await api.more()
    expect(lists).toEqual([[1, 'latest', 1], [1, 'likes', 1], [1, 'likes', 2]])
    await api.like(api.community.items.value[0])
    expect(logins).toEqual(['/pages/insights/index'])
    api.dispose()
  })

  test('onShow refresh supersedes a cloud read that started before editing', async () => {
    const old = deferred<LineupEditorResponse>(), current = deferred<LineupEditorResponse>()
    let count = 0
    const { api, lists } = setup({ deps: { mine: () => ++count === 1 ? old.promise : current.promise } })
    await flush()
    const refresh = api.refresh(); await flush()
    old.resolve(saved); await flush()
    expect(api.mineDraft.value).toBeNull()
    expect(api.mineLoading.value).toBe(true)
    current.resolve({ ...saved, draft: { ...saved.draft!, updated_at: 'after-editing' } })
    await refresh
    expect(api.mineDraft.value?.updated_at).toBe('after-editing')
    expect(lists.length).toBe(2)
    api.dispose()
  })

  test('inactive tabs cancel pending reads and resume them once; disposal blocks late data', async () => {
    const first = deferred<LineupEditorResponse>(), second = deferred<LineupEditorResponse>()
    let count = 0
    const { api, active } = setup({ deps: { mine: () => ++count === 1 ? first.promise : second.promise } })
    await flush()
    active.value = false; await flush()
    first.resolve(saved); await flush()
    expect(api.mineDraft.value).toBeNull()
    expect(api.mineLoading.value).toBe(false)
    active.value = true; await flush()
    expect(count).toBe(2)
    api.dispose(); second.resolve(saved); await flush()
    expect(api.mineDraft.value).toBeNull()
    expect(api.mineLoading.value).toBe(false)
  })

  test('a like completed while inactive invalidates the cached public page for the next visit', async () => {
    const write = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let serverLiked = false, reads = 0
    const { api, active } = setup({ deps: { community: {
      list: async id => { reads++; return { ...response(id), items: [{ ...card, liked_by_me: serverLiked, likes_count: serverLiked ? 3 : 2 }] } },
      like: () => write.promise,
    } } })
    await flush()
    const like = api.like(api.community.items.value[0])
    active.value = false; await flush()
    expect(api.community.page.value).toBe(0)
    serverLiked = true; write.resolve({ likes_count: 3, liked_by_me: true }); await like
    expect(reads).toBe(1)
    active.value = true; await flush()
    expect(reads).toBe(2)
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    expect(api.community.items.value[0].likes_count).toBe(3)
    api.dispose()
  })

  test('returning before an interrupted like finishes rechecks its final server state', async () => {
    const write = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let serverLiked = false, reads = 0
    const { api, active } = setup({ deps: { community: {
      list: async id => { reads++; return { ...response(id), items: [{ ...card, liked_by_me: serverLiked, likes_count: serverLiked ? 3 : 2 }] } },
      like: () => write.promise,
    } } })
    await flush()
    const like = api.like(api.community.items.value[0])
    active.value = false; await flush(); active.value = true; await flush()
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    expect(reads).toBe(2)
    serverLiked = true; write.resolve({ likes_count: 3, liked_by_me: true }); await like
    expect(reads).toBe(3)
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    expect(api.community.items.value[0].likes_count).toBe(3)
    api.dispose()
  })

  test('returning to a team before its interrupted like finishes refreshes that team after success', async () => {
    const write = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let serverLiked = false
    const reads: number[] = []
    const { api, selectedTeamId } = setup({ deps: { community: {
      list: async id => {
        reads.push(id)
        const liked = id === 1 && serverLiked
        return { ...response(id), items: [{ ...card, share_code: `${id}-1`, liked_by_me: liked, likes_count: liked ? 3 : 2 }] }
      },
      like: () => write.promise,
    } } })
    await flush()
    const like = api.like(api.community.items.value[0])
    selectedTeamId.value = 2; await flush()
    selectedTeamId.value = 1; await flush()
    expect(reads).toEqual([1, 2, 1])
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    serverLiked = true; write.resolve({ likes_count: 3, liked_by_me: true }); await like
    expect(reads).toEqual([1, 2, 1, 1])
    expect(api.community.team.value?.team_id).toBe(1)
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    expect(api.community.items.value[0].likes_count).toBe(3)
    api.dispose()
  })

  test('refresh replacing a pending like rechecks the server after that write finishes', async () => {
    const write = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let serverLiked = false, reads = 0
    const { api } = setup({ deps: { community: {
      list: async id => { reads++; return { ...response(id), items: [{ ...card, liked_by_me: serverLiked }] } },
      like: () => write.promise,
    } } })
    await flush()
    const like = api.like(api.community.items.value[0])
    await api.refresh()
    expect(api.community.items.value[0].liked_by_me).toBe(false)
    serverLiked = true; write.resolve({ likes_count: 3, liked_by_me: true }); await like
    expect(reads).toBe(3)
    expect(api.community.items.value[0].liked_by_me).toBe(true)
    api.dispose()
  })

  test('refreshing an old like result also tracks a newer pending write replaced by that refresh', async () => {
    const first = deferred<{ likes_count: number; liked_by_me: boolean }>()
    const second = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let firstLiked = false, secondLiked = false
    const { api } = setup({ deps: { community: {
      list: async id => ({ ...response(id), items: [
        { ...card, share_code: 'first', liked_by_me: firstLiked },
        { ...card, share_code: 'second', liked_by_me: secondLiked },
      ] }),
      like: code => code === 'first' ? first.promise : second.promise,
    } } })
    await flush()
    const firstLike = api.like(api.community.items.value[0])
    await api.refresh()
    const secondLike = api.like(api.community.items.value[1])
    firstLiked = true; first.resolve({ likes_count: 3, liked_by_me: true }); await firstLike
    expect(api.community.items.value.map(item => item.liked_by_me)).toEqual([true, false])
    secondLiked = true; second.resolve({ likes_count: 3, liked_by_me: true }); await secondLike
    expect(api.community.items.value.map(item => item.liked_by_me)).toEqual([true, true])
    api.dispose()
  })

  test('likes sorting internally replacing another pending write still reconciles its final state', async () => {
    const first = deferred<{ likes_count: number; liked_by_me: boolean }>()
    const second = deferred<{ likes_count: number; liked_by_me: boolean }>()
    let firstLiked = false, secondLiked = false
    const { api } = setup({ deps: { community: {
      list: async id => ({ ...response(id), items: [
        { ...card, share_code: 'first', liked_by_me: firstLiked },
        { ...card, share_code: 'second', liked_by_me: secondLiked },
      ] }),
      like: code => code === 'first' ? first.promise : second.promise,
    } } })
    await flush(); await api.changeSort('likes')
    const firstLike = api.like(api.community.items.value[0])
    const secondLike = api.like(api.community.items.value[1])
    firstLiked = true; first.resolve({ likes_count: 3, liked_by_me: true }); await firstLike
    secondLiked = true; second.resolve({ likes_count: 3, liked_by_me: true }); await secondLike
    expect(api.community.sort.value).toBe('likes')
    expect(api.community.items.value.map(item => item.liked_by_me)).toEqual([true, true])
    api.dispose()
  })
})
