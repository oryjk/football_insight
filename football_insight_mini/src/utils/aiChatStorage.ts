import type { AiChatMessage } from '../types/ai'

const AI_CHAT_HISTORY_STORAGE_PREFIX = 'fi_ai_chat_history'
const MAX_STORED_MESSAGES = 24

function getStorageKey(userId: string): string {
  return `${AI_CHAT_HISTORY_STORAGE_PREFIX}:${userId}`
}

export function getAiChatHistory(userId: string): AiChatMessage[] {
  const stored = uni.getStorageSync(getStorageKey(userId))
  if (!Array.isArray(stored)) {
    return []
  }

  return stored
    .filter((item): item is AiChatMessage => {
      if (!item || typeof item !== 'object') {
        return false
      }

      const record = item as Record<string, unknown>
      return (
        typeof record.id === 'string'
        && (record.role === 'user' || record.role === 'assistant')
        && typeof record.content === 'string'
        && typeof record.createdAt === 'string'
        && (record.kind === undefined || record.kind === 'text' || record.kind === 'image')
        && (record.imageUrl === undefined || typeof record.imageUrl === 'string')
      )
    })
    .slice(-MAX_STORED_MESSAGES)
}

export function setAiChatHistory(userId: string, messages: AiChatMessage[]): void {
  const sanitized = messages
    .filter((message) => message.content.trim().length > 0)
    .slice(-MAX_STORED_MESSAGES)

  uni.setStorageSync(getStorageKey(userId), sanitized)
}

export function clearAiChatHistory(userId: string): void {
  uni.removeStorageSync(getStorageKey(userId))
}
