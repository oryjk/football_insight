import { ref } from 'vue'

let nextInterceptId = 1
const interceptStack = ref<number[]>([])

export function allocateBackInterceptId(): number {
  const id = nextInterceptId
  nextInterceptId += 1
  return id
}

export function pushBackIntercept(id: number): void {
  const existingIndex = interceptStack.value.indexOf(id)
  if (existingIndex === -1) {
    interceptStack.value = [...interceptStack.value, id]
    return
  }
  if (existingIndex !== interceptStack.value.length - 1) {
    const next = interceptStack.value.filter((item) => item !== id)
    next.push(id)
    interceptStack.value = next
  }
}

export function removeBackIntercept(id: number): void {
  if (!interceptStack.value.includes(id)) return
  interceptStack.value = interceptStack.value.filter((item) => item !== id)
}

export function isTopBackIntercept(id: number): boolean {
  const stack = interceptStack.value
  return stack.length > 0 && stack[stack.length - 1] === id
}
