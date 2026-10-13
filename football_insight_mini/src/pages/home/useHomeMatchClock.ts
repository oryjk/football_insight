import { onScopeDispose, ref } from 'vue'
import { onHide, onShow, onUnload } from '@dcloudio/uni-app'

/** Refresh countdown dates while home is visible, and immediately after returning. */
export function useHomeMatchClock() {
  const now = ref(Date.now())
  let timer: ReturnType<typeof setInterval> | null = null
  function stop() { if (timer !== null) { clearInterval(timer); timer = null } }
  onShow(() => {
    stop()
    now.value = Date.now()
    timer = setInterval(() => { now.value = Date.now() }, 60_000)
  })
  onHide(stop)
  onUnload(stop)
  onScopeDispose(stop)
  return now
}
