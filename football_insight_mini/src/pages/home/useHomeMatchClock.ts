import { onScopeDispose, ref, watch } from 'vue'
import { onHide, onShow, onUnload } from '@dcloudio/uni-app'

const SLOW_TICK_MS = 60_000
const FAST_TICK_MS = 1_000

/**
 * Refresh countdown dates while home is visible, and immediately after returning.
 * Ticks every second only while `fast()` is true (a second-level clock is on screen).
 * `fast` receives the current clock value and is read lazily, so it may reference state declared later.
 */
export function useHomeMatchClock(fast: (now: number) => boolean = () => false) {
  const now = ref(Date.now())
  let timer: ReturnType<typeof setInterval> | null = null
  let visible = false
  function stop() { if (timer !== null) { clearInterval(timer); timer = null } }
  function start() {
    stop()
    if (!visible) return
    now.value = Date.now()
    timer = setInterval(() => { now.value = Date.now() }, fast(now.value) ? FAST_TICK_MS : SLOW_TICK_MS)
  }
  onShow(() => { visible = true; start() })
  onHide(() => { visible = false; stop() })
  onUnload(() => { visible = false; stop() })
  onScopeDispose(() => { visible = false; stop() })
  // Switching between the second-level clock and the day count changes the tick rate.
  watch(() => fast(now.value), start)
  return now
}
