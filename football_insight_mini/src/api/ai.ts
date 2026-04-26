import type {
  AiChatHistoryMessagePayload,
  AiChatStreamDeltaEvent,
  AiChatStreamDoneEvent,
  AiChatStreamStartedEvent,
} from '../types/ai'
import type { AiChatMode } from '../types/system'
import { API_BASE_URL } from '../config/apiBase'
import {
  WECHAT_CLOUD_IMAGE_FUNCTION_NAME,
  WECHAT_CLOUD_AI_MODEL,
  WECHAT_CLOUD_AI_PROVIDER,
  buildWechatCloudMessages,
  resolveAiChatMode,
} from '../config/aiChat'
import { getAccessToken } from '../utils/authStorage'
import { callWechatCloudFunction, getWechatCloudAiExtension } from '../utils/wechatCloud'
const STREAM_TIMEOUT_MS = 120000

interface StreamAiChatPayload {
  message: string
  history: AiChatHistoryMessagePayload[]
}

interface StreamAiChatCallbacks {
  onStarted?: (event: AiChatStreamStartedEvent) => void
  onDelta: (event: AiChatStreamDeltaEvent) => void
  onDone: (event: AiChatStreamDoneEvent) => void
  onError: (message: string) => void
}

export interface AiChatStreamHandle {
  abort: () => void
}

export interface GenerateAiImageResult {
  imageUrl: string
  revisedPrompt: string
}

interface StreamAiChatOptions {
  mode?: AiChatMode
}

interface ChunkedRequestTask {
  abort: () => void
  onChunkReceived?: (callback: (chunk: { data: ArrayBuffer }) => void) => void
}

function decodeChunk(data: ArrayBuffer): string {
  if (typeof TextDecoder !== 'undefined') {
    return new TextDecoder('utf-8').decode(data)
  }

  return String.fromCharCode(...new Uint8Array(data))
}

function normalizeErrorMessage(data: unknown, statusCode: number): string {
  if (typeof data === 'string' && data.trim()) {
    return data
  }

  if (data && typeof data === 'object') {
    const record = data as Record<string, unknown>
    for (const key of ['message', 'error', 'detail']) {
      const value = record[key]
      if (typeof value === 'string' && value.trim()) {
        return value
      }
    }
  }

  return `请求失败（${statusCode}）`
}

function parseServerEvent(rawEvent: string): { event: string; data: string } | null {
  const lines = rawEvent
    .split('\n')
    .map((line) => line.trimEnd())
    .filter((line) => line.length > 0)

  if (!lines.length) {
    return null
  }

  let event = 'message'
  const dataLines: string[] = []

  for (const line of lines) {
    if (line.startsWith('event:')) {
      event = line.slice(6).trim()
      continue
    }

    if (line.startsWith('data:')) {
      dataLines.push(line.slice(5).trim())
    }
  }

  return {
    event,
    data: dataLines.join('\n'),
  }
}

export function streamAiChat(
  payload: StreamAiChatPayload,
  callbacks: StreamAiChatCallbacks,
  options: StreamAiChatOptions = {},
): AiChatStreamHandle {
  const mode = resolveAiChatMode(options.mode)

  if (mode === 'frontend_direct') {
    return streamWechatCloudAiChat(payload, callbacks)
  }

  return streamBackendAiChat(payload, callbacks)
}

function streamBackendAiChat(
  payload: StreamAiChatPayload,
  callbacks: StreamAiChatCallbacks,
): AiChatStreamHandle {
  const token = getAccessToken()
  if (!token) {
    callbacks.onError('请先登录')
    return { abort: () => {} }
  }

  let buffer = ''
  let isAborted = false
  let hasCompleted = false

  const requestTask = uni.request({
    url: `${API_BASE_URL}/ai/chat/stream`,
    method: 'POST',
    timeout: STREAM_TIMEOUT_MS,
    enableChunked: true,
    responseType: 'arraybuffer',
    header: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
      Accept: 'text/event-stream',
    },
    data: payload,
    success: (response: { statusCode?: number; data: unknown }) => {
      const statusCode = response.statusCode ?? 500

      if (statusCode < 200 || statusCode >= 300) {
        callbacks.onError(normalizeErrorMessage(response.data, statusCode))
        return
      }

      if (!hasCompleted && typeof response.data === 'string' && response.data.trim()) {
        callbacks.onError(response.data)
      }
    },
    fail: (error: { errMsg?: string }) => {
      if (isAborted) {
        return
      }

      callbacks.onError(error.errMsg || 'AI 对话请求失败')
    },
  } as any) as ChunkedRequestTask

  const consumeBuffer = () => {
    const normalized = buffer.replace(/\r\n/g, '\n')
    const segments = normalized.split('\n\n')

    if (!normalized.endsWith('\n\n')) {
      buffer = segments.pop() ?? ''
    } else {
      buffer = ''
    }

    for (const segment of segments) {
      const parsed = parseServerEvent(segment)
      if (!parsed?.data) {
        continue
      }

      try {
        const payload = JSON.parse(parsed.data) as Record<string, unknown>

        if (parsed.event === 'started') {
          callbacks.onStarted?.({ model: String(payload.model ?? '') })
          continue
        }

        if (parsed.event === 'delta') {
          callbacks.onDelta({ content: String(payload.content ?? '') })
          continue
        }

        if (parsed.event === 'done') {
          hasCompleted = true
          callbacks.onDone({
            model: String(payload.model ?? ''),
            reply: String(payload.reply ?? ''),
          })
          continue
        }

        if (parsed.event === 'error') {
          callbacks.onError(String(payload.message ?? 'AI 对话失败'))
        }
      } catch (error) {
        callbacks.onError(error instanceof Error ? error.message : 'AI 对话解析失败')
      }
    }
  }

  if (typeof requestTask.onChunkReceived === 'function') {
    requestTask.onChunkReceived((chunk: { data: ArrayBuffer }) => {
      if (isAborted) {
        return
      }

      buffer += decodeChunk(chunk.data)
      consumeBuffer()
    })
  } else {
    callbacks.onError('当前小程序环境不支持流式响应')
  }

  return {
    abort: () => {
      isAborted = true
      requestTask.abort()
    },
  }
}

function streamWechatCloudAiChat(
  payload: StreamAiChatPayload,
  callbacks: StreamAiChatCallbacks,
): AiChatStreamHandle {
  const cloudAi = getWechatCloudAiExtension()
  if (!cloudAi?.createModel) {
    callbacks.onError('当前小程序环境不支持微信云开发 AI')
    return { abort: () => {} }
  }

  let isAborted = false

  void (async () => {
    try {
      callbacks.onStarted?.({ model: WECHAT_CLOUD_AI_MODEL })

      const response = await cloudAi
        .createModel(WECHAT_CLOUD_AI_PROVIDER)
        .streamText({
          data: {
            model: WECHAT_CLOUD_AI_MODEL,
            messages: buildWechatCloudMessages(payload.message, payload.history),
          },
        })

      let reply = ''

      for await (const event of response.eventStream as AsyncIterable<{ data?: string }>) {
        if (isAborted) {
          return
        }

        if (event.data === '[DONE]') {
          break
        }

        const parsed = parseWechatCloudEventData(event.data)
        if (!parsed) {
          continue
        }

        if (parsed.content) {
          reply += parsed.content
          callbacks.onDelta({ content: parsed.content })
        }
      }

      if (isAborted) {
        return
      }

      callbacks.onDone({
        model: WECHAT_CLOUD_AI_MODEL,
        reply,
      })
    } catch (error) {
      if (isAborted) {
        return
      }

      callbacks.onError(normalizeWechatCloudErrorMessage(error))
    }
  })()

  return {
    abort: () => {
      isAborted = true
    },
  }
}

function parseWechatCloudEventData(
  rawData: string | undefined,
): { content: string } | null {
  if (typeof rawData !== 'string' || !rawData.trim()) {
    return null
  }

  const payload = JSON.parse(rawData) as {
    choices?: Array<{
      delta?: {
        content?: string
      }
    }>
  }

  return {
    content: payload.choices?.[0]?.delta?.content ?? '',
  }
}

function normalizeWechatCloudErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim()) {
    return error.message
  }

  if (
    error &&
    typeof error === 'object' &&
    'errMsg' in error &&
    typeof error.errMsg === 'string' &&
    error.errMsg.trim()
  ) {
    return error.errMsg
  }

  return '微信云开发 AI 调用失败'
}

export async function generateAiImage(prompt: string): Promise<GenerateAiImageResult> {
  const normalizedPrompt = prompt.trim()
  if (!normalizedPrompt) {
    throw new Error('请输入图片描述')
  }

  if (!WECHAT_CLOUD_IMAGE_FUNCTION_NAME.trim()) {
    throw new Error('还未配置生图云函数名称')
  }

  const result = await callWechatCloudFunction<{
    success?: boolean
    imageUrl?: string
    revised_prompt?: string
    code?: string
    message?: string
  }>(WECHAT_CLOUD_IMAGE_FUNCTION_NAME, {
    prompt: normalizedPrompt,
  })

  if (!result?.success || !result.imageUrl) {
    throw new Error(result?.message?.trim() || '图片生成失败')
  }

  return {
    imageUrl: result.imageUrl,
    revisedPrompt: result.revised_prompt?.trim() || normalizedPrompt,
  }
}
