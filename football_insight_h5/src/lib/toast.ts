import { reactive } from 'vue'

interface ToastState {
  visible: boolean
  message: string
}

export const toastState = reactive<ToastState>({ visible: false, message: '' })

let timer: ReturnType<typeof setTimeout> | undefined

export function showToast(message: string, duration = 2200): void {
  toastState.message = message
  toastState.visible = true
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => {
    toastState.visible = false
  }, duration)
}
