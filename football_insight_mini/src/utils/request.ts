import { getAccessToken } from './authStorage'
import { API_BASE_URL } from '../config/apiBase'
const REQUEST_TIMEOUT_MS = 20000
const GET_TIMEOUT_RETRY_COUNT = 1

type HttpMethod = 'GET' | 'POST' | 'PUT'

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

          reject(new Error(normalizeErrorMessage(response.data, statusCode)))
        },
        fail: (error) => {
          const errMsg = error.errMsg || '网络请求失败'
          const isTimeout = errMsg.toLowerCase().includes('timeout')

          if (isTimeout && method === 'GET' && retryCount < GET_TIMEOUT_RETRY_COUNT) {
            console.warn(`[request retry] ${options.url} timed out, retrying once`)
            attemptRequest(retryCount + 1)
            return
          }

          if (isTimeout) {
            reject(new Error(`请求超时：${options.url}`))
            return
          }

          reject(new Error(errMsg))
        },
      })
    }

    attemptRequest(0)
  })
}
