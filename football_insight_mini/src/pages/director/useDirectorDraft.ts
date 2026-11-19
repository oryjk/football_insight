import { computed, ref } from 'vue'
import { getDirectorCandidates, getDirectorContext, upsertDirectorPlan } from '../../api/director'
import type { AppSession } from '../../stores/appSession'
import type {
  DirectorCandidateDto,
  DirectorContextResponse,
  DirectorLeagueFilter,
  DirectorPlanDetailResponse,
  DirectorPosition,
  PlanPlayerItem,
} from '../../types/director'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { buildDirectorEditUrl, DEFAULT_DIRECTOR_TEAM_ID } from '../../utils/directorLinks'
import { createLatestGuard } from '../../utils/latestGuard'
import { rememberPostLoginRedirect } from '../../utils/postLoginRedirect'
import {
  candidateToPlanPlayer,
  computeDirectorStyleTag,
  countUnicodeChars,
  createCustomPlanPlayer,
  deserializeDirectorDraft,
  DIRECTOR_DRAFT_STORAGE_KEY,
  DIRECTOR_MANIFESTO_MAX,
  filterCandidatesByLeague,
  serializeDirectorDraft,
  toUpsertPlayerInput,
  validateForeignQuota,
} from './helpers'

export interface DirectorRouteContext {
  teamId: number
  circleCode: string
  fromCode: string
}

export function useDirectorDraft(session: AppSession) {
  const routeCtx = ref<DirectorRouteContext>({ teamId: 0, circleCode: '', fromCode: '' })
  const activeTeamId = ref<number>(DEFAULT_DIRECTOR_TEAM_ID)
  const context = ref<DirectorContextResponse | null>(null)
  const loading = ref(false)
  const error = ref('')
  const saving = ref(false)

  const foreignStayIds = ref<number[]>([])
  const foreignLeaveIds = ref<number[]>([])
  const foreignIns = ref<PlanPlayerItem[]>([])
  const domesticIns = ref<PlanPlayerItem[]>([])
  const domesticOuts = ref<number[]>([])
  const manifesto = ref('')

  const sheetVisible = ref(false)
  const sheetMode = ref<'foreign' | 'domestic'>('foreign')
  const leagueFilter = ref<DirectorLeagueFilter>('all')
  const searchQuery = ref('')
  const rawCandidates = ref<DirectorCandidateDto[]>([])
  const candidatesLoading = ref(false)
  const candidatesError = ref('')

  const contextGuard = createLatestGuard(() => [activeTeamId.value, session.revision.value])
  const candidateGuard = createLatestGuard(() => [activeTeamId.value, sheetMode.value, leagueFilter.value, searchQuery.value])

  const foreignQuota = computed(() => context.value?.foreign_quota ?? 5)
  const maxDomesticIns = computed(() => context.value?.max_domestic_ins ?? 3)
  const maxDomesticOuts = computed(() => context.value?.max_domestic_outs ?? 3)

  const quotaCheck = computed(() =>
    validateForeignQuota(foreignStayIds.value.length, foreignIns.value.length, foreignQuota.value),
  )

  const previewStyleTag = computed(() =>
    computeDirectorStyleTag({
      foreignStaysCount: foreignStayIds.value.length,
      foreignLeavesCount: foreignLeaveIds.value.length,
      foreignIns: foreignIns.value,
      domesticIns: domesticIns.value,
    }),
  )

  const filteredCandidates = computed(() =>
    filterCandidatesByLeague(rawCandidates.value, leagueFilter.value),
  )

  const selectedForeignKeys = computed(() => new Set(foreignIns.value.map(item => item.key)))
  const selectedDomesticKeys = computed(() => new Set(domesticIns.value.map(item => item.key)))

  function setRouteContext(next: DirectorRouteContext): void {
    routeCtx.value = next
    if (next.teamId > 0) {
      activeTeamId.value = next.teamId
    }
  }

  function applyDefaultSquad(data: DirectorContextResponse): void {
    const quota = data.foreign_quota || 5
    const stays: number[] = []
    const leaves: number[] = []
    data.squad_foreign.forEach((player, index) => {
      if (index < quota) stays.push(player.player_id)
      else leaves.push(player.player_id)
    })
    foreignStayIds.value = stays
    foreignLeaveIds.value = leaves
    foreignIns.value = []
    domesticIns.value = []
    domesticOuts.value = []
    manifesto.value = ''
  }

  function applySavedPlan(data: DirectorContextResponse): void {
    const plan = data.my_plan
    if (!plan) {
      applyDefaultSquad(data)
      return
    }
    const validForeignIds = new Set(data.squad_foreign.map(p => p.player_id))
    const staySet = new Set(
      plan.foreign_stays.map(p => p.player_id).filter((id): id is number => typeof id === 'number' && validForeignIds.has(id)),
    )
    const leaveSet = new Set(
      plan.foreign_leaves.map(p => p.player_id).filter((id): id is number => typeof id === 'number' && validForeignIds.has(id) && !staySet.has(id)),
    )
    // 兜底：若球队新增了现役外援，默认放进留队或离队
    for (const player of data.squad_foreign) {
      if (!staySet.has(player.player_id) && !leaveSet.has(player.player_id)) {
        leaveSet.add(player.player_id)
      }
    }
    foreignStayIds.value = [...staySet]
    foreignLeaveIds.value = [...leaveSet]
    foreignIns.value = [...plan.foreign_ins]
    domesticIns.value = [...plan.domestic_ins]
    domesticOuts.value = plan.domestic_outs
      .map(p => p.player_id)
      .filter((id): id is number => typeof id === 'number')
    manifesto.value = plan.manifesto || ''
  }

  function tryRestorePendingDraft(data: DirectorContextResponse): boolean {
    let raw: unknown = ''
    try {
      raw = uni.getStorageSync(DIRECTOR_DRAFT_STORAGE_KEY)
    } catch {
      return false
    }
    const draft = deserializeDirectorDraft(raw)
    if (!draft || draft.teamId !== data.team.team_id) {
      return false
    }
    const validForeign = new Set(data.squad_foreign.map(p => p.player_id))
    const validDomestic = new Set(data.squad_domestic.map(p => p.player_id))
    foreignStayIds.value = draft.foreignStayIds.filter(id => validForeign.has(id))
    foreignLeaveIds.value = draft.foreignLeaveIds.filter(id => validForeign.has(id) && !foreignStayIds.value.includes(id))
    for (const player of data.squad_foreign) {
      if (!foreignStayIds.value.includes(player.player_id) && !foreignLeaveIds.value.includes(player.player_id)) {
        foreignLeaveIds.value.push(player.player_id)
      }
    }
    foreignIns.value = draft.foreignIns
    domesticIns.value = draft.domesticIns
    domesticOuts.value = draft.domesticOuts.filter(id => validDomestic.has(id))
    manifesto.value = draft.manifesto
    if (!routeCtx.value.circleCode && draft.circleCode) {
      routeCtx.value = { ...routeCtx.value, circleCode: draft.circleCode, fromCode: draft.fromCode }
    }
    try {
      uni.removeStorageSync(DIRECTOR_DRAFT_STORAGE_KEY)
    } catch {
      // 忽略清理失败
    }
    return true
  }

  function persistDraftForLogin(): void {
    const teamId = context.value?.team.team_id ?? activeTeamId.value
    try {
      uni.setStorageSync(
        DIRECTOR_DRAFT_STORAGE_KEY,
        serializeDirectorDraft({
          teamId,
          circleCode: routeCtx.value.circleCode,
          fromCode: routeCtx.value.fromCode,
          foreignStayIds: foreignStayIds.value,
          foreignLeaveIds: foreignLeaveIds.value,
          foreignIns: foreignIns.value,
          domesticIns: domesticIns.value,
          domesticOuts: domesticOuts.value,
          manifesto: manifesto.value,
          savedAt: Date.now(),
        }),
      )
    } catch {
      // 存储不可用时忽略
    }
  }

  async function loadContext(teamId?: number): Promise<void> {
    if (teamId && teamId > 0) {
      activeTeamId.value = teamId
    }
    const token = contextGuard.begin()
    loading.value = true
    error.value = ''
    try {
      const data = await getDirectorContext(activeTeamId.value)
      if (!token.isCurrent()) return
      context.value = data
      activeTeamId.value = data.team.team_id
      const restored = tryRestorePendingDraft(data)
      if (!restored) {
        applySavedPlan(data)
      } else {
        uni.showToast({ title: '已恢复你的草稿', icon: 'none' })
      }
    } catch (cause) {
      if (token.isCurrent()) {
        error.value = extractApiErrorMessage(cause, '建队数据加载失败，请稍后重试')
      }
    } finally {
      if (token.owns()) {
        loading.value = false
      }
    }
  }

  function setForeignAction(playerId: number, action: 'stay' | 'leave'): void {
    if (action === 'stay') {
      if (foreignStayIds.value.includes(playerId)) return
      if (foreignStayIds.value.length + foreignIns.value.length >= foreignQuota.value) {
        uni.showToast({ title: `外援名额已满（${foreignQuota.value}/${foreignQuota.value}），请先精简`, icon: 'none' })
        return
      }
      foreignLeaveIds.value = foreignLeaveIds.value.filter(id => id !== playerId)
      foreignStayIds.value = [...foreignStayIds.value, playerId]
    } else {
      if (foreignLeaveIds.value.includes(playerId)) return
      foreignStayIds.value = foreignStayIds.value.filter(id => id !== playerId)
      foreignLeaveIds.value = [...foreignLeaveIds.value, playerId]
    }
  }

  function removeForeignIn(key: string): void {
    foreignIns.value = foreignIns.value.filter(item => item.key !== key)
  }

  function removeDomesticIn(key: string): void {
    domesticIns.value = domesticIns.value.filter(item => item.key !== key)
  }

  function toggleDomesticOut(playerId: number): void {
    if (domesticOuts.value.includes(playerId)) {
      domesticOuts.value = domesticOuts.value.filter(id => id !== playerId)
      return
    }
    if (domesticOuts.value.length >= maxDomesticOuts.value) {
      uni.showToast({ title: `国内离队建议最多选 ${maxDomesticOuts.value} 人`, icon: 'none' })
      return
    }
    domesticOuts.value = [...domesticOuts.value, playerId]
  }

  async function fetchCandidates(): Promise<void> {
    if (leagueFilter.value === 'CUSTOM') {
      candidatesLoading.value = false
      candidatesError.value = ''
      return
    }
    const token = candidateGuard.begin()
    candidatesLoading.value = true
    candidatesError.value = ''
    try {
      const res = await getDirectorCandidates({
        teamId: activeTeamId.value,
        isForeign: sheetMode.value === 'foreign',
        league: leagueFilter.value,
        q: searchQuery.value,
      })
      if (!token.isCurrent()) return
      rawCandidates.value = res.items
    } catch (cause) {
      if (token.isCurrent()) {
        candidatesError.value = extractApiErrorMessage(cause, '候选人加载失败，请重试')
      }
    } finally {
      if (token.owns()) {
        candidatesLoading.value = false
      }
    }
  }

  function openCandidateSheet(mode: 'foreign' | 'domestic'): void {
    if (mode === 'foreign' && quotaCheck.value.remaining <= 0) {
      uni.showToast({ title: '外援名额已满，请先让现役外援离队或移除新签外援', icon: 'none' })
      return
    }
    if (mode === 'domestic' && domesticIns.value.length >= maxDomesticIns.value) {
      uni.showToast({ title: `国内重点补强最多选 ${maxDomesticIns.value} 人`, icon: 'none' })
      return
    }
    sheetMode.value = mode
    leagueFilter.value = 'all'
    searchQuery.value = ''
    sheetVisible.value = true
    void fetchCandidates()
  }

  function setLeagueFilter(next: DirectorLeagueFilter): void {
    if (leagueFilter.value === next) return
    leagueFilter.value = next
    void fetchCandidates()
  }

  function setSearchQuery(next: string): void {
    searchQuery.value = next
    void fetchCandidates()
  }

  function pickCandidate(candidate: DirectorCandidateDto): void {
    const item = candidateToPlanPlayer(candidate)
    if (sheetMode.value === 'foreign') {
      if (selectedForeignKeys.value.has(item.key)) {
        removeForeignIn(item.key)
        return
      }
      if (foreignStayIds.value.length + foreignIns.value.length >= foreignQuota.value) {
        uni.showToast({ title: `外援名额已满（${foreignQuota.value}/${foreignQuota.value}）`, icon: 'none' })
        return
      }
      foreignIns.value = [...foreignIns.value, item]
      if (foreignStayIds.value.length + foreignIns.value.length >= foreignQuota.value) {
        sheetVisible.value = false
      }
    } else {
      if (selectedDomesticKeys.value.has(item.key)) {
        removeDomesticIn(item.key)
        return
      }
      if (domesticIns.value.length >= maxDomesticIns.value) {
        uni.showToast({ title: `国内补强最多选 ${maxDomesticIns.value} 人`, icon: 'none' })
        return
      }
      domesticIns.value = [...domesticIns.value, item]
      if (domesticIns.value.length >= maxDomesticIns.value) {
        sheetVisible.value = false
      }
    }
  }

  function addCustomCandidate(input: {
    playerName: string
    position: DirectorPosition
    leagueTag: string
    currentClub: string
  }): boolean {
    const name = input.playerName.trim()
    if (!name) {
      uni.showToast({ title: '请输入球员姓名', icon: 'none' })
      return false
    }
    const item = createCustomPlanPlayer({
      ...input,
      isForeign: sheetMode.value === 'foreign',
    })
    if (sheetMode.value === 'foreign') {
      if (selectedForeignKeys.value.has(item.key)) {
        uni.showToast({ title: '该球员已在你的引援名单中', icon: 'none' })
        return false
      }
      if (foreignStayIds.value.length + foreignIns.value.length >= foreignQuota.value) {
        uni.showToast({ title: `外援名额已满（${foreignQuota.value}/${foreignQuota.value}）`, icon: 'none' })
        return false
      }
      foreignIns.value = [...foreignIns.value, item]
      sheetVisible.value = false
      return true
    }
    if (selectedDomesticKeys.value.has(item.key)) {
      uni.showToast({ title: '该球员已在你的引援名单中', icon: 'none' })
      return false
    }
    if (domesticIns.value.length >= maxDomesticIns.value) {
      uni.showToast({ title: `国内补强最多选 ${maxDomesticIns.value} 人`, icon: 'none' })
      return false
    }
    domesticIns.value = [...domesticIns.value, item]
    sheetVisible.value = false
    return true
  }

  function redirectToLogin(): void {
    persistDraftForLogin()
    const targetUrl = buildDirectorEditUrl({
      teamId: context.value?.team.team_id ?? activeTeamId.value,
      circleCode: routeCtx.value.circleCode,
      fromCode: routeCtx.value.fromCode,
    })
    rememberPostLoginRedirect({ type: 'navigateTo', url: targetUrl })
    uni.showToast({ title: '请先登录后保存方案', icon: 'none' })
    uni.switchTab({ url: '/pages/user/index' })
  }

  async function submitPlan(): Promise<DirectorPlanDetailResponse | null> {
    if (saving.value) return null
    if (!quotaCheck.value.valid) {
      uni.showToast({ title: quotaCheck.value.message || '请先检查外援名额', icon: 'none' })
      return null
    }
    if (countUnicodeChars(manifesto.value.trim()) > DIRECTOR_MANIFESTO_MAX) {
      uni.showToast({ title: `总监宣言最多 ${DIRECTOR_MANIFESTO_MAX} 字`, icon: 'none' })
      return null
    }
    if (!session.hasAccessToken.value) {
      redirectToLogin()
      return null
    }

    saving.value = true
    try {
      const result = await upsertDirectorPlan({
        team_id: context.value?.team.team_id ?? activeTeamId.value,
        circle_code: routeCtx.value.circleCode || null,
        parent_share_code: routeCtx.value.fromCode || null,
        foreign_stay_ids: foreignStayIds.value,
        foreign_leave_ids: foreignLeaveIds.value,
        foreign_ins: foreignIns.value.map(toUpsertPlayerInput),
        domestic_ins: domesticIns.value.map(toUpsertPlayerInput),
        domestic_outs: domesticOuts.value,
        manifesto: manifesto.value.trim(),
      })
      return result
    } catch (cause) {
      if (isUnauthorizedError(cause)) {
        redirectToLogin()
        return null
      }
      uni.showToast({ title: extractApiErrorMessage(cause, '保存方案失败，请稍后重试'), icon: 'none' })
      return null
    } finally {
      saving.value = false
    }
  }

  function dispose(): void {
    contextGuard.invalidate()
    candidateGuard.invalidate()
  }

  return {
    routeCtx,
    activeTeamId,
    context,
    loading,
    error,
    saving,
    foreignStayIds,
    foreignLeaveIds,
    foreignIns,
    domesticIns,
    domesticOuts,
    manifesto,
    foreignQuota,
    maxDomesticIns,
    maxDomesticOuts,
    quotaCheck,
    previewStyleTag,
    sheetVisible,
    sheetMode,
    leagueFilter,
    searchQuery,
    filteredCandidates,
    candidatesLoading,
    candidatesError,
    selectedForeignKeys,
    selectedDomesticKeys,
    setRouteContext,
    loadContext,
    setForeignAction,
    removeForeignIn,
    removeDomesticIn,
    toggleDomesticOut,
    openCandidateSheet,
    setLeagueFilter,
    setSearchQuery,
    fetchCandidates,
    pickCandidate,
    addCustomCandidate,
    submitPlan,
    dispose,
  }
}
