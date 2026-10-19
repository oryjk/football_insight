import { computed, ref, watch } from 'vue'
import { getMyLineup, getSquad, saveMyLineup } from '../../api/lineup'
import { useAppSession } from '../../stores/appSession'
import type { SavedLineup, SquadResponse } from '../../types/lineup'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { DEFAULT_FORMATION } from './formations'
import { autoFillStartingEleven, emptyLineup, type LineupSlots } from './helpers'
import { clearDraft, readDraft, sanitizeDraftSlots, writeDraft, type DraftStorage } from './lineupDraft'
import { useLineupEditor } from './useLineupEditor'

export interface LineupPageDeps {
  getSquad: (teamId: number) => Promise<SquadResponse>
  getMyLineup: (teamId: number) => Promise<{ lineup: SavedLineup | null }>
  saveMyLineup: (
    teamId: number,
    payload: { formation: string; slots: LineupSlots },
  ) => Promise<{ lineup: SavedLineup }>
  isLoggedIn: () => boolean
  storage: DraftStorage
  now: () => number
  /** 改动后多久写草稿（毫秒）。 */
  draftDelayMs: number
  sessionRevision: () => number
  invalidateSession: (revision: number) => void
}

export type SaveResult = 'saved' | 'need-login' | 'failed' | 'cancelled'
export type SaveState = 'idle' | 'saved' | 'dirty' | 'error'

/** 战术板数据层：加载名单与我的阵容、草稿、保存。唯一调用 api/lineup 的地方。 */
export function useLineupPage(overrides: Partial<LineupPageDeps> = {}) {
  const session = overrides.isLoggedIn ? null : useAppSession()
  const deps: LineupPageDeps = {
    getSquad,
    getMyLineup,
    saveMyLineup,
    isLoggedIn: () => Boolean(session?.hasAccessToken.value),
    // uni 只在小程序运行时存在；测试注入内存存储时不能提前引用它。
    storage: overrides.storage ?? uni,
    now: Date.now,
    draftDelayMs: 300,
    sessionRevision: () => session?.revision.value ?? 0,
    invalidateSession: (revision) => session?.invalidateSession(revision),
    ...overrides,
  }

  const editor = useLineupEditor()
  const teamId = ref(0)
  const squad = ref<SquadResponse | null>(null)
  const loading = ref(false)
  const errorMessage = ref('')
  const saving = ref(false)
  const failed = ref(false)
  const savedOnce = ref(false)
  const savedLineup = ref<SavedLineup | null>(null)
  /** 加载时发现的需要告诉用户的情况（如阵容里的球员已不在名单）。 */
  const notice = ref('')
  // 乱序响应保护：连续切换球队时只采用最后一次 load 的结果。
  let loadSeq = 0

  const saveState = computed<SaveState>(() => {
    if (failed.value) return 'error'
    if (editor.dirty.value) return 'dirty'
    return savedOnce.value ? 'saved' : 'idle'
  })

  let draftTimer: ReturnType<typeof setTimeout> | null = null
  // 页面卸载后（或卸载的同一时刻仍有待触发的 watch）不再创建草稿定时器。
  let disposed = false

  function persistDraft(): void {
    // 加载中的编辑器不属于目标球队，不能把它写入目标球队的草稿。
    if (disposed || loading.value || !squad.value || !teamId.value) return
    writeDraft(
      { teamId: teamId.value, formation: editor.formation.value, slots: editor.slots.value, savedAt: deps.now() },
      deps.storage,
    )
  }

  function cancelDraftTimer(): void {
    if (draftTimer) clearTimeout(draftTimer)
    draftTimer = null
  }

  // 每次改动 debounce 后写草稿：不管有没有登录，系统返回手势退出也不丢改动。
  watch(
    [editor.slots, editor.formation],
    () => {
      if (disposed || loading.value || !squad.value || !editor.dirty.value) return
      cancelDraftTimer()
      draftTimer = setTimeout(persistDraft, deps.draftDelayMs)
    },
    { deep: true },
  )

  async function load(nextTeamId: number): Promise<void> {
    // 必须在换 teamId 前落盘；取消 debounce 本身不能保证最后一次改动已保存。
    if (editor.dirty.value) persistDraft()
    const seq = ++loadSeq
    let authRevision = deps.sessionRevision()
    const current = () => !disposed && seq === loadSeq && authRevision === deps.sessionRevision()
    disposed = false
    cancelDraftTimer()
    teamId.value = nextTeamId
    loading.value = true
    squad.value = null
    saving.value = false
    editor.load(DEFAULT_FORMATION, emptyLineup(DEFAULT_FORMATION))
    errorMessage.value = ''
    notice.value = ''
    failed.value = false
    savedOnce.value = false
    savedLineup.value = null
    try {
      const [loadedSquad, mine] = await Promise.all([
        deps.getSquad(nextTeamId),
        deps.isLoggedIn() ? deps.getMyLineup(nextTeamId).catch(error => {
          if (!current() || !isUnauthorizedError(error)) throw error
          deps.invalidateSession(authRevision)
          authRevision = deps.sessionRevision()
          return { lineup: null }
        }) : Promise.resolve({ lineup: null }),
      ])
      if (!current()) return
      savedLineup.value = mine.lineup
      squad.value = loadedSquad
      const known = new Set([...loadedSquad.active, ...loadedSquad.former].map((p) => p.player_id))
      const draft = readDraft(nextTeamId, deps.storage)
      if (draft) {
        editor.load(draft.formation, sanitizeDraftSlots(draft.slots, known), true)
      } else if (mine.lineup) {
        const slots = mine.lineup.slots as LineupSlots
        const clean = sanitizeDraftSlots(slots, known)
        const removed = clean.some((s, i) => s.player_id !== slots[i].player_id)
        // 阵容里有球员已不在名单：置空、标记未保存，否则每次保存都会被后端 400 拒绝且没有任何解释。
        editor.load(mine.lineup.formation, clean, removed)
        if (removed) notice.value = '阵容里有球员已不在球队名单，已从阵容中移出，请重新选择后保存。'
        savedOnce.value = !removed
      } else {
        // 第一次进来（没有已保存阵容也没有草稿）：给一套按位置与号码推出的初始首发，不算"改动"，用户再自己换。
        editor.load(
          DEFAULT_FORMATION,
          autoFillStartingEleven(emptyLineup(DEFAULT_FORMATION), DEFAULT_FORMATION, loadedSquad.active),
        )
      }
    } catch (error) {
      if (!current()) return
      errorMessage.value = extractApiErrorMessage(error, '战术板加载失败')
    } finally {
      if (!disposed && seq === loadSeq) loading.value = false
    }
  }

  async function save(): Promise<SaveResult> {
    if (disposed || loading.value || !squad.value || saving.value) return 'cancelled'
    if (!deps.isLoggedIn()) {
      cancelDraftTimer()
      persistDraft()
      return 'need-login'
    }
    if (savedOnce.value && !editor.dirty.value && !failed.value) return 'saved'
    cancelDraftTimer()
    saving.value = true
    const teamAtStart = teamId.value
    const contextAtStart = loadSeq
    const revisionAtStart = editor.revision.value
    const authRevision = deps.sessionRevision()
    const current = () => !disposed && loadSeq === contextAtStart && teamId.value === teamAtStart && deps.sessionRevision() === authRevision
    try {
      const response = await deps.saveMyLineup(teamAtStart, {
        formation: editor.formation.value,
        slots: editor.slots.value.map((slot) => ({ ...slot })),
      })
      if (!current()) return 'cancelled'
      savedLineup.value = response.lineup
      failed.value = false
      savedOnce.value = true
      if (editor.revision.value === revisionAtStart) {
        editor.markSaved()
        cancelDraftTimer()
        clearDraft(teamAtStart, deps.storage)
      } else {
        // 保存期间又改了位置：这次保存没有覆盖最新改动，保持未保存并保留草稿。
        persistDraft()
      }
      return 'saved'
    } catch (error) {
      if (!current()) return 'cancelled'
      if (isUnauthorizedError(error)) {
        persistDraft()
        saving.value = false
        deps.invalidateSession(authRevision)
        return 'need-login'
      }
      persistDraft()
      failed.value = true
      return 'failed'
    } finally {
      if (!disposed && loadSeq === contextAtStart && teamId.value === teamAtStart) saving.value = false
    }
  }

  /** 一键填入首发：只填空位、不动已排的球员；球员名单还没加载时不做任何事。 */
  function fillStartingEleven(): void {
    if (!squad.value) return
    const filled = autoFillStartingEleven(editor.slots.value, editor.formation.value, squad.value.active)
    editor.slots.value.forEach((slot, index) => {
      if (slot.player_id === null && filled[index].player_id !== null) editor.pick(slot.slot, filled[index].player_id)
    })
  }

  function dispose(): void {
    // 离开时若还有没来得及写草稿的改动（系统返回手势没有确认框），先写下来再停定时器。
    if (!disposed && editor.dirty.value) persistDraft()
    disposed = true
    loadSeq += 1
    cancelDraftTimer()
  }

  return { teamId, squad, loading, errorMessage, notice, saving, saveState, savedLineup, editor, load, save, fillStartingEleven, dispose }
}
