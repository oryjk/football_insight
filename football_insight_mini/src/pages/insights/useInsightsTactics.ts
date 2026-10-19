import { computed, ref, watch, type Ref } from 'vue'
import { getMyLineupEditor, getTeamLineups, setLineupLike, setLineupDislike } from '../../api/lineup'
import { createLineupList, type CommunityDeps } from '../../composables/useLineupCommunity'
import type { AppSession } from '../../stores/appSession'
import type { LineupReaction, LineupSort, PublicLineupSummary, SavedLineup, SavedLineupDraft } from '../../types/lineup'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'

export interface InsightsTacticsOptions {
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'status' | 'revision' | 'invalidateSession'>
  selectedTeamId: Readonly<Ref<number | null>>
  active: Readonly<Ref<boolean>>
  deps?: {
    mine?: typeof getMyLineupEditor
    community?: Partial<CommunityDeps>
  }
}

/** Page-owned state: private cloud drafts and public tactics have independent loading. */
export function useInsightsTactics(options: InsightsTacticsOptions) {
  const { session, selectedTeamId, active } = options
  const mine = options.deps?.mine ?? getMyLineupEditor
  const mineLoading = ref(false), mineError = ref('')
  const mineDraft = ref<SavedLineupDraft | null>(null)
  const minePublished = ref<SavedLineup | null>(null)
  // A refresh failure preserves the resolved user; a login intent can expose an old token.
  const privateUser = () => (session.status.value === 'authenticated' || session.status.value === 'error')
    && session.hasAccessToken.value ? session.currentUser.value?.id ?? '' : ''
  const authenticated = computed(() => Boolean(privateUser()))
  const list = options.deps?.community?.list ?? getTeamLineups
  const writeLike = options.deps?.community?.like ?? setLineupLike
  const writeDislike = options.deps?.community?.dislike ?? setLineupDislike
  let pendingWrites = 0, interruptedLikes = 0
  const community = createLineupList({
    revision: () => session.revision.value,
    // An authentication intent can retain its previous token until the new one arrives.
    // A public 401 from that token must not invalidate the in-flight login intent.
    loggedIn: () => session.hasAccessToken.value && session.status.value !== 'loading',
    invalidate: revision => session.invalidateSession(revision),
    ...options.deps?.community,
    list: async (...args) => {
      // Includes first-page replacements performed internally by likes sorting.
      if (args[2] === 1) beforeListReplace()
      const result = await list(...args)
      return privateUser() ? result : {
        ...result, items: result.items.map(item => ({ ...item, liked_by_me: false, disliked_by_me: false })),
      }
    },
    like: async (...args) => {
      pendingWrites++
      try { return await writeLike(...args) }
      finally { pendingWrites-- }
    },
    dislike: async (...args) => {
      pendingWrites++
      try { return await writeDislike(...args) }
      finally { pendingWrites-- }
    },
  })
  let disposed = false, context = '', generation = 0, mineLoaded = false
  let mineFlight: Promise<void> | null = null

  const teamId = () => {
    const id = selectedTeamId.value
    return id !== null && Number.isInteger(id) && id > 0 ? id : 0
  }
  const identity = () => [session.revision.value, authenticated.value ? 'resolved' : session.status.value,
    session.hasAccessToken.value, session.currentUser.value?.id ?? ''].join(':')
  const contextKey = () => `${teamId()}/${identity()}`
  const enabled = () => !disposed && active.value && Boolean(teamId())

  function cancelMine(): void {
    generation++
    mineFlight = null
    mineLoading.value = false
  }

  function beforeListReplace(): void {
    if (pendingWrites > 0) interruptedLikes++
  }

  function loadPublic(id: number, order?: LineupSort): Promise<void> {
    beforeListReplace()
    return community.load(id, order)
  }

  function reconcileContext(): void {
    const next = contextKey()
    if (next === context) return
    context = next
    cancelMine()
    mineLoaded = false
    mineDraft.value = null
    minePublished.value = null
    mineError.value = ''
    // Also forget liked_by_me and any pending like from the previous identity.
    void loadPublic(0)
  }

  function loadMine(force = false): Promise<void> {
    if (!enabled() || !privateUser() || mineFlight || (!force && mineLoaded)) return mineFlight ?? Promise.resolve()
    const owner = generation, key = contextKey(), id = teamId(), revision = session.revision.value
    const current = () => enabled() && owner === generation && contextKey() === key && Boolean(privateUser())
    mineLoading.value = true
    mineError.value = ''
    const request = (async () => {
      try {
        const result = await Promise.resolve().then(() => current() ? mine(id) : null)
        if (!result || !current()) return
        mineDraft.value = result.draft
        minePublished.value = result.lineup
        mineLoaded = true
      } catch (cause) {
        if (!current()) return
        if (isUnauthorizedError(cause)) session.invalidateSession(revision)
        else mineError.value = extractApiErrorMessage(cause, '我的战术加载失败，请重试')
      } finally {
        if (current()) {
          mineLoading.value = false
          mineFlight = null
        }
      }
    })()
    mineFlight = request
    return request
  }

  function ensureLoaded(): void {
    if (!enabled()) return
    if (!community.loading.value && community.page.value === 0) void loadPublic(teamId())
    void loadMine()
  }

  const stop = watch(() => [teamId(), active.value, identity()] as const, () => {
    reconcileContext()
    if (!enabled()) {
      cancelMine()
      if (pendingWrites > 0) {
        // A canceled response may already have changed the server. Re-read on the next visit.
        void loadPublic(0)
      }
      community.invalidatePending()
      return
    }
    ensureLoaded()
  }, { immediate: true })

  /** onShow refreshes editor changes and detail-page likes without serializing the two reads. */
  async function refresh(): Promise<void> {
    reconcileContext()
    if (!enabled()) return
    cancelMine()
    const publicRead = loadPublic(teamId())
    await Promise.all([publicRead, loadMine(true)])
  }

  function retryMine(): Promise<void> {
    reconcileContext()
    return loadMine(true)
  }

  function changeSort(order: LineupSort): Promise<void> {
    reconcileContext()
    if (!enabled() || (community.sort.value === order && community.page.value > 0 && !community.error.value)) return Promise.resolve()
    return loadPublic(teamId(), order)
  }

  function more(): Promise<void> {
    reconcileContext()
    return enabled() ? community.more() : Promise.resolve()
  }

  function like(item: PublicLineupSummary, kind: LineupReaction = 'like'): Promise<void> {
    reconcileContext()
    const target = community.items.value.find(entry => entry.share_code === item.share_code)
    if (!enabled() || !target || session.status.value === 'loading') return Promise.resolve()
    const key = contextKey(), interruption = interruptedLikes
    return community.like(target, '/pages/insights/index', kind).then(() => {
      // The user may return before the canceled write finishes; refresh its final server state too.
      if (interruption !== interruptedLikes && enabled() && contextKey() === key) return loadPublic(teamId())
    })
  }

  function dispose(): void {
    disposed = true
    stop()
    cancelMine()
    community.dispose()
  }

  const dislike = (item: PublicLineupSummary) => like(item, 'dislike')
  return { authenticated, mineLoading, mineError, mineDraft, minePublished, community, refresh, retryMine, changeSort, more, like, dislike, dispose }
}
