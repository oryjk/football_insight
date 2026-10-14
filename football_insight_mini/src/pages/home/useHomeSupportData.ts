import { ref, type Ref } from 'vue'
import { castSupportVote, getSupportProfile, listSupportTeams, setFavoriteTeam } from '../../api/support'
import type { CastSupportVotePayload, SetFavoriteTeamPayload, SupportMatchDetail, SupportProfile, SupportTeam } from '../../types/support'
import type { AppSession } from '../../stores/appSession'
import { extractApiErrorMessage } from '../../utils/apiError'
import { isHomeAuthExpiredError } from './helpers'

export interface HomeSupportDataDeps {
  session: AppSession
  /** 测试注入可控的私有/公开读取；默认走真实 api 层。 */
  getSupportProfile?: () => Promise<SupportProfile>
  listSupportTeams?: () => Promise<SupportTeam[]>
  /** 测试注入可控的写操作；默认走真实 api 层。 */
  setFavoriteTeam?: (payload: SetFavoriteTeamPayload) => Promise<SupportTeam>
  castSupportVote?: (matchId: number, payload: CastSupportVotePayload) => Promise<SupportMatchDetail>
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
  /** 设置主队写操作是否提交中（会话作用域，跨账号同步复位）。 */
  favoriteSubmitting: Ref<boolean>
  voteSubmittingTeamId: Ref<number | null>
  /** 直接助力；重复点击或旧会话的迟到结果返回 false，不发布到当前页面。 */
  submitSupportVote: (teamId: number) => Promise<boolean>
  /**
   * 设置主队写操作（带操作身份）。返回 true 表示写成功且调用方仍属于原会话，
   * 可继续关闭 sheet/toast/写后强制刷新；返回 false 表示等待期间会话已切换——
   * 旧写的成功/失败都不再作用于新账号页面（错误也被吞掉，由新会话自己处理）。
   * 写成功后的刷新必须走 loadSupportData({ force: true })。
   */
  submitFavoriteTeam: (teamId: number) => Promise<boolean>
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
 * - 设置主队写操作带独立 owner（submitFavoriteTeam）：PUT 等待期间会话被切换时，
 *   旧写的成功/失败/finally 都不作用于新账号（不关 sheet、不 toast、不强制刷新、
 *   不清新提交的 submitting）；写成功且仍有效时由页面接 force 写后刷新；
 * - success/catch/finally 都核对 owner + revision：旧账号的迟到结果不发布、
 *   旧 finally 不清掉新请求的 loading；ensure 之后发私有请求前同样核对。
 */
export function useHomeSupportData(deps: HomeSupportDataDeps): HomeSupportData {
  const session = deps.session
  const fetchSupportProfile = deps.getSupportProfile ?? getSupportProfile
  const fetchSupportTeams = deps.listSupportTeams ?? listSupportTeams
  const writeFavoriteTeam = deps.setFavoriteTeam ?? setFavoriteTeam
  const writeSupportVote = deps.castSupportVote ?? castSupportVote

  const supportLoading = ref(true)
  const supportErrorMessage = ref('')
  const supportProfile = ref<SupportProfile | null>(null)
  const supportTeams = ref<SupportTeam[]>([])
  const supportTeamsLoading = ref(false)
  const loginPromptVisible = ref(false)
  const favoriteTeamSheetVisible = ref(false)
  const selectedFavoriteTeamId = ref<number | null>(null)
  const favoriteSubmitting = ref(false)
  const voteSubmittingTeamId = ref<number | null>(null)

  // 私有加载的 owner：每次新加载（非合并）替换；旧请求凭 owner + revision 判定失效。
  let loadOwner: symbol | null = null
  let inFlightLoad: { revision: number; promise: Promise<void> } | null = null
  // 设置主队写操作的 owner：每次新写替换；旧写的成功/失败/finally 凭 owner + revision 判定失效。
  let submitOwner: symbol | null = null
  let voteOwner: symbol | null = null

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
    // 写操作状态同步复位：旧写的 finally 凭 owner 判定失效，不会清掉新账号新提交的 submitting。
    submitOwner = null
    favoriteSubmitting.value = false
    voteOwner = null
    voteSubmittingTeamId.value = null
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

  async function submitFavoriteTeam(teamId: number): Promise<boolean> {
    // 写操作身份在调用瞬间、PUT 发出前建立。
    const requestRevision = session.revision.value
    const owner = Symbol('home-favorite-submit')
    submitOwner = owner
    favoriteSubmitting.value = true
    const isCurrent = () => submitOwner === owner && session.revision.value === requestRevision

    try {
      await writeFavoriteTeam({ team_id: teamId })
      // false = 等待 PUT 期间会话已切换：旧写成功不得关闭新账号的 sheet/toast，
      // 也不得 force 刷新替换新账号在途的读取。
      return isCurrent()
    } catch (error) {
      if (!isCurrent()) {
        // 旧 actor 的迟到失败不影响新账号：吞掉错误，不写 errorMessage。
        return false
      }
      throw error
    } finally {
      // 旧 finally 不清掉新 owner 的提交中状态。
      if (isCurrent()) {
        favoriteSubmitting.value = false
      }
    }
  }

  async function submitSupportVote(teamId: number): Promise<boolean> {
    if (voteSubmittingTeamId.value !== null || favoriteSubmitting.value) return false
    const profile = supportProfile.value
    const match = profile?.next_match
    if (!session.hasAccessToken.value) throw new Error('请先登录再助力')
    if (!match) throw new Error('比赛信息已更新，请刷新后再助力')
    if (teamId !== match.home_team.team_id && teamId !== match.away_team.team_id) {
      throw new Error('请选择本场比赛的球队')
    }
    if (match.viewer.has_supported) throw new Error('本场比赛已经助力过了')
    const kickoff = Date.parse(match.kickoff_at)
    if (match.support_window_status !== 'open' || !Number.isFinite(kickoff) || kickoff <= Date.now()) {
      throw new Error('本场助力已关闭')
    }
    if (!match.viewer.can_support) throw new Error('本场暂时不能助力，请刷新后重试')

    const requestRevision = session.revision.value
    const favoriteTeamId = profile.favorite_team?.team_id
    const owner = Symbol('home-support-vote')
    voteOwner = owner
    voteSubmittingTeamId.value = teamId
    const isCurrent = () => voteOwner === owner && session.revision.value === requestRevision
      && supportProfile.value?.next_match?.match_id === match.match_id
      && supportProfile.value?.favorite_team?.team_id === favoriteTeamId
    const invalidateReads = () => {
      loadOwner = null
      inFlightLoad = null
      supportLoading.value = false
    }
    // 助力前的 profile 快照迟到时不能覆盖 POST 返回的票数与已助力状态。
    invalidateReads()
    try {
      const updatedMatch = await writeSupportVote(match.match_id, { supported_team_id: teamId })
      if (!isCurrent()) return false
      if (updatedMatch.match_id !== match.match_id) throw new Error('助力结果与当前比赛不一致，请刷新后重试')
      invalidateReads()
      supportProfile.value = { ...supportProfile.value!, next_match: updatedMatch }
      return true
    } catch (error) {
      if (!isCurrent()) return false
      if (isHomeAuthExpiredError(error)) {
        session.invalidateSession(requestRevision)
        loginPromptVisible.value = true
      }
      throw error
    } finally {
      if (voteOwner === owner && session.revision.value === requestRevision) {
        voteSubmittingTeamId.value = null
        voteOwner = null
      }
    }
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
    favoriteSubmitting,
    voteSubmittingTeamId,
    submitSupportVote,
    submitFavoriteTeam,
    loadSupportData,
    loadSupportTeams,
    resetSessionScopedState,
  }
}
