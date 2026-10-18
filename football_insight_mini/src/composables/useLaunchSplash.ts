import { onUnmounted, ref, watch, type Ref } from 'vue'
import { SPLASH_FADE_MS, decideSplash } from '../utils/launchSplash'

const TICK_MS = 50
// 启动幕只属于一次冷启动：本进程内首页再次创建（返回、切 tab）不再出现。
let consumed = false

export type SplashPhase = 'idle' | 'visible' | 'leaving' | 'done'

export function useLaunchSplash(settled: Ref<boolean>) {
  const phase = ref<SplashPhase>(consumed ? 'done' : 'idle')
  if (consumed) return { phase }
  consumed = true

  const startedAt = Date.now()
  let shownAt: number | null = null
  let tick: ReturnType<typeof setInterval> | undefined
  let fade: ReturnType<typeof setTimeout> | undefined

  function stop() {
    if (tick !== undefined) clearInterval(tick)
    tick = undefined
  }
  function finish(decision: 'skip' | 'hide') {
    stop()
    if (decision === 'skip') {
      phase.value = 'done'
      return
    }
    phase.value = 'leaving'
    fade = setTimeout(() => { phase.value = 'done' }, SPLASH_FADE_MS)
  }
  function step() {
    const now = Date.now()
    const decision = decideSplash({
      elapsedMs: now - startedAt,
      settled: settled.value,
      visibleForMs: shownAt === null ? null : now - shownAt,
    })
    if (decision === 'show') {
      shownAt = now
      phase.value = 'visible'
    } else if (decision === 'skip' || decision === 'hide') {
      finish(decision)
    }
  }

  tick = setInterval(step, TICK_MS)
  // 数据落定时立即评估一次，免得等下一个 tick。
  watch(settled, step)
  onUnmounted(() => {
    stop()
    if (fade !== undefined) clearTimeout(fade)
  })
  return { phase }
}
