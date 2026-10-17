import { computed, readonly, ref, type Ref } from 'vue'
import * as miniReviewApi from '../api/miniReview'
import type { MiniReviewStatus } from '../types/system'
import { extractApiErrorMessage } from '../utils/apiError'

const MINI_REVIEW_CACHE_MS = 60_000

export interface MiniReviewStore {
  readonly status: Readonly<Ref<MiniReviewStatus | null>>
  readonly isUnderReview: Readonly<Ref<boolean>>
  /** 个人页等待审核查询期间的展示限制；开发构建始终不限制。 */
  readonly shouldRestrictFeatures: Readonly<Ref<boolean>>
  readonly loading: Readonly<Ref<boolean>>
  /** ready 区分「已尝试过」；available 区分「本次已获取到」。 */
  readonly ready: Readonly<Ref<boolean>>
  readonly available: Readonly<Ref<boolean>>
  readonly errorMessage: Readonly<Ref<string>>
  ensureMiniReviewReady(options?: { force?: boolean }): Promise<MiniReviewStatus | null>
  /**
   * 登录账号变化（登录/切换/退出）后调用：审核豁免按账号判定，必须用新身份重新查询。
   * 使在途的旧身份请求失效，不合并它的结果。
   */
  refreshForIdentityChange(): Promise<MiniReviewStatus | null>
  /** 管理员 PUT 成功后调用：立即发布新审核值，并使先前在途 GET 失效。 */
  setMiniReviewStatus(status: MiniReviewStatus): void
}

/**
 * 审核状态共享（project code football_insight_mini + 生成版本号，见 api/miniReview）：
 * 并发调用合并在途请求，成功值缓存 60 秒；查询失败返回 null（isUnderReview=false，
 * 兼容既有显隐基线），不永久缓存，下次 onShow/显式重试可重新请求。
 * 生产构建沿用审核失败时的业务显隐政策；开发构建保留真实 status 供设置页展示，
 * 但 isUnderReview / shouldRestrictFeatures 始终为 false。
 */
export function createMiniReviewStore(
  source: typeof miniReviewApi = miniReviewApi,
  options: { isDevelopment?: boolean } = {},
): MiniReviewStore {
  const isDevelopment = options.isDevelopment ?? Boolean(import.meta.env.DEV)
  const status = ref<MiniReviewStatus | null>(null)
  const loading = ref(false)
  const ready = ref(false)
  const available = ref(false)
  const errorMessage = ref('')
  // status 保留真实登记值，展示限制只在生产构建生效。
  const isUnderReview = computed(() => !isDevelopment && Boolean(status.value?.is_reviewing))
  const shouldRestrictFeatures = computed(() => !isDevelopment && (!ready.value || isUnderReview.value))

  let lastSuccessAt = 0
  // PUT 发布时递增，使先前在途 GET 的成功/失败都不能覆盖 PUT 结果。
  let requestEpoch = 0
  let inFlight: Promise<MiniReviewStatus | null> | null = null

  function publish(value: MiniReviewStatus): void {
    status.value = value
    available.value = true
    ready.value = true
    errorMessage.value = ''
    lastSuccessAt = Date.now()
  }

  function setMiniReviewStatus(value: MiniReviewStatus): void {
    requestEpoch += 1
    publish(value)
  }

  async function ensureMiniReviewReady(options?: { force?: boolean }): Promise<MiniReviewStatus | null> {
    const force = options?.force === true

    if (!force && available.value && status.value
      && Date.now() - lastSuccessAt < MINI_REVIEW_CACHE_MS) {
      return status.value
    }

    if (inFlight) {
      return inFlight
    }

    const epoch = requestEpoch
    loading.value = true
    let current: Promise<MiniReviewStatus | null> | null = null
    current = (async (): Promise<MiniReviewStatus | null> => {
      try {
        const result = await source.getMiniReviewStatus()
        if (epoch !== requestEpoch) {
          return status.value
        }
        publish(result)
        return result
      } catch (error) {
        if (epoch !== requestEpoch) {
          return status.value
        }
        lastSuccessAt = 0
        status.value = null
        available.value = false
        ready.value = true
        errorMessage.value = extractApiErrorMessage(error, '审核状态查询失败')
        return null
      } finally {
        // 身份变化后可能已有更新的在途请求，只清理自己登记的那一个。
        if (inFlight === current) {
          loading.value = false
          inFlight = null
        }
      }
    })()
    inFlight = current

    return current
  }

  function refreshForIdentityChange(): Promise<MiniReviewStatus | null> {
    requestEpoch += 1
    inFlight = null
    return ensureMiniReviewReady({ force: true })
  }

  return {
    status: readonly(status),
    isUnderReview,
    shouldRestrictFeatures,
    loading: readonly(loading),
    ready: readonly(ready),
    available: readonly(available),
    errorMessage: readonly(errorMessage),
    ensureMiniReviewReady,
    refreshForIdentityChange,
    setMiniReviewStatus,
  }
}

let singleton: MiniReviewStore | null = null

export function useMiniReviewStatus(): MiniReviewStore {
  if (!singleton) {
    singleton = createMiniReviewStore()
  }
  return singleton
}
