import { describe, expect, test } from 'bun:test'

import {
  buildPasswordLoginPayload,
  resolveUserLoginPresentation,
  validatePasswordLoginForm,
} from './loginHelpers'

describe('resolveUserLoginPresentation', () => {
  test('uses password login for H5', () => {
    expect(resolveUserLoginPresentation(true)).toEqual({
      method: 'password',
      actionText: '账号登录',
      switchAccountCaption: '如需切换账号，可退出后重新使用账号密码登录。',
    })
  })

  test('keeps WeChat login for Mini Program', () => {
    expect(resolveUserLoginPresentation(false)).toEqual({
      method: 'mini-wechat',
      actionText: '去登录',
      switchAccountCaption: '如需切换账号，可退出后重新使用微信登录。',
    })
  })
})

describe('password login form', () => {
  test('requires an account identifier', () => {
    expect(validatePasswordLoginForm({
      accountIdentifier: '   ',
      password: 'secret123',
    })).toBe('请输入账号')
  })

  test('requires a password', () => {
    expect(validatePasswordLoginForm({
      accountIdentifier: 'browser-tester',
      password: '   ',
    })).toBe('请输入密码')
  })

  test('builds the existing login API payload without changing the password', () => {
    expect(buildPasswordLoginPayload({
      accountIdentifier: ' browser-tester ',
      password: ' secret123 ',
    })).toEqual({
      account_identifier: 'browser-tester',
      password: ' secret123 ',
    })
  })
})
