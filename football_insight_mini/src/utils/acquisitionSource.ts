import type { AcquisitionSource } from '../types/auth'

interface SourceStorage {
  getStorageSync(key: string): unknown
  setStorageSync(key: string, value: unknown): void
}
const STORAGE_KEY = 'fi_first_acquisition_v1'
function object(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown> : {}
}
function label(value: unknown, max: number): string | undefined {
  return typeof value === 'string' && value.length <= max && /^[a-z0-9_-]+$/.test(value) ? value : undefined
}
function sanitize(value: unknown): AcquisitionSource | undefined {
  const input = object(value)
  const src = label(input.src, 32)
  if (!src) return undefined
  const cid = label(input.cid, 64)
  const scene = input.landing_scene
  return {
    src,
    ...(cid ? { cid } : {}),
    ...(typeof scene === 'number' && Number.isInteger(scene) && scene > 0 && scene <= 65535
      ? { landing_scene: scene } : {}),
  }
}
export function parseAcquisitionLaunch(options: unknown): AcquisitionSource | undefined {
  const launch = object(options)
  const query = object(launch.query)
  const direct = sanitize({ ...query, landing_scene: launch.scene })
  if (direct) return direct
  if (typeof query.scene !== 'string' || query.scene.length > 96) return undefined
  try {
    const decoded = decodeURIComponent(query.scene)
    if (decoded.length > 32) return undefined
    const fields: Record<string, string> = {}
    for (const pair of decoded.split('&')) {
      const parts = pair.split('=')
      const key = parts[0]!
      if (parts.length !== 2 || (key !== 's' && key !== 'c') || fields[key] !== undefined) return undefined
      fields[key] = parts[1]!
    }
    return sanitize({ src: fields.s, cid: fields.c, landing_scene: launch.scene })
  } catch {
    return undefined
  }
}
export function readAcquisitionSource(storage?: SourceStorage): AcquisitionSource | undefined {
  try {
    return sanitize((storage ?? uni).getStorageSync(STORAGE_KEY))
  } catch {
    console.warn('acquisition source storage unavailable')
    return undefined
  }
}
export function captureAcquisitionSource(options?: unknown, storage?: SourceStorage): void {
  try {
    if (readAcquisitionSource(storage)) return
    const source = parseAcquisitionLaunch(options ?? uni.getLaunchOptionsSync())
    if (source) (storage ?? uni).setStorageSync(STORAGE_KEY, source)
  } catch {
    console.warn('acquisition source capture unavailable')
  }
}
