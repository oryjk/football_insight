import type { PublicSystemConfig, SystemConfig } from '../types/system'
import { request } from '../utils/request'

function normalizeEnvValue(value: unknown): string {
  return typeof value === 'string' ? value.trim() : ''
}

const DEFAULT_MINI_PROGRAM_VERSION = import.meta.env.PROD ? '1.0.40' : '1.0.41'

export const MINI_PROGRAM_VERSION =
  normalizeEnvValue(import.meta.env.VITE_MINI_PROGRAM_VERSION) || DEFAULT_MINI_PROGRAM_VERSION
export const MINI_PROGRAM_APP_ID = normalizeEnvValue(import.meta.env.VITE_MINI_PROGRAM_APP_ID)

export function getPublicSystemConfig(): Promise<PublicSystemConfig> {
  return request<PublicSystemConfig>({ url: '/system/public-config' })
}

function encodeQueryValue(value: string): string {
  return encodeURIComponent(value).replace(/%20/g, '+')
}

export function buildSystemConfigUrl(version: string, appId?: string): string {
  const params = [`version=${encodeQueryValue(version.trim())}`]

  const normalizedAppId = appId?.trim()
  if (normalizedAppId) {
    params.push(`app_id=${encodeQueryValue(normalizedAppId)}`)
  }

  return `/system_config?${params.join('&')}`
}

export function getSystemConfig(
  version: string = MINI_PROGRAM_VERSION,
  appId: string = MINI_PROGRAM_APP_ID,
): Promise<SystemConfig> {
  return request<SystemConfig>({ url: buildSystemConfigUrl(version, appId) })
}
