import { onBeforeUnmount, ref, watch } from 'vue'
import { buildIntegerSequence } from '../utils/animatedInteger'

export function useAnimatedInteger(source: () => number, stepDelayMs = 72) {
  const value = ref(0)
  let timer: ReturnType<typeof setTimeout> | null = null

  function clearTimer(): void {
    if (timer === null) {
      return
    }

    clearTimeout(timer)
    timer = null
  }

  function animateTo(target: number): void {
    clearTimer()

    const sequence = buildIntegerSequence(value.value, target)
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

      timer = setTimeout(tick, stepDelayMs)
    }

    tick()
  }

  watch(
    source,
    (target) => {
      animateTo(target)
    },
    { immediate: true },
  )

  onBeforeUnmount(() => {
    clearTimer()
  })

  return value
}
