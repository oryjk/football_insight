import type { Ref } from 'vue'
import { isSessionSupersededError, useAppSession, type AppSession } from '../../stores/appSession'
import type {
  H5TestLoginUser,
  MiniWechatBindingRequiredResponse,
} from '../../types/auth'
import { extractApiErrorMessage } from '../../utils/apiError'
import { consumePostLoginRedirect, navigateToPostLoginTarget } from '../../utils/postLoginRedirect'
import { buildPasswordLoginPayload } from './loginHelpers'

/** 登录/绑定/退出编排涉及的页面状态（全部以 ref 注入，页面持有所有权）。 */
export interface UserLoginFlowState {
  passwordLoginSheetVisible: Ref<boolean>
  passwordLoginSubmitting: Ref<boolean>
  miniWechatBindState: Ref<MiniWechatBindingRequiredResponse | null>
}

export interface UserLoginFlowDeps {
  redirectAfterLogin: () => Promise<void>
  showToast: (title: string, icon: 'success' | 'none') => void
  getMiniWechatCode: () => Promise<string>
}

const defaultDeps: UserLoginFlowDeps = {
  redirectAfterLogin: async () => {
    const target = consumePostLoginRedirect()
    if (!target || target.url === '/pages/user/index') {
      return
    }

    try {
      await navigateToPostLoginTarget(target)
    } catch (error) {
      uni.showToast({ title: extractApiErrorMessage(error, '登录后跳转失败'), icon: 'none' })
    }
  },
  showToast: (title, icon) => uni.showToast({ title, icon }),
  getMiniWechatCode: () => new Promise((resolve, reject) => {
    uni.login({
      success: (result) => {
        if (!result.code) {
          reject(new Error('微信登录 code 获取失败'))
          return
        }

        resolve(result.code)
      },
      fail: (error) => {
        reject(new Error(error.errMsg || '微信登录失败'))
      },
    })
  }),
}

export interface MiniWechatBindForm {
  bindToken: string
  inviteCode: string
  referralCode: string
  displayName: string
  avatarDataUrl: string
}

export interface UserLoginFlow {
  submitPasswordLogin(form: { accountIdentifier: string; password: string }): Promise<void>
  loginWithMiniWechat(): Promise<void>
  bindMiniWechatAccount(payload: MiniWechatBindForm): Promise<void>
  loginAsH5TestUser(user: H5TestLoginUser): Promise<void>
  logout(): Promise<void>
  /**
   * revision 监听用：该 revision 是否由本 flow 的在途认证提交产生。
   * 自己发起的提交（beginAuthIntent 同步递增）不该清掉正在填写的认证表单；
   * 退出/被其他操作替代时不匹配，仍全量清理。
   */
  isOwnAuthSubmit(revision: number): boolean
}

/**
 * user 页登录/绑定/退出的 handler 编排。每个操作在 action 返回后、loadUser 完成后、
 * toast/跳转执行前都核对 revision：被更新操作（退出/其他登录）替代的旧操作
 * 不提示成功、不跳转、不改新操作的状态；同 revision 的失败保留表单上下文以便重试。
 */
export function createUserLoginFlow(
  session: AppSession,
  state: UserLoginFlowState,
  loadUser: () => Promise<void>,
  deps: UserLoginFlowDeps = defaultDeps,
): UserLoginFlow {
  /** 本 flow 发起的在途认证提交的 intentRevision 集合。 */
  const pendingSubmitRevisions = new Set<number>()

  function trackSubmit(): { intentRevision: number; done: () => void } {
    // session 的认证 action 在调用瞬间同步 beginAuthIntent（revision + 1）。
    const intentRevision = session.revision.value
    pendingSubmitRevisions.add(intentRevision)
    return {
      intentRevision,
      done: () => {
        pendingSubmitRevisions.delete(intentRevision)
      },
    }
  }

  function isOwnAuthSubmit(revision: number): boolean {
    return pendingSubmitRevisions.has(revision)
  }

  /** 成功路径的后续编排：loadUser → revision 校验 → toast → 跳转。 */
  async function settleSuccess(
    settledRevision: number,
    toastTitle: string,
    options: { redirect: boolean },
  ): Promise<void> {
    await loadUser()
    if (session.revision.value !== settledRevision) {
      // loadUser 期间被退出/其他登录替代：不提示成功也不跳转。
      return
    }
    deps.showToast(toastTitle, 'success')
    if (options.redirect) {
      await deps.redirectAfterLogin()
    }
  }

  /** 失败路径：superseded/被替代静默放弃；同 revision 的错误才提示。 */
  function settleFailure(
    intentRevision: number,
    error: unknown,
    fallback: string,
  ): boolean {
    if (isSessionSupersededError(error)) {
      return false
    }
    if (session.revision.value !== intentRevision) {
      return false
    }
    deps.showToast(extractApiErrorMessage(error, fallback), 'none')
    return true
  }

  async function submitPasswordLogin(form: { accountIdentifier: string; password: string }): Promise<void> {
    state.passwordLoginSubmitting.value = true
    const action = session.loginWithPassword(buildPasswordLoginPayload(form))
    const submit = trackSubmit()
    try {
      // token/user 由 session 发布，本页通过共享只读状态自动刷新。
      await action
      const settledRevision = session.revision.value
      state.passwordLoginSheetVisible.value = false
      state.passwordLoginSubmitting.value = false
      await settleSuccess(settledRevision, '登录成功', { redirect: true })
    } catch (error) {
      if (settleFailure(submit.intentRevision, error, '登录失败')) {
        // 同 revision 失败：保留弹层与表单内容，允许原地重试。
        state.passwordLoginSubmitting.value = false
      }
    } finally {
      submit.done()
    }
  }

  async function loginWithMiniWechat(): Promise<void> {
    let code: string
    try {
      code = await deps.getMiniWechatCode()
    } catch (error) {
      deps.showToast(extractApiErrorMessage(error, '微信登录失败'), 'none')
      return
    }

    const action = session.loginWithWechatCode(code)
    const submit = trackSubmit()
    try {
      const result = await action
      const settledRevision = session.revision.value
      if (result.status === 'authenticated') {
        // 登录态由 session 同步发布，isGuestPage 基于共享只读状态自动切到已登录渲染。
        await settleSuccess(settledRevision, '微信登录成功', { redirect: true })
        return
      }

      // binding_required：不发布 token，绑定表单状态留在页面内。
      state.miniWechatBindState.value = result
    } catch (error) {
      settleFailure(submit.intentRevision, error, '微信登录失败')
    } finally {
      submit.done()
    }
  }

  async function bindMiniWechatAccount(payload: MiniWechatBindForm): Promise<void> {
    const action = session.bindWechatAccount({
      bind_token: payload.bindToken,
      invite_code: payload.inviteCode || undefined,
      referral_code: payload.referralCode || undefined,
      display_name: payload.displayName,
      avatar_data_url: payload.avatarDataUrl,
    })
    const submit = trackSubmit()
    try {
      await action
      const settledRevision = session.revision.value
      state.miniWechatBindState.value = null
      await settleSuccess(settledRevision, '微信绑定成功', { redirect: true })
    } catch (error) {
      // 同 revision 失败（邀请码无效/网络错误）：保留绑定表单与 bind_token，允许修改后重试。
      settleFailure(submit.intentRevision, error, '绑定失败')
    } finally {
      submit.done()
    }
  }

  async function loginAsH5TestUser(user: H5TestLoginUser): Promise<void> {
    const action = session.loginAsTestUser(user.id)
    const submit = trackSubmit()
    try {
      await action
      const settledRevision = session.revision.value
      await settleSuccess(
        settledRevision,
        `已切换到 ${user.display_name?.trim() || user.account_identifier}`,
        { redirect: false },
      )
    } catch (error) {
      settleFailure(submit.intentRevision, error, '测试登录失败')
    } finally {
      submit.done()
    }
  }

  async function logout(): Promise<void> {
    const action = session.logoutSession()
    // logoutSession 同步递增 revision 并清本地会话。
    const logoutRevision = session.revision.value
    await action
    await loadUser()
    if (session.revision.value !== logoutRevision) {
      // 退出等待期间已有新登录：不提示「已退出登录」。
      return
    }
    deps.showToast('已退出登录', 'success')
  }

  return {
    submitPasswordLogin,
    loginWithMiniWechat,
    bindMiniWechatAccount,
    loginAsH5TestUser,
    logout,
    isOwnAuthSubmit,
  }
}
