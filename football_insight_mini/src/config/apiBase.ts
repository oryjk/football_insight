const DEVELOPMENT_API_BASE_URL = 'http://172.16.60.233:18092/api/v1'
const PRODUCTION_API_BASE_URL = 'https://match.oryjk.cn/api/v1'

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
    return envBaseUrl ?? PRODUCTION_API_BASE_URL
  }

  if (!envBaseUrl || envBaseUrl === PRODUCTION_API_BASE_URL) {
    return DEVELOPMENT_API_BASE_URL
  }

  return envBaseUrl
}

export const API_BASE_URL = resolveApiBaseUrl()

if (import.meta.env.DEV) {
  console.info(`[api-base] ${API_BASE_URL}`)
}
