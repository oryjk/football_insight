import { describe, expect, test } from 'bun:test'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { buildTopicSharePayload, topicShareExcerpt, topicShareSignature, wrapShareCardText } from './share'

const article: DailyTopicItem = { id: 'topic_2026-10-13', teamId: 77680, title: '蓉城争冠关键战', summary: '这是摘要',
  paragraphs: ['  正文第一段  ', '正文第二段'], publishedAt: '2026-10-13T00:00:00Z', publishedLabel: '2026/10/13 08:00',
  sources: [], sourceLabel: '来源', heat: 0, readMinutes: 1, audio: { status: 'pending', url: null, durationSeconds: null } }

describe('article sharing', () => {
  test('links to the current article with its title and generated cover', () => {
    expect(buildTopicSharePayload(article, '/tmp/topic.png')).toEqual({ title: article.title,
      path: '/pages/topic-detail/index?id=topic_2026-10-13', imageUrl: '/tmp/topic.png' })
  })
  test('missing and invalid articles lead to the public topic list', () => {
    expect(buildTopicSharePayload(null).path).toBe('/pages/topics/index')
    expect(buildTopicSharePayload({ ...article, id: '../private?token=secret' }).path).toBe('/pages/topics/index')
  })
  test('the cover excerpt comes from full-text paragraphs', () => {
    expect(topicShareExcerpt(article)).toBe('正文第一段 正文第二段')
    expect(topicShareExcerpt(article).includes(article.summary)).toBe(false)
  })
  test('wraps and truncates without splitting Unicode characters', () => {
    const measure = (text: string) => Array.from(text).length
    expect(wrapShareCardText('甲乙丙丁戊己庚辛', measure, 3, 2)).toEqual(['甲乙丙', '丁戊…'])
    expect(wrapShareCardText('甲⚽乙', measure, 2, 2)).toEqual(['甲⚽', '乙'])
    expect(wrapShareCardText('甲乙丙', measure, 3, 1)).toEqual(['甲乙丙'])
  })
  test('content changes invalidate the cached cover', () => {
    expect(topicShareSignature({ ...article, paragraphs: ['新版正文'] }) === topicShareSignature(article)).toBe(false)
  })
})
