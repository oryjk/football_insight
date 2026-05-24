import { describe, expect, test } from 'bun:test'

import {
  buildWechatCloudMessages,
  getAiChatCapabilityNotice,
  getAiInteractionMeta,
  resolveAiChatMode,
} from './aiChat'
import * as aiChatConfig from './aiChat'

describe('resolveAiChatMode', () => {
  test('falls back to backend_proxy for unknown values', () => {
    expect(resolveAiChatMode(undefined)).toBe('backend_proxy')
    expect(resolveAiChatMode('unknown')).toBe('backend_proxy')
    expect(resolveAiChatMode('frontend_direct')).toBe('frontend_direct')
  })
})

describe('buildWechatCloudMessages', () => {
  test('does not duplicate the latest user message already present in history', () => {
    expect(
      buildWechatCloudMessages('再看看上海申花', [
        { role: 'user', content: '你好' },
        { role: 'assistant', content: '你好，我在。' },
        { role: 'user', content: '再看看上海申花' },
      ]),
    ).toEqual([
      { role: 'user', content: '你好' },
      { role: 'assistant', content: '你好，我在。' },
      { role: 'user', content: '再看看上海申花' },
    ])
  })

  test('appends the current user message when it is not already in history', () => {
    expect(
      buildWechatCloudMessages('分析一下这轮榜首走势', [
        { role: 'user', content: '你好' },
        { role: 'assistant', content: '你好，我在。' },
      ]),
    ).toEqual([
      { role: 'user', content: '你好' },
      { role: 'assistant', content: '你好，我在。' },
      { role: 'user', content: '分析一下这轮榜首走势' },
    ])
  })
})

describe('getAiChatCapabilityNotice', () => {
  test('returns frontend direct notice only for wechat cloud mode', () => {
    expect(getAiChatCapabilityNotice('backend_proxy')).toBeNull()
    expect(getAiChatCapabilityNotice('frontend_direct')).toEqual({
      title: '当前 AI 能力说明',
      content:
        '当前为云开发 AI，对话不能联网搜索最新新闻，知识截止到 2024 年 6 月。',
    })
  })
})

describe('getAiInteractionMeta', () => {
  test('returns text chat copy only', () => {
    expect(getAiInteractionMeta()).toEqual({
      emptyCopy: '你可以直接问榜首走势，也可以问某支球队、某位球员，或者欧冠、世界杯这些更泛的足球问题。',
      emptyTitle: '和小罗聊聊今天想看的足球话题',
      placeholder: '问问今天的榜首走势，或者任何足球相关的问题',
      submitLabel: '发送',
    })
  })

  test('does not expose image generation configuration', () => {
    expect((aiChatConfig as Record<string, unknown>).WECHAT_CLOUD_IMAGE_FUNCTION_NAME).toBe(undefined)
  })
})
