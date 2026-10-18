import { ref, type Ref } from 'vue'
import { getLiveTeamInsights } from '../../api/insight'
import type { TeamInsight, TeamInsightTeam, TeamInsightsResponse } from '../../types/insight'
import type { AppSession } from '../../stores/appSession'
import { extractApiErrorMessage } from '../../utils/apiError'

export interface InsightsDataDeps {
  session: AppSession
  /** 审核中不拉取会员洞察。 */
  isUnderReview: () => boolean
  /** 会员权益锁定（取关公众号）时不加载。 */
  isMembershipLocked: () => boolean
  /** 测试注入可控的私有读取；默认走真实 api 层。 */
  getLiveTeamInsights?: () => Promise<TeamInsightsResponse>
}

export interface InsightsData {
  userLoading: Ref<boolean>
  loading: Ref<boolean>
  errorMessage: Ref<string>
  teams: Ref<TeamInsightTeam[]>
  insights: Ref<TeamInsight[]>
  selectedTeamId: Ref<number | null>
  goalsForOpponentExpanded: Ref<boolean>
  goalsForPlayerExpanded: Ref<boolean>
  assistsForPlayerExpanded: Ref<boolean>
  goalsAgainstOpponentExpanded: Ref<boolean>
  /**
   * 确认当前用户。返回值表示本次调用跨 await 后是否仍属于原会话：
   * false = 等待期间会话身份已变化（如切账号/退出/会话失效），调用方必须停止
   * 后续阶段（不再 loadInsights），避免旧 onShow continuation 读取/遮盖新账号数据。
   */
  loadUser: () => Promise<boolean>
  loadInsights: () => Promise<void>
  resetExpandedState: () => void
  /** 会话身份变化（revision 递增）时同步丢弃旧账号私有缓存/选择/展开/错误/loading。 */
  resetSessionScopedState: () => void
  /** 审核中视图：停止 loading、清空会员数据、使在途读取 owner 失效（旧响应不再发布），不发起任何请求。 */
  applyUnderReviewView: () => void
}

/**
 * 洞察页会员私有数据的加载编排。
 *
 * - onShow（loadUser 之后）与 currentUser watcher 在同一会话（同 revision）触发的
 *   加载合并为一个私有请求，不会在 ensure 发布后重复请求；
 * - onShow 生命周期跨 await 保留身份：loadUser 返回 false（等待期间会话已被替代）
 *   时调用方停止后续阶段，旧 continuation 不再读取/遮盖新账号数据；
 * - success/catch/finally 都核对 owner + revision：旧账号的迟到成功/失败不发布、
 *   旧 finally 不清掉新请求的 loading；退出后迟到的 ensure 回调不再触发读取。
 */
export function useInsightsData(deps: InsightsDataDeps): InsightsData {
  const session = deps.session
  const fetchTeamInsights = deps.getLiveTeamInsights ?? getLiveTeamInsights

  const userLoading = ref(true)
  const loading = ref(false)
  const errorMessage = ref('')
  const teams = ref<TeamInsightTeam[]>([])
  const insights = ref<TeamInsight[]>([])
  const selectedTeamId = ref<number | null>(null)
  const goalsForOpponentExpanded = ref(false)
  const goalsForPlayerExpanded = ref(false)
  const assistsForPlayerExpanded = ref(false)
  const goalsAgainstOpponentExpanded = ref(false)

  let loadOwner: symbol | null = null
  let inFlightLoad: { revision: number; promise: Promise<void> } | null = null

  function resetExpandedState(): void {
    goalsForOpponentExpanded.value = false
    goalsForPlayerExpanded.value = false
    assistsForPlayerExpanded.value = false
    goalsAgainstOpponentExpanded.value = false
  }

  function resetSessionScopedState(): void {
    // 跨账号不保留旧缓存：列表/选中球队/展开状态/错误一并清掉；
    // 正常同账号刷新不触发本方法（revision 不变），缓存保留。
    teams.value = []
    insights.value = []
    selectedTeamId.value = null
    resetExpandedState()
    errorMessage.value = ''
    loading.value = false
    userLoading.value = false
  }

  function applyUnderReviewView(): void {
    // 进入审核视图：在途读取的 owner 一并失效，迟到的成功/失败/finally 不再发布。
    loadOwner = null
    inFlightLoad = null
    userLoading.value = false
    loading.value = false
    teams.value = []
    insights.value = []
    selectedTeamId.value = null
  }

  async function loadUser(): Promise<boolean> {
    const requestRevision = session.revision.value
    userLoading.value = true

    try {
      await session.ensureSessionReady()
    } catch {
      // 断网/5xx 由 session 保留 token 并记录可重试错误；页面沿用游客/锁定提示，不清会话。
      // 退出/替代后迟到的 ensure 回调（superseded）也走这里，不再读/发布任何状态。
    } finally {
      // 旧 revision 的 finally 不清掉新会话的确认中状态。
      if (session.revision.value === requestRevision) {
        userLoading.value = false
      }
    }

    // 跨 await 生命周期身份：等待期间会话身份已变化（切账号/退出/失效）时返回 false，
    // 调用方（onShow）必须停止后续 loadInsights，否则旧 continuation 会读取新账号、
    // 其失败还会遮盖新账号已加载成功的数据。
    return session.revision.value === requestRevision
  }

  async function runLoad(requestRevision: number, owner: symbol): Promise<void> {
    const isCurrent = () => loadOwner === owner && session.revision.value === requestRevision

    loading.value = true
    errorMessage.value = ''

    try {
      const response = await fetchTeamInsights()
      if (!isCurrent()) {
        return
      }
      teams.value = response.teams
      insights.value = response.insights
      if (!response.teams.some(team => team.team_id === selectedTeamId.value)) {
        selectedTeamId.value = response.teams[0]?.team_id ?? null
      }
      resetExpandedState()
    } catch (error) {
      if (!isCurrent()) {
        return
      }
      errorMessage.value = extractApiErrorMessage(error, '洞察加载失败，请稍后重试。')
    } finally {
      if (isCurrent()) {
        loading.value = false
      }
    }
  }

  function loadInsights(): Promise<void> {
    if (deps.isUnderReview()) {
      return Promise.resolve()
    }
    if (!session.currentUser.value || deps.isMembershipLocked()) {
      return Promise.resolve()
    }

    const requestRevision = session.revision.value
    // 同 revision 的在途加载合并：onShow 与 currentUser watcher 同会话只发一次私有请求；
    // 正常刷新（上一请求已结束）或新会话建立新 owner。
    if (inFlightLoad && inFlightLoad.revision === requestRevision) {
      return inFlightLoad.promise
    }

    const owner = Symbol('insights-load')
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
    userLoading,
    loading,
    errorMessage,
    teams,
    insights,
    selectedTeamId,
    goalsForOpponentExpanded,
    goalsForPlayerExpanded,
    assistsForPlayerExpanded,
    goalsAgainstOpponentExpanded,
    loadUser,
    loadInsights,
    resetExpandedState,
    resetSessionScopedState,
    applyUnderReviewView,
  }
}
