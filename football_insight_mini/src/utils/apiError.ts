/** 请求错误类型，对齐 registration_system_mini 的实现：
 * statusCode 区分业务/认证错误（401 => isUnauthorizedError），networkFailed 区分断网/超时；
 * 认证失效的清 token / 登录引导由消费方决定，请求层不做全局导航。
 */
export class ApiRequestError extends Error {
  readonly statusCode: number
  readonly networkFailed: boolean
  /** 后端原始响应体（扁平 JSON 或纯文本），供需要细节的消费方使用。 */
  readonly body: unknown

  constructor(message: string, statusCode = 0, networkFailed = false, body: unknown = null) {
    super(message)
    this.name = 'ApiRequestError'
    this.statusCode = statusCode
    this.networkFailed = networkFailed
    this.body = body
  }
}

export function isUnauthorizedError(error: unknown): boolean {
  return error instanceof ApiRequestError && error.statusCode === 401
}

export function isNetworkUnavailableError(error: unknown): boolean {
  return error instanceof ApiRequestError && error.networkFailed
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
