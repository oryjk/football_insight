import { getAccessToken } from './auth'

export class ApiRequestError extends Error {
  statusCode: number

  constructor(message: string, statusCode: number) {
    super(message)
    this.name = 'ApiRequestError'
    this.statusCode = statusCode
  }
}

const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL || 'http://172.16.60.233:8092/api/v1').replace(/\/$/, '')

interface RequestOptions {
  url: string
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  data?: unknown
  auth?: boolean
}

export async function request<T>(options: RequestOptions): Promise<T> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (options.auth) {
    const token = getAccessToken()
    if (token) {
      headers.Authorization = `Bearer ${token}`
    }
  }

  let response: Response
  try {
    response = await fetch(`${API_BASE_URL}${options.url}`, {
      method: options.method || 'GET',
      headers,
      body: options.data === undefined ? undefined : JSON.stringify(options.data),
    })
  } catch {
    throw new ApiRequestError('网络连接不可用，请检查网络后重试', 0)
  }

  const text = await response.text()
  let body: unknown = null
  if (text) {
    try {
      body = JSON.parse(text)
    } catch {
      body = null
    }
  }

  if (!response.ok) {
    const message =
      (body && typeof body === 'object' && typeof (body as { message?: unknown }).message === 'string'
        ? (body as { message: string }).message
        : '') || `请求失败（${response.status}）`
    throw new ApiRequestError(message, response.status)
  }

  return body as T
}

export function extractApiErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof ApiRequestError && error.message) {
    return error.message
  }
  return fallback
}
