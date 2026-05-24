const AI_ENTRY_INTENT_KEY = 'fi_open_ai_chat_intent'

export function requestOpenAiChat(): void {
  try {
    uni.setStorageSync(AI_ENTRY_INTENT_KEY, '1')
  } catch {
    // Ignore storage failures; the home page still opens normally.
  }
}

export function consumeOpenAiChatIntent(): boolean {
  try {
    const value = uni.getStorageSync(AI_ENTRY_INTENT_KEY)
    if (value !== '1') {
      return false
    }
    uni.removeStorageSync(AI_ENTRY_INTENT_KEY)
    return true
  } catch {
    return false
  }
}
