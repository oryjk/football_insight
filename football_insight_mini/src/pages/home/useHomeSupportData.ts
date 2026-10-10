import { ref, type Ref } from 'vue'
import { getSupportProfile, listSupportTeams } from '../../api/support'
import type { SupportProfile, SupportTeam } from '../../types/support'
import type { AppSession } from '../../stores/appSession'
import { extractApiErrorMessage } from '../../utils/apiError'
import { isHomeAuthExpiredError } from './helpers'

export interface HomeSupportDataDeps {
  session: AppSession
  /** 测试注入可控的私有/公开读取；默认走真实 api 层。 */
  getSupportProfile?: () => Promise<SupportProfile>
  listSupportTeams?: () => Promise<SupportTeam[]>
}

export interface HomeSupportData {
  supportLoading: Ref<boolean>
  supportErrorMessage: Ref<string>
  supportProfile: Ref<SupportProfile | null>
  supportTeams: Ref<SupportTeam[]>
  supportTeamsLoading: Ref<boolean>
  loginPromptVisible: Ref<boolean>
  favoriteTeamSheetVisible: Ref<boolean>
  selectedFavoriteTeamId: Ref<number | null>
  /**
   * 普通加载：同 revision 的在途读取合并（onShow 与 currentUser watcher 同会话只发一次）。
   * force（写后刷新，如设置主队成功后）：不合并写前在途读取——旧 owner 立即失效，
   * 其成功/失败/finally 都不再发布，直接建立新 owner 重新读。
   */
  loadSupportData: (options?: { force?: boolean }) => Promise<void>
  loadSupportTeams: () => Promise<void>
  /** 会话身份变化（revision 递增）时同步丢弃旧账号私有缓存/选择/弹层。 */
  resetSessionScopedState: () => void
}

/**
 * 首页助力面板（账号私有数据）的加载编排。
 *
 * - 私有读取先等待共享 ensure 确认用户：与启动 bootstrap 合并在途 /auth/me，
 *   旧 token 的 me 401 由 session 统一失效，这里不再抢发 profile 请求提前取消恢复；
 * - onShow 与 currentUser watcher 在同一会话（同 revision）触发的加载合并为一个
 *   私有请求；正常刷新（上一请求已结束）建立新 owner；
 * - 写后刷新（设置主队成功后）走 force 入口：不与写前在途 GET 合并，写前读取的
 *   成功/失败/finally 全部失效，避免旧主队快照迟到后覆盖刚写入的新选择；
 * - success/catch/finally 都核对 owner + revision：旧账号的迟到结果不发布、
 *   旧 finally 不清掉新请求的 loading；ensure 之后发私有请求前同样核对。
 */
export function useHomeSupportData(deps: HomeSupportDataDeps): HomeSupportData {
  const session = deps.session
  const fetchSupportProfile = deps.getSupportProfile ?? getSupportProfile
  const fetchSupportTeams = deps.listSupportTeams ?? listSupportTeams

  const supportLoading = ref(true)
  const supportErrorMessage = ref('')
  const supportProfile = ref<SupportProfile | null>(null)
  const supportTeams = ref<SupportTeam[]>([])
  const supportTeamsLoading = ref(false)
  const loginPromptVisible = ref(false)
  const favoriteTeamSheetVisible = ref(false)
  const selectedFavoriteTeamId = ref<number | null>(null)

  // 私有加载的 owner：每次新加载（非合并）替换；旧请求凭 owner + revision 判定失效。
  let loadOwner: symbol | null = null
  let inFlightLoad: { revision: number; promise: Promise<void> } | null = null

  function applyGuestView(): void {
    supportProfile.value = null
    supportTeams.value = []
    supportErrorMessage.value = ''
    supportLoading.value = false
    loginPromptVisible.value = true
  }

  function resetSessionScopedState(): void {
    // 跨账号不保留旧缓存：资料/球队/选中主队/弹层/错误一并清掉；
    // 正常同账号刷新不触发本方法（revision 不变），缓存保留。
    supportProfile.value = null
    supportTeams.value = []
    supportTeamsLoading.value = false
    supportErrorMessage.value = ''
    supportLoading.value = false
    favoriteTeamSheetVisible.value = false
    selectedFavoriteTeamId.value = null
    loginPromptVisible.value = !session.currentUser.value
  }

  async function loadSupportTeams(): Promise<void> {
    if (supportTeams.value.length || supportTeamsLoading.value) {
      return
    }

    supportTeamsLoading.value = true
    try {
      supportTeams.value = await fetchSupportTeams()
    } catch (error) {
      console.warn('[home] support teams load failed', error)
    } finally {
      supportTeamsLoading.value = false
    }
  }

  async function runLoad(requestRevision: number, owner: symbol): Promise<void> {
    const isCurrent = () => loadOwner === owner && session.revision.value === requestRevision

    supportErrorMessage.value = ''
    loginPromptVisible.value = false

    if (!session.hasAccessToken.value) {
      applyGuestView()
      return
    }

    supportLoading.value = true
    // 先等待共享 ensure 确认用户；被替代（revision 已变）直接放弃。
    // 断网/5xx 时 ensure 保留 token，沿用旧路径继续尝试私有读取（失败走重试提示）。
    let confirmedUser = session.currentUser.value
    try {
      confirmedUser = await session.ensureSessionReady()
    } catch {
      if (!isCurrent()) {
        return
      }
    }
    // ensure 之后才发私有请求：发出前必须仍是当前 owner + revision。
    if (!isCurrent()) {
      return
    }
    if (!confirmedUser && !session.hasAccessToken.value) {
      // 确认无用户（如无 token/会话已被确认失效）：游客视图，不发私有请求。
      applyGuestView()
      return
    }

    try {
      const profile = await fetchSupportProfile()
      if (!isCurrent()) {
        return
      }
      supportProfile.value = profile
      selectedFavoriteTeamId.value = profile.favorite_team?.team_id ?? selectedFavoriteTeamId.value
      void loadSupportTeams()
    } catch (error) {
      if (!isCurrent()) {
        return
      }

      if (isHomeAuthExpiredError(error)) {
        // 401：仅失效当前 revision 的会话；断网/5xx 走下方重试提示，不清 token。
        session.invalidateSession(requestRevision)
        supportProfile.value = null
        supportErrorMessage.value = ''
        supportLoading.value = false
        loginPromptVisible.value = true
        return
      }

      supportErrorMessage.value = extractApiErrorMessage(error, '助力入口加载失败，请稍后重试。')
    } finally {
      // 旧请求的 finally 不能清掉新 owner 的 loading。
      if (isCurrent()) {
        supportLoading.value = false
      }
    }
  }

  function loadSupportData(options?: { force?: boolean }): Promise<void> {
    const requestRevision = session.revision.value
    if (options?.force) {
      // 写后刷新：替换 owner 使写前在途读取立即失效（其成功/失败/finally 全部被
      // isCurrent 挡住、clearIfCurrent 也不清新条目），不合并旧 promise，直接重新读。
      loadOwner = null
      inFlightLoad = null
    } else if (inFlightLoad && inFlightLoad.revision === requestRevision) {
      // 同 revision 的在途加载合并（onShow 与 currentUser watcher 同会话只发一次私有请求）。
      return inFlightLoad.promise
    }

    const owner = Symbol('home-support-load')
    loadOwner = owner
    const promise = runLoad(requestRevision, owner)
    inFlightLoad = { revision: requestRevision, promise }
    const clearIfCurrent = () => {
      if (inFlightLoad?.promise === promise) {
        inFlightLoad = null
      }
    }
    promise.then(clearIfCurrent, clearIfCurrent)
    return promise
  }

  return {
    supportLoading,
    supportErrorMessage,
    supportProfile,
    supportTeams,
    supportTeamsLoading,
    loginPromptVisible,
    favoriteTeamSheetVisible,
    selectedFavoriteTeamId,
    loadSupportData,
    loadSupportTeams,
    resetSessionScopedState,
  }
}
