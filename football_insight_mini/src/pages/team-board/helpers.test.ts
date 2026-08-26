import { describe, expect, test } from 'bun:test'
import { ApiRequestError } from '../../utils/apiError'
import { resolveTeamBoardLoadFailure } from './helpers'

const FALLBACK = '战术板加载失败，请稍后重试。'

describe('resolveTeamBoardLoadFailure', () => {
  test('401 无论文案如何都走登录引导', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('登录已过期', 401), FALLBACK)).toEqual({ kind: 'unauthorized' })
  })

  test('403 保留后端文案继续展示', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('无权限查看该球队', 403), FALLBACK)).toEqual({
      kind: 'error',
      message: '无权限查看该球队',
    })
  })

  test('500 空响应体时使用兜底文案', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('', 500), FALLBACK)).toEqual({
      kind: 'error',
      message: FALLBACK,
    })
  })

  test('非 401 且文案包含“未登录”不得当作登录失效（旧实现的误报场景）', () => {
    expect(resolveTeamBoardLoadFailure(new ApiRequestError('服务器开小差（未登录字样误报）', 500), FALLBACK)).toEqual({
      kind: 'error',
      message: '服务器开小差（未登录字样误报）',
    })
  })

  test('普通 Error 不按认证失效处理', () => {
    expect(resolveTeamBoardLoadFailure(new Error('未登录'), FALLBACK)).toEqual({
      kind: 'error',
      message: '未登录',
    })
  })
})
