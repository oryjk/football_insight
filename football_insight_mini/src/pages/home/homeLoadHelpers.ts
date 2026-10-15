import { isUnauthorizedError } from '../../utils/apiError'

export interface HomeLoadPlan {
  critical: string[]
  deferred: string[]
  onDemand: string[]
}

export function resolveHomeHasAuthToken(token: string | null): boolean {
  return typeof token === 'string' && token.trim().length > 0
}

export function shouldShowHomeSupportLoading(options: {
  loading: boolean
  hasCachedProfile: boolean
}): boolean {
  return options.loading && !options.hasCachedProfile
}

/** 认证过期判断依据 HTTP 401 状态码（对齐注册系统，不做文案匹配）。 */
export function isHomeAuthExpiredError(error: unknown): boolean {
  return isUnauthorizedError(error)
}

export function resolveHomeLoadPlan(hasAuthToken: boolean): HomeLoadPlan {
  return {
    critical: ['overview'],
    deferred: hasAuthToken
      ? ['rankings', 'matches', 'rounds', 'public-config', 'support-profile', 'support-teams']
      : ['rankings', 'matches', 'rounds', 'public-config'],
    onDemand: hasAuthToken ? ['auth-me'] : ['auth-me', 'support-teams'],
  }
}
