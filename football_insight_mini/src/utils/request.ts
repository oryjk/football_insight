import { ApiRequestError } from './apiError'
import { getAccessToken } from './authStorage'
import { API_BASE_URL } from '../config/apiBase'
const REQUEST_TIMEOUT_MS = 20000
const GET_TIMEOUT_RETRY_COUNT = 1

const NETWORK_UNAVAILABLE_MESSAGE = '网络连接不可用，请检查网络后重试'
const NETWORK_TIMEOUT_MESSAGE = '网络连接超时，请稍后重试'

/** 断网/超时的 errMsg 形如 "request:fail " / "request:fail timeout"，统一转成用户能看懂的提示。 */
function normalizeNetworkFailureMessage(errMsg: string): string {
  return errMsg.includes('timeout') ? NETWORK_TIMEOUT_MESSAGE : NETWORK_UNAVAILABLE_MESSAGE
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
}

function joinUrl(path: string): string {
  if (path.startsWith('http')) {
    return path
  }

  return `${API_BASE_URL}${path}`
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

export function request<TResponse>(
  options: RequestOptions,
): Promise<TResponse> {
  const method = options.method ?? 'GET'

  return new Promise((resolve, reject) => {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }

    if (options.auth) {
      const token = getAccessToken()
      if (token) {
        headers.Authorization = `Bearer ${token}`
      }
    }

    const attemptRequest = (retryCount: number) => {
      uni.request({
        url: joinUrl(options.url),
        method,
        data: options.data as string | Record<string, unknown> | ArrayBuffer | undefined,
        timeout: REQUEST_TIMEOUT_MS,
        header: headers,
        success: (response) => {
          const statusCode = response.statusCode ?? 500
          if (statusCode >= 200 && statusCode < 300) {
            resolve(response.data as TResponse)
            return
          }

          reject(new ApiRequestError(normalizeErrorMessage(response.data, statusCode), statusCode, false, response.data))
        },
        fail: (error) => {
          const errMsg = error.errMsg || '网络请求失败'
          const isTimeout = errMsg.toLowerCase().includes('timeout')

          if (isTimeout && method === 'GET' && retryCount < GET_TIMEOUT_RETRY_COUNT) {
            console.warn(`[request retry] ${options.url} timed out, retrying once`)
            attemptRequest(retryCount + 1)
            return
          }

          const message = normalizeNetworkFailureMessage(errMsg)
          showNetworkUnavailableDialog(message)
          reject(new ApiRequestError(message, 0, true))
        },
      })
    }

    attemptRequest(0)
  })
}
