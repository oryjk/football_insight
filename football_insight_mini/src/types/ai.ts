export type AiChatRole = 'user' | 'assistant'

export interface AiChatMessage {
  id: string
  role: AiChatRole
  content: string
  createdAt: string
}

export interface AiChatHistoryMessagePayload {
  role: AiChatRole
  content: string
}

export interface AiChatStreamStartedEvent {
  model: string
}

export interface AiChatStreamDeltaEvent {
  content: string
}

export interface AiChatStreamDoneEvent {
  model: string
  reply: string
}
