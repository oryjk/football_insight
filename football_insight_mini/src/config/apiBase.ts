const LOCAL_API_BASE_URL = 'http://127.0.0.1:8092/api/v1'
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
    return LOCAL_API_BASE_URL
  }

  return envBaseUrl
}

export const API_BASE_URL = resolveApiBaseUrl()

if (import.meta.env.DEV) {
  console.info(`[api-base] ${API_BASE_URL}`)
}
