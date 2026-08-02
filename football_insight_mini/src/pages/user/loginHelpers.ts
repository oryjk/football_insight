import type { LoginPayload } from '../../types/auth'

export interface PasswordLoginForm {
  accountIdentifier: string
  password: string
}

export interface UserLoginPresentation {
  method: 'password' | 'mini-wechat'
  actionText: string
  switchAccountCaption: string
}

export function resolveUserLoginPresentation(isH5: boolean): UserLoginPresentation {
  if (isH5) {
    return {
      method: 'password',
      actionText: '账号登录',
      switchAccountCaption: '如需切换账号，可退出后重新使用账号密码登录。',
    }
  }

  return {
    method: 'mini-wechat',
    actionText: '去登录',
    switchAccountCaption: '如需切换账号，可退出后重新使用微信登录。',
  }
}

export function validatePasswordLoginForm(form: PasswordLoginForm): string | null {
  if (!form.accountIdentifier.trim()) {
    return '请输入账号'
  }

  if (!form.password.trim()) {
    return '请输入密码'
  }

  return null
}

export function buildPasswordLoginPayload(form: PasswordLoginForm): LoginPayload {
  return {
    account_identifier: form.accountIdentifier.trim(),
    password: form.password,
  }
}
