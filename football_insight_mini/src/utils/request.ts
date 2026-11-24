import { ApiRequestError } from './apiError'
import { getAccessToken } from './authStorage'
import { API_BACKUP_BASE_URL, API_BASE_URL } from '../config/apiBase'

const REQUEST_TIMEOUT_MS = 20000
const PRIMARY_PROBE_COOLDOWN_MS = 60_000
const AUTH_ROUTE_PROBE_TIMEOUT_MS = 3_000

const NETWORK_UNAVAILABLE_MESSAGE = '网络连接不可用，请检查网络后重试'
const NETWORK_TIMEOUT_MESSAGE = '网络连接超时，请稍后重试'

/** 断网/超时的 errMsg 形如 "request:fail " / "request:fail timeout"，统一转成用户能看懂的提示。 */
function normalizeNetworkFailureMessage(errMsg: string): string {
  return errMsg.toLowerCase().includes('timeout') ? NETWORK_TIMEOUT_MESSAGE : NETWORK_UNAVAILABLE_MESSAGE
}

/** 断网时并发请求会一起失败，全局只弹一个提示框，关闭后再次失败才重新弹。 */
let networkErrorDialogVisible = false

function showNetworkUnavailableDialog(content: string) {
  if (networkErrorDialogVisible) return
  networkErrorDialogVisible = true
  uni.showModal({
    title: '网络不可用',
    content,
    showCancel: false,
    confirmText: '知道了',
    complete: () => {
      networkErrorDialogVisible = false
    },
  })
}

type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE'

type RequestBody = object | string | ArrayBuffer | undefined

interface RequestOptions {
  url: string
  method?: HttpMethod
  data?: RequestBody
  auth?: boolean
  headers?: Record<string, string>
  /** Background statistics must not interrupt reading with a network dialog. */
  silentNetworkError?: boolean
  /** 排除有写入副作用的 GET，不用它探测或切换生产线路。 */
  allowFailover?: boolean
}

function joinUrl(base: string, path: string): string {
  if (path.startsWith('http')) {
    return path
  }

  return `${base}${path}`
}

function isGatewayFailure(statusCode: number): boolean {
  return statusCode === 502 || statusCode === 503 || statusCode === 504
}

function normalizeErrorMessage(data: unknown, statusCode: number): string {
  if (typeof data === 'string' && data.trim()) {
    return data
  }

  if (data && typeof data === 'object') {
    const record = data as Record<string, unknown>
    for (const key of ['message', 'error', 'detail']) {
      const value = record[key]
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }

  return `请求失败（${statusCode}）`
}

export function createApiRequester(primary: string, backup: string | null, now = Date.now) {
  let activeBase = primary
  let routeRevision = 0
  let routeConfirmed = false
  let nextPrimaryProbeAt = 0
  let authRouteProbe: Promise<void> | null = null

  function sendRequest<TResponse>(options: RequestOptions, timeout = REQUEST_TIMEOUT_MS): Promise<TResponse> {
    const method = options.method ?? 'GET'
    // 仅路由本项目的相对 API 路径，完整 URL 保持调用方指定的地址。
    const canRoute = Boolean(backup) && /^\/(?![\\/])/.test(options.url)
    const canRetry = method === 'GET' && (!canRoute || options.allowFailover !== false)
    const alternate = canRoute && canRetry ? backup : null
    const revision = routeRevision
    let base = canRoute ? activeBase : primary

    if (alternate && base === backup && now() >= nextPrimaryProbeAt) {
      // 并发读请求中，只有首个请求在冷却期结束时试探主线。
      base = primary
      nextPrimaryProbeAt = now() + PRIMARY_PROBE_COOLDOWN_MS
    }

    return new Promise((resolve, reject) => {
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...options.headers,
      }
      if (options.auth) {
        const token = getAccessToken()
        if (token) headers.Authorization = `Bearer ${token}`
      }

      function attemptRequest(attemptBase: string, firstFailure?: ApiRequestError) {
        function finishFailure(error: ApiRequestError, timedOut = false) {
          if (canRoute && routeRevision === revision && attemptBase === activeBase) {
            routeConfirmed = false
          }

          if (alternate && !firstFailure) {
            attemptRequest(attemptBase === primary ? alternate : primary, error)
            return
          }
          if (!alternate && canRetry && timedOut && !firstFailure) {
            console.warn(`[request retry] ${options.url} timed out, retrying once`)
            attemptRequest(attemptBase, error)
            return
          }

          const finalError = (alternate && firstFailure) || error
          if (finalError.networkFailed && !options.silentNetworkError) {
            showNetworkUnavailableDialog(finalError.message)
          }
          reject(finalError)
        }

        uni.request({
          url: joinUrl(attemptBase, options.url),
          method,
          data: options.data as string | Record<string, unknown> | ArrayBuffer | undefined,
          timeout,
          header: headers,
          success: (response) => {
            const statusCode = response.statusCode ?? 500
            const success = statusCode >= 200 && statusCode < 300
            if (canRoute && routeRevision === revision && !isGatewayFailure(statusCode)) {
              // 只在成功时切换；此前发起的请求不能覆盖较新的选路结果。
              if (success && attemptBase !== activeBase) {
                activeBase = attemptBase
                routeRevision += 1
                if (activeBase === backup) nextPrimaryProbeAt = now() + PRIMARY_PROBE_COOLDOWN_MS
              }
              if (attemptBase === activeBase) routeConfirmed = true
            }
            if (success) {
              resolve(response.data as TResponse)
              return
            }

            const error = new ApiRequestError(normalizeErrorMessage(response.data, statusCode), statusCode, false, response.data)
            if (isGatewayFailure(statusCode)) {
              finishFailure(error)
              return
            }
            reject(error)
          },
          fail: (error) => {
            const errMsg = error.errMsg || '网络请求失败'
            const isTimeout = errMsg.toLowerCase().includes('timeout')
            finishFailure(new ApiRequestError(normalizeNetworkFailureMessage(errMsg), 0, true), isTimeout)
          },
        })
      }

      attemptRequest(base)
    })
  }

  function prepareApiRoute(): Promise<void> | undefined {
    if (!backup || routeConfirmed) return
    if (!authRouteProbe) {
      authRouteProbe = sendRequest({
        url: '/system/public-config',
        silentNetworkError: true,
      }, AUTH_ROUTE_PROBE_TIMEOUT_MS)
        .then(() => undefined, () => undefined)
        .finally(() => { authRouteProbe = null })
    }
    return authRouteProbe
  }

  return {
    request: <TResponse>(options: RequestOptions) => sendRequest<TResponse>(options),
    prepareApiRoute,
  }
}

export const { request, prepareApiRoute } = createApiRequester(API_BASE_URL, API_BACKUP_BASE_URL)
