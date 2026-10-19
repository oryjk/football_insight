import { computed, ref, watch } from 'vue'
import { getMyLineupEditor, getSquad, publishMyLineup, saveMyLineupDraft } from '../../api/lineup'
import { useAppSession } from '../../stores/appSession'
import type { LineupEditorResponse, SavedLineup, SavedLineupDraft, SquadResponse } from '../../types/lineup'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { DEFAULT_FORMATION } from './formations'
import { autoFillStartingEleven, emptyLineup, type LineupSlots } from './helpers'
import { clearDraft, consumeGuestDraft, markGuestDraftIntent, readDraft, sameLineup, sanitizeDraftSlots, writeDraft, type DraftStorage } from './lineupDraft'
import { canShareLineup } from './lineupPoster'
import { useLineupEditor } from './useLineupEditor'

interface Payload { formation: string; slots: LineupSlots }
export interface LineupPageDeps {
  getSquad: (teamId: number) => Promise<SquadResponse>
  getMyLineupEditor: (teamId: number) => Promise<LineupEditorResponse>
  saveMyLineupDraft: (teamId: number, payload: Payload) => Promise<{ draft: SavedLineupDraft }>
  publishMyLineup: (teamId: number, payload: Payload) => Promise<{ lineup: SavedLineup; draft: SavedLineupDraft }>
  isLoggedIn: () => boolean
  userId: () => string | null
  storage: DraftStorage
  now: () => number
  draftDelayMs: number
  sessionRevision: () => number
  invalidateSession: (revision: number) => void
}
export type SaveResult = 'saved' | 'need-login' | 'failed' | 'cancelled' | 'incomplete'
export type SaveState = 'idle' | 'dirty' | 'draft' | 'published' | 'error'

/** 编辑草稿、云端私密草稿和已发布快照分别管理。 */
export function useLineupPage(overrides: Partial<LineupPageDeps> = {}) {
  const session = overrides.isLoggedIn ? null : useAppSession()
  const deps: LineupPageDeps = {
    getSquad, getMyLineupEditor, saveMyLineupDraft, publishMyLineup,
    isLoggedIn: () => Boolean(session?.hasAccessToken.value),
    userId: () => session?.currentUser.value?.id ?? null,
    storage: overrides.storage ?? uni,
    now: Date.now, draftDelayMs: 300,
    sessionRevision: () => session?.revision.value ?? 0,
    invalidateSession: revision => session?.invalidateSession(revision),
    ...overrides,
  }
  const editor = useLineupEditor()
  const teamId = ref(0)
  const squad = ref<SquadResponse | null>(null)
  const loading = ref(false)
  const errorMessage = ref('')
  const notice = ref('')
  const savingDraft = ref(false)
  const publishing = ref(false)
  const saving = computed(() => savingDraft.value || publishing.value)
  const failed = ref(false)
  const savedLineup = ref<SavedLineup | null>(null)
  const cloudDraft = ref<SavedLineupDraft | null>(null)
  const content = (): Payload => ({ formation: editor.formation.value, slots: editor.slots.value })
  const hasUnpublishedChanges = computed(() => !sameLineup(content(), savedLineup.value))
  const saveState = computed<SaveState>(() => {
    if (failed.value) return 'error'
    if (editor.dirty.value) return 'dirty'
    if (cloudDraft.value && sameLineup(content(), cloudDraft.value) && hasUnpublishedChanges.value) return 'draft'
    if (savedLineup.value && !hasUnpublishedChanges.value) return 'published'
    return 'idle'
  })
  let loadSeq = 0
  let disposed = false
  let loadedOwner: string | null = null
  let loadedAuthRevision = 0
  let draftTimer: ReturnType<typeof setTimeout> | null = null
  // 未解析身份时禁止使用共享的 authenticated 缓存。
  const owner = () => deps.isLoggedIn() ? (deps.userId() ? `user:${deps.userId()}` : null) : 'guest'
  function cancelDraftTimer(): void {
    if (draftTimer) clearTimeout(draftTimer)
    draftTimer = null
  }
  function persistDraft(): void {
    if (disposed || loading.value || !squad.value || !teamId.value || !loadedOwner) return
    // 本地编辑发生在当前已知远端之后，避免设备时钟略慢导致未同步改动被误判为过期。
    const remoteAt = Date.parse(cloudDraft.value?.updated_at ?? savedLineup.value?.updated_at ?? '')
    const savedAt = Math.max(deps.now(), Number.isFinite(remoteAt) ? remoteAt + 1 : 0)
    writeDraft({ teamId: teamId.value, ...content(), savedAt }, deps.storage, loadedOwner)
  }
  watch([editor.slots, editor.formation], () => {
    if (disposed || loading.value || !squad.value || !editor.dirty.value) return
    cancelDraftTimer()
    draftTimer = setTimeout(persistDraft, deps.draftDelayMs)
  }, { deep: true })

  async function load(nextTeamId: number, options: { restoreGuestDraft?: boolean } = {}): Promise<void> {
    if (editor.dirty.value) persistDraft()
    const seq = ++loadSeq
    let authRevision = deps.sessionRevision()
    const current = () => !disposed && seq === loadSeq && authRevision === deps.sessionRevision()
    disposed = false
    cancelDraftTimer()
    loadedOwner = owner()
    loadedAuthRevision = authRevision
    const ownerAtStart = loadedOwner
    teamId.value = nextTeamId
    loading.value = true
    squad.value = null
    savingDraft.value = false
    publishing.value = false
    editor.load(DEFAULT_FORMATION, emptyLineup(DEFAULT_FORMATION))
    errorMessage.value = ''; notice.value = ''; failed.value = false
    savedLineup.value = null; cloudDraft.value = null
    try {
      const [loadedSquad, mine] = await Promise.all([
        deps.getSquad(nextTeamId),
        deps.isLoggedIn() ? deps.getMyLineupEditor(nextTeamId).catch(error => {
          if (!current() || !isUnauthorizedError(error)) throw error
          deps.invalidateSession(authRevision)
          authRevision = deps.sessionRevision()
          loadedAuthRevision = authRevision
          loadedOwner = owner()
          return { lineup: null, draft: null }
        }) : Promise.resolve({ lineup: null, draft: null }),
      ])
      if (!current()) return
      savedLineup.value = mine.lineup
      cloudDraft.value = mine.draft
      squad.value = loadedSquad
      const known = new Set([...loadedSquad.active, ...loadedSquad.former].map(p => p.player_id))
      let local = loadedOwner ? readDraft(nextTeamId, deps.storage, loadedOwner) : null
      const guest = options.restoreGuestDraft && deps.isLoggedIn() && ownerAtStart && ownerAtStart !== 'guest'
        ? consumeGuestDraft(nextTeamId, deps.storage) : null
      if (guest) {
        // 显式操作优先于云端，立即以新账号持有，之后登录其他账号不能再导入。
        local = { ...guest, savedAt: deps.now() }
        writeDraft(local, deps.storage, ownerAtStart!)
      }
      const remote = mine.draft ?? mine.lineup
      const remoteAt = remote ? Date.parse(remote.updated_at) : 0
      const useLocal = local && (Boolean(guest) || local.savedAt > (Number.isFinite(remoteAt) ? remoteAt : 0))
      if (local && !useLocal && loadedOwner) clearDraft(nextTeamId, deps.storage, loadedOwner)
      const initial = useLocal ? local : remote
      if (initial) {
        const clean = sanitizeDraftSlots(initial.slots, known)
        const removed = !sameLineup(initial, { formation: initial.formation, slots: clean })
        editor.load(initial.formation, clean, Boolean(useLocal) || removed)
        if (removed) notice.value = '阵容里有球员已不在球队名单，已从草稿中移出，请重新选择。'
      } else {
        editor.load(DEFAULT_FORMATION, autoFillStartingEleven(emptyLineup(DEFAULT_FORMATION), DEFAULT_FORMATION, loadedSquad.active))
      }
    } catch (error) {
      if (current()) errorMessage.value = extractApiErrorMessage(error, '战术板加载失败')
    } finally {
      if (!disposed && seq === loadSeq) loading.value = false
    }
  }

  async function submit(kind: 'draft' | 'publish'): Promise<SaveResult> {
    if (disposed || loading.value || !squad.value || saving.value) return 'cancelled'
    if (loadedAuthRevision !== deps.sessionRevision() || loadedOwner !== owner()) return 'cancelled'
    if (kind === 'publish' && !canShareLineup(editor.slots.value)) return 'incomplete'
    if (!deps.isLoggedIn()) {
      cancelDraftTimer(); persistDraft()
      markGuestDraftIntent(teamId.value, deps.storage)
      return 'need-login'
    }
    if (kind === 'draft' && !editor.dirty.value && sameLineup(content(), cloudDraft.value) && !failed.value) return 'saved'
    cancelDraftTimer()
    const lock = kind === 'draft' ? savingDraft : publishing
    lock.value = true
    const teamAtStart = teamId.value, contextAtStart = loadSeq, revisionAtStart = editor.revision.value
    const authRevision = deps.sessionRevision()
    const current = () => !disposed && loadSeq === contextAtStart && teamId.value === teamAtStart && deps.sessionRevision() === authRevision && owner() === loadedOwner
    const payload = { formation: editor.formation.value, slots: editor.slots.value.map(s => ({ ...s })) }
    try {
      let draft: SavedLineupDraft
      let publication: SavedLineup | undefined
      if (kind === 'draft') draft = (await deps.saveMyLineupDraft(teamAtStart, payload)).draft
      else {
        const response = await deps.publishMyLineup(teamAtStart, payload)
        draft = response.draft
        publication = response.lineup
      }
      if (!current()) return 'cancelled'
      cloudDraft.value = draft
      if (publication) savedLineup.value = publication
      failed.value = false
      if (editor.revision.value === revisionAtStart) {
        editor.markSaved(); cancelDraftTimer()
        if (loadedOwner) clearDraft(teamAtStart, deps.storage, loadedOwner)
      } else persistDraft()
      return 'saved'
    } catch (error) {
      if (!current()) return 'cancelled'
      persistDraft()
      if (isUnauthorizedError(error)) {
        deps.invalidateSession(authRevision)
        return 'need-login'
      }
      failed.value = true
      return 'failed'
    } finally {
      if (!disposed && loadSeq === contextAtStart && teamId.value === teamAtStart) lock.value = false
    }
  }
  const saveDraft = () => submit('draft')
  const publish = () => submit('publish')
  function fillStartingEleven(): void {
    if (!squad.value) return
    const filled = autoFillStartingEleven(editor.slots.value, editor.formation.value, squad.value.active)
    editor.slots.value.forEach((slot, index) => {
      if (slot.player_id === null && filled[index].player_id !== null) editor.pick(slot.slot, filled[index].player_id)
    })
  }
  function dispose(): void {
    if (!disposed && editor.dirty.value) persistDraft()
    disposed = true; loadSeq += 1; cancelDraftTimer()
  }
  return { teamId, squad, loading, errorMessage, notice, saving, savingDraft, publishing, saveState, savedLineup, cloudDraft, hasUnpublishedChanges, editor, load, saveDraft, publish, save: saveDraft, fillStartingEleven, dispose }
}
