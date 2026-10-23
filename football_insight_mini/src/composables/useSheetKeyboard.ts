import { onBeforeUnmount, ref } from 'vue'

// 小程序 focus / keyboardheightchange 事件的 detail.height 是键盘高度（px）；
// 类型声明里 focus 被标成 DOM FocusEvent，这里按 unknown 收再安全取值。
function readKeyboardHeight(event: unknown): number {
  const height = (event as { detail?: { height?: unknown } } | undefined)?.detail?.height
  return typeof height === 'number' && height > 0 ? height : 0
}

/**
 * 底部弹层里的输入框配 :adjust-position="false" 时，按输入框自身事件跟踪键盘高度，
 * 传给 FiBottomSheet 的 keyboard-height 把面板抬到键盘上沿。
 *
 * 只靠 keyboardheightchange 不够：部分机型收起键盘时不回传高度 0，面板会停在高处，
 * 所以以 blur 为准落回；blur 延迟处理，在两个输入框之间切换焦点时面板不跟着跳。
 */
export function useSheetKeyboard() {
  const keyboardHeight = ref(0)
  let lastHeight = 0
  let blurTimer: ReturnType<typeof setTimeout> | null = null

  function clearBlurTimer(): void {
    if (blurTimer) {
      clearTimeout(blurTimer)
      blurTimer = null
    }
  }

  function applyHeight(height: number): void {
    const next = Math.round(height)
    keyboardHeight.value = next
    if (next > 0) lastHeight = next
  }

  function onInputFocus(event?: unknown): void {
    clearBlurTimer()
    const height = readKeyboardHeight(event)
    if (height > 0) {
      applyHeight(height)
    } else if (keyboardHeight.value === 0 && lastHeight > 0) {
      keyboardHeight.value = lastHeight
    }
  }

  function onKeyboardHeightChange(event?: unknown): void {
    applyHeight(readKeyboardHeight(event))
  }

  function onInputBlur(): void {
    clearBlurTimer()
    blurTimer = setTimeout(() => {
      blurTimer = null
      keyboardHeight.value = 0
    }, 80)
  }

  function resetKeyboard(): void {
    clearBlurTimer()
    keyboardHeight.value = 0
  }

  onBeforeUnmount(clearBlurTimer)

  return { keyboardHeight, onInputFocus, onInputBlur, onKeyboardHeightChange, resetKeyboard }
}
