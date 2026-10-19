import { ref, watch } from 'vue'
import { getSupportProfile, listSupportTeams } from '../../api/support'
import type { AppSession } from '../../stores/appSession'
import type { SupportTeam } from '../../types/support'
import { extractApiErrorMessage } from '../../utils/apiError'

interface Dependencies {
  list?: typeof listSupportTeams
  profile?: typeof getSupportProfile
  storage?: { getStorageSync(key: string): unknown; setStorageSync(key: string, value: unknown): void }
}
const TEAM_KEY = 'fi-insights-selected-team'

/** 公开球队选择独立于会员统计和慢赛程；仅显式选择或主队初始化可以更改。 */
export function useInsightsTeams(session: AppSession, deps: Dependencies = {}) {
  const teams = ref<SupportTeam[]>([])
  const selectedTeamId = ref<number | null>(null)
  const loading = ref(false), error = ref('')
  const storage = deps.storage ?? uni
  let disposed = false, chosen = false
  let pending: Promise<void> | null = null
  let profileIdentity: string | null = null
  const identity = () => `${session.currentUser.value?.id ?? ''}:${session.revision.value}`

  async function maybeChooseFavorite(): Promise<void> {
    if (disposed || chosen || !teams.value.length || !session.currentUser.value || session.status.value !== 'authenticated') return
    const owner = identity()
    if (profileIdentity === owner) return
    profileIdentity = owner
    try {
      const profile = await (deps.profile ?? getSupportProfile)()
      if (disposed || chosen || owner !== identity() || session.status.value !== 'authenticated') return
      const favorite = profile.favorite_team?.team_id
      if (favorite && teams.value.some(team => team.team_id === favorite)) selectedTeamId.value = favorite
    } catch {
      if (profileIdentity === owner) profileIdentity = null
      // 主队不是首屏必需数据，失败不隐藏公开列表或已选球队。
    }
  }

  function load(): Promise<void> {
    if (disposed) return Promise.resolve()
    if (pending) return pending
    if (teams.value.length) { void maybeChooseFavorite(); return Promise.resolve() }
    loading.value = true; error.value = ''
    const work = (async () => {
      try {
        const list = await (deps.list ?? listSupportTeams)()
        if (disposed) return
        teams.value = list
        let stored: unknown
        try { stored = storage.getStorageSync(TEAM_KEY) } catch { /* 偏好读取失败沿用默认主队 */ }
        const remembered = typeof stored === 'number' && list.some(team => team.team_id === stored)
        chosen = chosen || remembered
        if (!list.some(team => team.team_id === selectedTeamId.value)) {
          selectedTeamId.value = remembered ? stored as number : list.find(team => team.team_name === '成都蓉城')?.team_id ?? list[0]?.team_id ?? null
        }
        void maybeChooseFavorite()
      } catch (cause) {
        if (!disposed) error.value = extractApiErrorMessage(cause, '球队加载失败，请重试')
      } finally {
        if (!disposed) loading.value = false
      }
    })()
    pending = work
    void work.finally(() => { if (pending === work) pending = null })
    return work
  }

  function selectTeam(id: number): void {
    if (!teams.value.some(team => team.team_id === id)) return
    chosen = true
    selectedTeamId.value = id
    try { storage.setStorageSync(TEAM_KEY, id) } catch { /* 不影响本次球队选择 */ }
  }
  const stop = watch([() => session.currentUser.value?.id, session.status], () => void maybeChooseFavorite())
  function dispose(): void { disposed = true; stop() }
  return { teams, selectedTeamId, loading, error, load, selectTeam, dispose }
}
