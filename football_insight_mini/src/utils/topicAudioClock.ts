/** Keep the requested position visible while a native seek is still reporting its old clock. */
export function createTopicAudioClock(now: () => number = Date.now) {
  let position = 0
  let pending: { target: number; elapsed: number; lastAt: number } | null = null
  let playing = false
  function settle() {
    if (!pending) return
    const at = now()
    if (playing) pending.elapsed += Math.max(0, at - pending.lastAt)
    pending.lastAt = at
  }
  function setPlaying(value: boolean) { settle(); playing = value }
  function reset(next = 0) { position = next; pending = null; playing = false }
  function seek(seconds: number, duration: number): number | null {
    if (!Number.isFinite(seconds) || !Number.isFinite(duration) || duration <= 0) return null
    const target = Math.min(duration, Math.max(0, seconds))
    pending = { target, elapsed: 0, lastAt: now() }
    position = target
    return target
  }
  function observe(seconds: number): number {
    if (!Number.isFinite(seconds) || seconds < 0) return position
    if (pending) {
      settle()
      const { target } = pending
      const elapsed = pending.elapsed / 1000
      // A valid report must be near the target or within the time actually spent playing since seeking.
      if (seconds < target - 0.5 || seconds > target + elapsed + 0.5) return position
      pending = null
    }
    position = seconds
    return position
  }
  return { reset, seek, observe, setPlaying }
}
