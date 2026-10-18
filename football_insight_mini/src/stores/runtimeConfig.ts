import { readonly, ref, type Ref } from 'vue'
import * as systemApi from '../api/system'
import type { PublicSystemConfig } from '../types/system'
import { extractApiErrorMessage } from '../utils/apiError'
import { applyBrandSubtitle } from '../utils/brandSubtitle'

const RUNTIME_CONFIG_CACHE_MS = 60_000

export interface RuntimeConfigStore {
  readonly config: Readonly<Ref<PublicSystemConfig | null>>
  readonly loading: Readonly<Ref<boolean>>
  /** ready 区分「已尝试过」；available 区分「本次已获取到」。 */
  readonly ready: Readonly<Ref<boolean>>
  readonly available: Readonly<Ref<boolean>>
  readonly errorMessage: Readonly<Ref<string>>
  ensureRuntimeConfig(options?: { force?: boolean }): Promise<PublicSystemConfig | null>
}

/**
 * 公共系统配置共享：并发调用合并在途请求，成功值缓存 60 秒；
 * 刷新失败保留最后一次成功 config，但 available/errorMessage 反映本次失败，
 * 不标成永久已加载（下次 onShow/显式重试会重新请求），也不新增常驻轮询。
 */
export function createRuntimeConfigStore(source: typeof systemApi = systemApi): RuntimeConfigStore {
  const config = ref<PublicSystemConfig | null>(null)
  const loading = ref(false)
  const ready = ref(false)
  const available = ref(false)
  const errorMessage = ref('')

  let lastSuccessAt = 0
  let inFlight: Promise<PublicSystemConfig | null> | null = null

  async function ensureRuntimeConfig(options?: { force?: boolean }): Promise<PublicSystemConfig | null> {
    const force = options?.force === true

    if (!force && available.value && config.value
      && Date.now() - lastSuccessAt < RUNTIME_CONFIG_CACHE_MS) {
      return config.value
    }

    // force 只绕过成功缓存，仍合并同类在途请求。
    if (inFlight) {
      return inFlight
    }

    loading.value = true
    inFlight = (async (): Promise<PublicSystemConfig | null> => {
      try {
        const result = await source.getPublicSystemConfig()
        config.value = result
        applyBrandSubtitle(result.brand_subtitle)
        available.value = true
        ready.value = true
        errorMessage.value = ''
        lastSuccessAt = Date.now()
        return result
      } catch (error) {
        lastSuccessAt = 0
        available.value = false
        ready.value = true
        errorMessage.value = extractApiErrorMessage(error, '公共配置加载失败')
        return null
      } finally {
        loading.value = false
        inFlight = null
      }
    })()

    return inFlight
  }

  return {
    // 含数组字段，不做 vue readonly() 深只读包装；仅在类型层面对外只读，写入收口在本模块内。
    config: config as Readonly<Ref<PublicSystemConfig | null>>,
    loading: readonly(loading),
    ready: readonly(ready),
    available: readonly(available),
    errorMessage: readonly(errorMessage),
    ensureRuntimeConfig,
  }
}

let singleton: RuntimeConfigStore | null = null

export function useRuntimeConfig(): RuntimeConfigStore {
  if (!singleton) {
    singleton = createRuntimeConfigStore()
  }
  return singleton
}
