import type { Ref } from 'vue'
import { canControlMiniReview } from '../../api/miniReview'
import { useAppSession, type AppSession } from '../../stores/appSession'
import type { MiniWechatBindingRequiredResponse } from '../../types/auth'

/** user 页会话私有状态（全部以 ref 注入，页面持有所有权）。 */
export interface UserAccountEffectsState {
  loading: Ref<boolean>
  isReviewController: Ref<boolean>
  miniWechatBindState: Ref<MiniWechatBindingRequiredResponse | null>
  passwordLoginSheetVisible: Ref<boolean>
  passwordLoginSubmitting: Ref<boolean>
}

export interface UserAccountEffectsDeps {
  canControlMiniReview: () => Promise<{ allowed: boolean }>
}

const defaultDeps: UserAccountEffectsDeps = {
  canControlMiniReview,
}

export interface UserAccountEffects {
  loadUser(): Promise<void>
  /** 只清旧账号的私有展示数据（白名单/loading），保留正在提交的认证表单。 */
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
 * user 页账号数据的读写入口。所有异步结果（白名单判定、账号恢复）
 * 都核对捕获的 revision，旧会话的请求在会话被替代后不写状态、不弹 toast；
 * 用户读取走共享 session.ensureSessionReady（401/断网语义由 session 统一处理）。
 */
export function createUserAccountEffects(
  session: AppSession,
  state: UserAccountEffectsState,
  deps: UserAccountEffectsDeps = defaultDeps,
): UserAccountEffects {
  // 页面 onShow 初始化与会话恢复通知可能同时触发 loadUser：同一会话（revision）的
  // 并发加载合并为一个请求，避免白名单接口被重复调用。
  let inFlightLoad: { revision: number; promise: Promise<void> } | null = null

  async function performLoadUser(): Promise<void> {
    const requestRevision = session.revision.value
    const isCurrent = () => session.revision.value === requestRevision

    state.loading.value = session.hasAccessToken.value

    // 设置入口的显隐不依赖审核模式：管理员在审核期间也需要进设置切换审核状态，
    // 因此只要有登录 token 就判一次白名单，认证流程不依赖审核展示状态。
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

    if (!session.hasAccessToken.value) {
      if (isCurrent()) {
        state.loading.value = false
      }
      return
    }

    // 用户读取走共享会话；401 / 断网语义由 session 统一处理。
    await Promise.allSettled([session.ensureSessionReady()])

    if (!isCurrent()) {
      return
    }

    state.loading.value = false
  }

  function resetPrivateViewState(): void {
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
