import type { Ref } from 'vue'
import { getNotificationEmail, updateNotificationEmail } from '../../api/auth'
import { canControlMiniReview } from '../../api/miniReview'
import { useAppSession, type AppSession } from '../../stores/appSession'
import type { MiniWechatBindingRequiredResponse, NotificationEmailResponse } from '../../types/auth'
import { extractApiErrorMessage } from '../../utils/apiError'
import { loadSystemConfigUnderReview } from '../../utils/systemConfig'

/** user 页会话私有状态（全部以 ref 注入，页面持有所有权）。 */
export interface UserAccountEffectsState {
  loading: Ref<boolean>
  systemConfigUnderReview: Ref<boolean>
  isReviewController: Ref<boolean>
  notificationEmail: Ref<string>
  notificationEmailSheetVisible: Ref<boolean>
  notificationEmailSaving: Ref<boolean>
  miniWechatBindState: Ref<MiniWechatBindingRequiredResponse | null>
  passwordLoginSheetVisible: Ref<boolean>
  passwordLoginSubmitting: Ref<boolean>
}

export interface UserAccountEffectsDeps {
  canControlMiniReview: () => Promise<{ allowed: boolean }>
  getNotificationEmail: () => Promise<NotificationEmailResponse>
  updateNotificationEmail: (email: string) => Promise<NotificationEmailResponse>
  loadSystemConfigUnderReview: () => Promise<boolean>
  showToast: (title: string, icon: 'success' | 'none') => void
}

const defaultDeps: UserAccountEffectsDeps = {
  canControlMiniReview,
  getNotificationEmail,
  updateNotificationEmail,
  loadSystemConfigUnderReview,
  showToast: (title, icon) => uni.showToast({ title, icon }),
}

export interface UserAccountEffects {
  loadUser(): Promise<void>
  saveNotificationEmail(email: string): Promise<void>
  /** 只清旧账号的私有展示数据（邮箱/白名单/loading），保留正在提交的认证表单。 */
  resetPrivateViewState(): void
  /** 清认证表单上下文（密码弹层/绑定 token）；用于退出或被其他认证操作替代时。 */
  resetAuthFormState(): void
  /** 会话 revision 变化时调用：丢弃按旧会话展示的私有状态（展示数据 + 认证表单）。 */
  resetSessionScopedState(): void
  /**
   * 页面 watch(session.revision) 的处理入口：本页自己在途的认证提交只清展示数据、
   * 保留认证表单；退出/被其他操作替代则全量清理。
   */
  handleSessionRevisionChange(isOwnAuthSubmit: boolean): void
}

/**
 * user 页账号数据的读写入口。所有异步结果（白名单判定、邮箱读取/保存、finally）
 * 都核对捕获的 revision，旧会话的请求在会话被替代后不写状态、不弹 toast；
 * 用户读取走共享 session.ensureSessionReady（401/断网语义由 session 统一处理）。
 */
export function createUserAccountEffects(
  session: AppSession,
  state: UserAccountEffectsState,
  deps: UserAccountEffectsDeps = defaultDeps,
): UserAccountEffects {
  // 页面 onShow 初始化与会话恢复通知可能同时触发 loadUser：同一会话（revision）的
  // 并发加载合并为一个请求，避免白名单/邮箱接口被重复调用。
  let inFlightLoad: { revision: number; promise: Promise<void> } | null = null

  async function performLoadUser(): Promise<void> {
    const requestRevision = session.revision.value
    const isCurrent = () => session.revision.value === requestRevision

    state.loading.value = session.hasAccessToken.value

    // 设置入口的显隐不依赖审核模式：管理员在审核期间也需要进设置切换审核状态，
    // 因此只要有登录 token 就先判一次白名单，再走审核模式的提前返回。
    if (session.hasAccessToken.value) {
      try {
        const allowed = (await deps.canControlMiniReview()).allowed
        if (isCurrent()) {
          state.isReviewController.value = allowed
        }
      } catch {
        if (isCurrent()) {
          state.isReviewController.value = false
        }
      }
    } else {
      state.isReviewController.value = false
    }

    // 公共配置共享自 miniReview store，非会话私有，不做 revision 防护。
    state.systemConfigUnderReview.value = await deps.loadSystemConfigUnderReview()

    if (state.systemConfigUnderReview.value) {
      if (isCurrent()) {
        state.miniWechatBindState.value = null
        state.loading.value = false
      }
      return
    }

    if (!session.hasAccessToken.value) {
      if (isCurrent()) {
        state.notificationEmail.value = ''
        state.miniWechatBindState.value = null
        state.loading.value = false
      }
      return
    }

    const [, notificationEmailResult] = await Promise.allSettled([
      // 用户读取走共享会话：401 由 session 清会话，断网/5xx 由 session 保留旧态并记录 errorMessage。
      session.ensureSessionReady(),
      deps.getNotificationEmail(),
    ])

    if (!isCurrent()) {
      return
    }

    state.notificationEmail.value = notificationEmailResult.status === 'fulfilled'
      ? notificationEmailResult.value.email?.trim() || ''
      : ''
    state.loading.value = false
  }

  async function saveNotificationEmail(email: string): Promise<void> {
    const requestRevision = session.revision.value
    state.notificationEmailSaving.value = true
    try {
      const result = await deps.updateNotificationEmail(email)
      if (session.revision.value !== requestRevision) {
        return
      }
      state.notificationEmail.value = result.email?.trim() || email
      state.notificationEmailSheetVisible.value = false
      deps.showToast('邮箱已保存', 'success')
    } catch (error) {
      if (session.revision.value !== requestRevision) {
        return
      }
      deps.showToast(extractApiErrorMessage(error, '邮箱保存失败'), 'none')
    } finally {
      if (session.revision.value === requestRevision) {
        state.notificationEmailSaving.value = false
      }
    }
  }

  function resetPrivateViewState(): void {
    state.notificationEmail.value = ''
    state.notificationEmailSheetVisible.value = false
    state.notificationEmailSaving.value = false
    state.isReviewController.value = false
    state.loading.value = false
  }

  function resetAuthFormState(): void {
    state.miniWechatBindState.value = null
    state.passwordLoginSheetVisible.value = false
    state.passwordLoginSubmitting.value = false
  }

  function resetSessionScopedState(): void {
    resetPrivateViewState()
    resetAuthFormState()
  }

  function loadUser(): Promise<void> {
    const loadRevision = session.revision.value
    if (inFlightLoad && inFlightLoad.revision === loadRevision) {
      return inFlightLoad.promise
    }

    const promise = performLoadUser()
    inFlightLoad = { revision: loadRevision, promise }
    const clearIfCurrent = () => {
      if (inFlightLoad?.promise === promise) {
        inFlightLoad = null
      }
    }
    promise.then(clearIfCurrent, clearIfCurrent)
    return promise
  }

  function handleSessionRevisionChange(isOwnAuthSubmit: boolean): void {
    if (isOwnAuthSubmit) {
      resetPrivateViewState()
      return
    }
    resetSessionScopedState()
    // 后台静默恢复/其他登录发布后：页面已在前台时直接刷新私有区（游客态内部早退），
    // 无需再次切 Tab；与同 revision 的初始化/登录后加载合并在途请求。
    void loadUser()
  }

  return {
    loadUser,
    saveNotificationEmail,
    resetPrivateViewState,
    resetAuthFormState,
    resetSessionScopedState,
    handleSessionRevisionChange,
  }
}

/** 页面便捷入口：默认使用共享 session 单例。 */
export function useUserAccountEffects(state: UserAccountEffectsState): UserAccountEffects {
  return createUserAccountEffects(useAppSession(), state)
}
