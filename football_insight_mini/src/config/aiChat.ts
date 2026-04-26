import type { AiChatHistoryMessagePayload } from '../types/ai'
import type { AiChatMode } from '../types/system'

export const DEFAULT_AI_CHAT_MODE: AiChatMode = 'backend_proxy'
export const WECHAT_CLOUD_ENV_ID = 'football-insight-3fknj64915d5867'
export const WECHAT_CLOUD_AI_PROVIDER = 'hunyuan-exp'
export const WECHAT_CLOUD_AI_MODEL = 'hunyuan-turbos-latest'
export const WECHAT_CLOUD_IMAGE_FUNCTION_NAME = 'generateImage-pa1pub'

export interface AiChatCapabilityNotice {
  title: string
  content: string
}

export type AiInteractionMode = 'text' | 'image'

export interface AiInteractionMeta {
  emptyCopy: string
  emptyTitle: string
  placeholder: string
  submitLabel: string
}

export function resolveAiChatMode(mode: string | null | undefined): AiChatMode {
  return mode === 'frontend_direct' ? 'frontend_direct' : DEFAULT_AI_CHAT_MODE
}

export function getAiChatCapabilityNotice(
  mode: AiChatMode | string | null | undefined,
): AiChatCapabilityNotice | null {
  if (resolveAiChatMode(mode) !== 'frontend_direct') {
    return null
  }

  return {
    title: '当前 AI 能力说明',
    content: '当前为云开发 AI，对话不能联网搜索最新新闻，知识截止到 2024 年 6 月。',
  }
}

export function getAiInteractionMeta(mode: AiInteractionMode): AiInteractionMeta {
  if (mode === 'image') {
    return {
      emptyCopy: '你可以描述想要的足球主题画面，比如吉祥物、球员海报、训练场景或比赛氛围图。',
      emptyTitle: '试试生成一张足球主题图片',
      placeholder: '描述你想生成的图片，例如：一只穿着红色球衣的小蜜蜂在足球场上带球',
      submitLabel: '生成图片',
    }
  }

  return {
    emptyCopy: '你可以直接问榜首走势，也可以问某支球队、某位球员，或者欧冠、世界杯这些更泛的足球问题。',
    emptyTitle: '和小罗聊聊今天想看的足球话题',
    placeholder: '问问今天的榜首走势，或者任何足球相关的问题',
    submitLabel: '发送',
  }
}

export function buildWechatCloudMessages(
  message: string,
  history: AiChatHistoryMessagePayload[],
): AiChatHistoryMessagePayload[] {
  const normalizedMessage = message.trim()

  const normalizedHistory = history
    .map((item) => ({
      role: item.role,
      content: item.content.trim(),
    }))
    .filter((item) => item.content.length > 0)

  const lastMessage = normalizedHistory[normalizedHistory.length - 1]
  if (lastMessage?.role === 'user' && lastMessage.content === normalizedMessage) {
    return normalizedHistory
  }

  return [
    ...normalizedHistory,
    {
      role: 'user',
      content: normalizedMessage,
    },
  ]
}
