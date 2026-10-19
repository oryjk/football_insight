import { ref, watch, type Ref } from 'vue'
import { getMyLineup } from '../../api/lineup'
import type { AppSession } from '../../stores/appSession'
import type { PublicLineup } from '../../types/lineup'

/** Match a public code to the authenticated user's private published snapshot. */
export function useLineupOwnership(
  detail: Readonly<Ref<PublicLineup | null>>,
  session: Pick<AppSession, 'currentUser' | 'hasAccessToken' | 'status' | 'revision'>,
  mine: typeof getMyLineup = getMyLineup,
) {
  const isOwner = ref(false), loading = ref(false)
  let generation = 0, disposed = false
  const userId = () => session.hasAccessToken.value
    && (session.status.value === 'authenticated' || session.status.value === 'error')
    ? session.currentUser.value?.id ?? '' : ''
  // 多数据源逐项比较：同一账号的 currentUser 被换成新对象时 userId 不变，不重新查归属。
  const stop = watch([() => session.revision.value, userId, () => detail.value?.share_code, () => detail.value?.team.team_id],
    ([revision, user, code, teamId]) => {
      const owner = ++generation
      isOwner.value = false
      loading.value = false
      if (disposed || !user || !code || !teamId) return
      const current = () => !disposed && generation === owner && session.revision.value === revision
        && userId() === user && detail.value?.share_code === code && detail.value?.team.team_id === teamId
      loading.value = true
      void (async () => {
        try {
          const result = await mine(teamId)
          if (current()) isOwner.value = result.lineup?.share_code === code
        } catch {
          // Public browsing and arranging one's own team remain available if this private read fails.
          if (current()) isOwner.value = false
        } finally {
          if (current()) loading.value = false
        }
      })()
    }, { immediate: true })
  function dispose(): void { disposed = true; generation++; stop(); isOwner.value = false; loading.value = false }
  return { isOwner, loading, dispose }
}
