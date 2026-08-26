/** 带 HTTP 状态码的请求错误：认证/重试等判断依据状态码，而不是匹配错误文案。 */
export class ApiError extends Error {
  readonly statusCode: number
  readonly body: unknown

  constructor(message: string, statusCode: number, body: unknown = null) {
    super(message)
    this.name = 'ApiError'
    this.statusCode = statusCode
    this.body = body
  }

  get isUnauthorized(): boolean {
    return this.statusCode === 401
  }
}

export function isApiError(value: unknown): value is ApiError {
  return value instanceof ApiError
}

export function extractApiErrorMessage(error: unknown, fallback: string): string {
  if (typeof error === 'string' && error.trim()) {
    return error
  }

  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message.trim()) {
      return message
    }
  }

  return fallback
}
