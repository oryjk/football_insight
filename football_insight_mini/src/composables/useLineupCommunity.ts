import { ref } from 'vue'
import { getSharedLineup, getTeamLineups, setLineupLike, setLineupDislike } from '../api/lineup'
import { useAppSession } from '../stores/appSession'
import type { LineupLikes, LineupReaction, LineupSort, PublicLineup, PublicLineupSummary, TeamLineups } from '../types/lineup'
import { extractApiErrorMessage, isUnauthorizedError } from '../utils/apiError'
import { rememberPostLoginRedirect } from '../utils/postLoginRedirect'

export interface CommunityDeps {
  list: typeof getTeamLineups
  detail: typeof getSharedLineup
  like: typeof setLineupLike
  dislike: typeof setLineupDislike
  loggedIn: () => boolean
  revision: () => number
  invalidate: (revision: number) => void
  login: (returnUrl: string) => void
}
export function communityDeps(overrides: Partial<CommunityDeps> = {}): CommunityDeps {
  const session = overrides.revision ? null : useAppSession()
  return {
    list: getTeamLineups, detail: getSharedLineup, like: setLineupLike, dislike: setLineupDislike,
    loggedIn: () => Boolean(session?.hasAccessToken.value), revision: () => session?.revision.value ?? 0,
    invalidate: revision => session?.invalidateSession(revision),
    login: returnUrl => {
      rememberPostLoginRedirect({ type: returnUrl === '/pages/insights/index' ? 'switchTab' : 'navigateTo', url: returnUrl })
      uni.showToast({ title: '请登录后继续', icon: 'none' })
      uni.switchTab({ url: '/pages/user/index' })
    }, ...overrides,
  }
}

/** Retry a public read under the latest identity when startup/login replaces its token.
 * A stale response never publishes personalized flags or invalidates a newer session.
 */
export async function publicRead<T>(deps: CommunityDeps, request: () => Promise<T>, current: () => boolean): Promise<T | null> {
  for (let attempt = 0; attempt < 3; attempt++) {
    if (!current()) return null
    const revision = deps.revision()
    try {
      const result = await request()
      if (!current()) return null
      if (deps.revision() !== revision) continue
      return result
    } catch (error) {
      if (!current()) return null
      if (deps.revision() !== revision) continue
      if (!isUnauthorizedError(error) || !deps.loggedIn()) throw error
      deps.invalidate(revision)
    }
  }
  throw new Error('登录状态已变化，请重试加载')
}

export function createLineupLikes(deps: CommunityDeps) {
  const pending = ref<string[]>([])
  let generation = 0
  function reset(): void { generation++; pending.value = [] }
  async function apply(item: PublicLineupSummary, returnUrl: string, current: () => boolean, publish: (likes: LineupLikes) => void, kind: LineupReaction = 'like'): Promise<string> {
    if (pending.value.includes(item.share_code)) return ''
    if (!deps.loggedIn()) { deps.login(returnUrl); return '' }
    const owner = generation
    const revision = deps.revision()
    const valid = () => current() && generation === owner && deps.revision() === revision
    const code = item.share_code
    pending.value = [...pending.value, code]
    try {
      // Publish server-confirmed state; a failed write keeps the previous count and intent.
      const result = kind === 'like'
        ? await deps.like(code, !item.liked_by_me)
        : await deps.dislike(code, !item.disliked_by_me)
      if (valid()) publish({ ...result, disliked_by_me: result.disliked_by_me ?? false })
      return ''
    } catch (error) {
      if (!valid()) return ''
      if (isUnauthorizedError(error)) { deps.invalidate(revision); deps.login(returnUrl); return '' }
      return extractApiErrorMessage(error, kind === 'like' ? '点赞失败，请重试' : '点踩失败，请重试')
    } finally {
      if (owner === generation) pending.value = pending.value.filter(value => value !== code)
    }
  }
  return { pending, apply, reset }
}

export function createLineupList(overrides: Partial<CommunityDeps> = {}) {
  const deps = communityDeps(overrides)
  const teamId = ref(0), sort = ref<LineupSort>('latest'), page = ref(0), total = ref(0)
  const team = ref<TeamLineups['team'] | null>(null), items = ref<PublicLineupSummary[]>([])
  const loading = ref(false), error = ref('')
  const likes = createLineupLikes(deps)
  let generation = 0, disposed = false
  async function fetchPage(next: number): Promise<void> {
    if (disposed || loading.value || !teamId.value) return
    const owner = generation, id = teamId.value, order = sort.value
    const current = () => !disposed && generation === owner
    loading.value = true; error.value = ''
    try {
      const result = await publicRead(deps, () => deps.list(id, order, next, 20), current)
      if (!result || !current()) return
      team.value = result.team; total.value = result.total; page.value = result.page
      const codes = new Set(items.value.map(item => item.share_code))
      items.value = next === 1 ? result.items : [...items.value, ...result.items.filter(item => !codes.has(item.share_code))]
    } catch (cause) { if (current()) error.value = extractApiErrorMessage(cause, '球队战术板加载失败') }
    finally { if (current()) loading.value = false }
  }
  function load(id: number, order: LineupSort = sort.value): Promise<void> {
    generation++; disposed = false; likes.reset(); loading.value = false
    teamId.value = id; sort.value = order; page.value = 0; total.value = 0; items.value = []; team.value = null
    return fetchPage(1)
  }
  function more(): Promise<void> {
    return page.value === 0 || items.value.length < total.value ? fetchPage(page.value + 1) : Promise.resolve()
  }
  async function like(item: PublicLineupSummary, returnUrl: string, kind: LineupReaction = 'like'): Promise<void> {
    const owner = generation
    if (likes.pending.value.includes(item.share_code)) return
    error.value = ''
    let updated = false
    const message = await likes.apply(item, returnUrl, () => !disposed && generation === owner, result => {
      const target = items.value.find(entry => entry.share_code === item.share_code)
      if (target) { Object.assign(target, result); updated = true }
    }, kind)
    if (owner !== generation || disposed) return
    if (message) error.value = message
    else if (updated && sort.value === 'likes') await load(teamId.value, 'likes')
  }
  /** Pause an embedded list without discarding its last successfully loaded page. */
  function invalidatePending(): void { generation++; likes.reset(); loading.value = false }
  function dispose(): void { disposed = true; invalidatePending() }
  const dislike = (item: PublicLineupSummary, returnUrl: string) => like(item, returnUrl, 'dislike')
  return { teamId, sort, page, total, team, items, loading, error, pendingLikes: likes.pending, load, more, like, dislike, invalidatePending, dispose }
}

export function createLineupDetail(overrides: Partial<CommunityDeps> = {}) {
  const deps = communityDeps(overrides)
  const detail = ref<PublicLineup | null>(null), loading = ref(false), error = ref('')
  const likes = createLineupLikes(deps)
  let generation = 0, disposed = false
  async function load(code: string): Promise<void> {
    const owner = ++generation
    disposed = false; likes.reset(); detail.value = null; loading.value = true; error.value = ''
    const current = () => !disposed && generation === owner
    try { const result = await publicRead(deps, () => deps.detail(code), current); if (current()) detail.value = result }
    catch (cause) { if (current()) error.value = extractApiErrorMessage(cause, '阵容不可用，可能尚未排满或已取消公开') }
    finally { if (current()) loading.value = false }
  }
  async function like(returnUrl: string, kind: LineupReaction = 'like'): Promise<void> {
    const target = detail.value, owner = generation
    if (!target || likes.pending.value.includes(target.share_code)) return
    error.value = ''
    const message = await likes.apply(target, returnUrl, () => !disposed && generation === owner, result => Object.assign(target, result), kind)
    if (message && owner === generation) error.value = message
  }
  function dispose(): void { disposed = true; generation++; likes.reset() }
  const dislike = (returnUrl: string) => like(returnUrl, 'dislike')
  return { detail, loading, error, pendingLikes: likes.pending, load, like, dislike, dispose }
}
