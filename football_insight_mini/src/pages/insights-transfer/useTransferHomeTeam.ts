import { computed, ref, watch } from 'vue'
import { getSupportProfile, listSupportTeams, setFavoriteTeam } from '../../api/support'
import type { AppSession } from '../../stores/appSession'
import type { SupportTeam } from '../../types/support'
import { extractApiErrorMessage } from '../../utils/apiError'

const GUEST_DEFAULT_TEAM = '成都蓉城'

/**
 * 转会表态只对「我的主队」：已登录且已设主队 → 固定为主队；已登录没有主队 → 先选主队；
 * 游客 → 只读展示默认球队（表态时走登录）。页面不再提供换队入口。
 */
export function useTransferHomeTeam(session: Pick<AppSession, 'currentUser' | 'revision' | 'status'>) {
  const teams = ref<SupportTeam[]>([])
  const favoriteId = ref<number | null>(null)
  const loading = ref(false), submitting = ref(false), error = ref('')
  const resolved = ref(false)
  let disposed = false, generation = 0

  const loggedIn = computed(() => Boolean(session.currentUser.value) && session.status.value === 'authenticated')
  const needsPick = computed(() => resolved.value && loggedIn.value && favoriteId.value === null)
  const teamId = computed<number | null>(() => {
    if (!resolved.value || needsPick.value) return null
    if (loggedIn.value) return favoriteId.value
    return teams.value.find(team => team.team_name === GUEST_DEFAULT_TEAM)?.team_id ?? teams.value[0]?.team_id ?? null
  })
  const team = computed(() => teams.value.find(item => item.team_id === teamId.value) ?? null)

  async function load(): Promise<void> {
    if (disposed) return
    const owner = ++generation
    loading.value = true; error.value = ''
    try {
      if (!teams.value.length) teams.value = await listSupportTeams()
      let favorite: number | null = null
      if (loggedIn.value) favorite = (await getSupportProfile()).favorite_team?.team_id ?? null
      if (disposed || owner !== generation) return
      favoriteId.value = favorite
      resolved.value = true
    } catch (cause) {
      if (!disposed && owner === generation) error.value = extractApiErrorMessage(cause, '主队信息加载失败，请重试')
    } finally {
      if (!disposed && owner === generation) loading.value = false
    }
  }

  async function pick(id: number): Promise<void> {
    if (submitting.value || !teams.value.some(item => item.team_id === id)) return
    submitting.value = true; error.value = ''
    try {
      await setFavoriteTeam({ team_id: id })
      if (!disposed) favoriteId.value = id
    } catch (cause) {
      if (!disposed) error.value = extractApiErrorMessage(cause, '设置主队失败，请重试')
    } finally {
      if (!disposed) submitting.value = false
    }
  }

  // 换账号/登录状态变化后重新读取主队，不沿用上一个身份的结果。
  const stop = watch([() => session.currentUser.value?.id, session.status, session.revision], () => {
    resolved.value = false; favoriteId.value = null
    void load()
  })
  function dispose(): void { disposed = true; generation++; stop() }
  return { teams, team, teamId, needsPick, loggedIn, loading, submitting, error, load, pick, dispose }
}
