import { onScopeDispose, ref, watch } from 'vue'
import { getLineupCode } from '../../api/lineup'
import { materializePosterCode } from '../../utils/canvasPoster'

/**
 * 球场右下角的小程序码：已发布阵容才有 share_code，用它取该阵容的小程序码，
 * 用户截图分享时别人扫码直达这套阵容。取码失败只是不显示，不影响页面。
 */
export function useLineupPitchCode(shareCode: () => string | null | undefined) {
  const codeSrc = ref('')
  let cleanup: (() => void) | null = null
  let generation = 0

  function release(): void {
    cleanup?.()
    cleanup = null
    codeSrc.value = ''
  }

  watch(shareCode, async (code) => {
    const owner = ++generation
    release()
    if (!code) return
    try {
      const image = await getLineupCode(code)
      if (owner !== generation) return
      const materialized = await materializePosterCode(image)
      if (owner !== generation) {
        materialized.cleanup()
        return
      }
      cleanup = materialized.cleanup
      codeSrc.value = materialized.path
    } catch {
      // 未登录、取码失败时球场只保留文字水印
    }
  }, { immediate: true })

  onScopeDispose(() => {
    generation++
    release()
  })

  return { codeSrc }
}
