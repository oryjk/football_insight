import { describe, expect, test } from 'bun:test'

import {
  buildWechatCloudMessages,
  getAiChatCapabilityNotice,
  getAiInteractionMeta,
  resolveAiChatMode,
} from './aiChat'

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
  test('returns image mode copy for generate image flow', () => {
    expect(getAiInteractionMeta('image')).toEqual({
      emptyCopy: '你可以描述想要的足球主题画面，比如吉祥物、球员海报、训练场景或比赛氛围图。',
      emptyTitle: '试试生成一张足球主题图片',
      placeholder: '描述你想生成的图片，例如：一只穿着红色球衣的小蜜蜂在足球场上带球',
      submitLabel: '生成图片',
    })
  })
})
