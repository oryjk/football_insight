import { computed, ref, watch } from 'vue'
import { getMyLineup, getSquad, saveMyLineup } from '../../api/lineup'
import { useAppSession } from '../../stores/appSession'
import type { SavedLineup, SquadResponse } from '../../types/lineup'
import { extractApiErrorMessage } from '../../utils/apiError'
import { DEFAULT_FORMATION } from './formations'
import { emptyLineup, type LineupSlots } from './helpers'
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
}

export type SaveResult = 'saved' | 'need-login' | 'failed'
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
    if (!teamId.value) return
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
      if (disposed || !editor.dirty.value) return
      cancelDraftTimer()
      draftTimer = setTimeout(persistDraft, deps.draftDelayMs)
    },
    { deep: true },
  )

  async function load(nextTeamId: number): Promise<void> {
    const seq = ++loadSeq
    const current = () => seq === loadSeq
    disposed = false
    cancelDraftTimer()
    teamId.value = nextTeamId
    loading.value = true
    errorMessage.value = ''
    notice.value = ''
    failed.value = false
    savedOnce.value = false
    try {
      const [loadedSquad, mine] = await Promise.all([
        deps.getSquad(nextTeamId),
        deps.isLoggedIn() ? deps.getMyLineup(nextTeamId) : Promise.resolve({ lineup: null }),
      ])
      if (!current()) return
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
        editor.load(DEFAULT_FORMATION, emptyLineup(DEFAULT_FORMATION))
      }
    } catch (error) {
      if (!current()) return
      errorMessage.value = extractApiErrorMessage(error, '战术板加载失败')
    } finally {
      if (current()) loading.value = false
    }
  }

  async function save(): Promise<SaveResult> {
    if (!deps.isLoggedIn()) {
      cancelDraftTimer()
      persistDraft()
      return 'need-login'
    }
    if (!editor.dirty.value && !failed.value) return 'saved'
    cancelDraftTimer()
    saving.value = true
    const revisionAtStart = editor.revision.value
    try {
      await deps.saveMyLineup(teamId.value, { formation: editor.formation.value, slots: editor.slots.value })
      failed.value = false
      savedOnce.value = true
      if (editor.revision.value === revisionAtStart) {
        editor.markSaved()
        clearDraft(teamId.value, deps.storage)
      } else {
        // 保存期间又改了位置：这次保存没有覆盖最新改动，保持未保存并保留草稿。
        persistDraft()
      }
      return 'saved'
    } catch {
      persistDraft()
      failed.value = true
      return 'failed'
    } finally {
      saving.value = false
    }
  }

  function dispose(): void {
    // 离开时若还有没来得及写草稿的改动（系统返回手势没有确认框），先写下来再停定时器。
    if (!disposed && editor.dirty.value) persistDraft()
    disposed = true
    cancelDraftTimer()
  }

  return { teamId, squad, loading, errorMessage, notice, saving, saveState, editor, load, save, dispose }
}
