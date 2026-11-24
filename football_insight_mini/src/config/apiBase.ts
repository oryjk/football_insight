// 微信开发者工具和真机直连 local233 内网后端，使用具体 IP，避免指向客户端的回环地址。
// 开发地址可通过 VITE_API_BASE_URL 显式覆盖。
const DEVELOPMENT_API_BASE_URL = 'http://172.16.60.233:18092/api/v1'
const PRODUCTION_API_BASE_URL = 'https://match.oryjk.cn/api/v1'
const WEIXIN_PRODUCTION_API_BASE_URL = 'https://oryjk.cn:18443/football-api/v1'

function normalizeBaseUrl(value: unknown): string | null {
  if (typeof value !== 'string') {
    return null
  }

  const normalized = value.trim().replace(/\/+$/, '')
  return normalized.length ? normalized : null
}

function resolveApiBaseUrl(): string {
  const envBaseUrl = normalizeBaseUrl(import.meta.env.VITE_API_BASE_URL)
  const isProductionBuild = import.meta.env.PROD

  if (isProductionBuild) {
    // #ifdef MP-WEIXIN
    return normalizeBaseUrl(import.meta.env.VITE_MP_WEIXIN_API_BASE_URL)
      ?? WEIXIN_PRODUCTION_API_BASE_URL
    // #endif
    return envBaseUrl ?? PRODUCTION_API_BASE_URL
  }

  if (!envBaseUrl || envBaseUrl === PRODUCTION_API_BASE_URL) {
    return DEVELOPMENT_API_BASE_URL
  }

  return envBaseUrl
}

export const API_BASE_URL = resolveApiBaseUrl()

function resolveApiBackupBaseUrl(): string | null {
  // #ifdef MP-WEIXIN
  if (import.meta.env.PROD && API_BASE_URL === WEIXIN_PRODUCTION_API_BASE_URL) {
    return 'https://backup.oryjk.cn:19443/football-api/v1'
  }
  // #endif
  return null
}

export const API_BACKUP_BASE_URL = resolveApiBackupBaseUrl()

if (import.meta.env.DEV) {
  console.info(`[api-base] ${API_BASE_URL}`)
}
