import { onBeforeUnmount, ref, watch } from 'vue'
import type { WatchSource } from 'vue'
import { buildIntegerSequence } from '../utils/animatedInteger'

export interface AnimatedIntegerOptions {
  /** true：首次拿到值时从 0 滚动到目标；false（默认）：首次直接显示目标值。 */
  initialFromZero?: boolean
  /** 相邻两帧的间隔，约 32ms 一帧（小程序逻辑层无 rAF，用 setTimeout 驱动）。 */
  frameDelayMs?: number
}

const DEFAULT_FRAME_DELAY_MS = 32

export function useAnimatedInteger(source: WatchSource<number>, options: AnimatedIntegerOptions = {}) {
  const { initialFromZero = false, frameDelayMs = DEFAULT_FRAME_DELAY_MS } = options
  const value = ref(0)
  let timer: ReturnType<typeof setTimeout> | null = null
  let started = false

  function clearTimer(): void {
    if (timer === null) {
      return
    }

    clearTimeout(timer)
    timer = null
  }

  function animateFrom(from: number, target: number): void {
    clearTimer()

    const sequence = buildIntegerSequence(from, target)
    if (sequence.length <= 1) {
      value.value = target
      return
    }

    let index = 1

    const tick = (): void => {
      value.value = sequence[index]
      index += 1

      if (index >= sequence.length) {
        timer = null
        return
      }

      timer = setTimeout(tick, frameDelayMs)
    }

    timer = setTimeout(tick, frameDelayMs)
  }

  watch(
    source,
    (target) => {
      if (!started) {
        started = true

        if (initialFromZero) {
          value.value = 0
          animateFrom(0, target)
        } else {
          value.value = target
        }

        return
      }

      // 中途被打断时从「当前正在显示的值」继续，不回跳、不叠定时器。
      animateFrom(value.value, target)
    },
    { immediate: true },
  )

  onBeforeUnmount(() => {
    clearTimer()
  })

  return value
}
