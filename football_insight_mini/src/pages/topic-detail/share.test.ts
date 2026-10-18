import { describe, expect, test } from 'bun:test'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { buildTopicTimelinePayload, buildTopicSharePayload, containSharePhoto, topicShareExcerpt, topicShareSignature, wrapShareCardText } from './share'

const article: DailyTopicItem = { id: 'topic_2026-10-13', teamId: 77680, title: '蓉城争冠关键战', summary: '这是摘要',
  paragraphs: ['  正文第一段  ', '正文第二段'], publishedAt: '2026-10-13T00:00:00Z', publishedLabel: '2026/10/13 08:00',
  sources: [], sourceLabel: '来源', heat: 0, readMinutes: 1, audio: { status: 'pending', url: null, durationSeconds: null } }

describe('article sharing', () => {
  test('links to the current article with its title and generated cover', () => {
    expect(buildTopicSharePayload(article, '/tmp/topic.png')).toEqual({ title: article.title,
      path: '/pages/topic-detail/index?id=topic_2026-10-13&src=share&cid=topic_detail', imageUrl: '/tmp/topic.png' })
  })
  test('missing and invalid articles lead to the public topic list', () => {
    expect(buildTopicSharePayload(null).path).toBe('/pages/topics/index?src=share&cid=topic_detail')
    expect(buildTopicSharePayload({ ...article, id: '../private?token=secret' }).path).toBe('/pages/topics/index?src=share&cid=topic_detail')
  })
  test('timeline sharing keeps the public article id and cover', () => {
    expect(buildTopicTimelinePayload(article, '/tmp/topic.png')).toEqual({
      title: article.title, query: 'id=topic_2026-10-13&src=share&cid=topic_detail', imageUrl: '/tmp/topic.png',
    })
    for (const item of [null, { ...article, id: '../private?token=secret' }]) {
      expect(buildTopicTimelinePayload(item)).toEqual({
        title: '足球洞察 · 足球新闻', query: 'src=share&cid=topic_detail',
      })
    }
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
  test('first-photo changes invalidate the cover without replacing article text', () => {
    const photo = { url: 'https://media.example.com/first.jpg', width: 640, height: 426,
      sourceUrl: 'https://news.example.com/a', publisher: '媒体', afterParagraph: 1 }
    const illustrated = { ...article, images: [photo] }
    expect(topicShareSignature(illustrated) === topicShareSignature(article)).toBe(false)
    expect(topicShareSignature({ ...article, images: [{ ...photo, url: 'https://media.example.com/new-first.jpg' }] }) === topicShareSignature(illustrated)).toBe(false)
    expect(topicShareSignature({ ...article, images: [photo, { ...photo, url: 'https://media.example.com/second.jpg' }] })).toBe(topicShareSignature(illustrated))
    expect(topicShareExcerpt(illustrated)).toBe(topicShareExcerpt(article))
  })
  test('fits complete landscape and portrait photos inside the card without cropping', () => {
    const area = { x: 64, y: 336, width: 420, height: 300 }
    for (const [width, height] of [[640, 426], [400, 800]]) {
      const photo = containSharePhoto(width, height, area)!
      expect(Math.abs(photo.width / photo.height - width / height) < 1e-8).toBe(true)
      expect(photo.x >= area.x && photo.y >= area.y).toBe(true)
      expect(photo.x + photo.width <= area.x + area.width).toBe(true)
      expect(photo.y + photo.height <= area.y + area.height).toBe(true)
    }
    for (const width of [0, -1, NaN, Infinity]) expect(containSharePhoto(width, 426, area)).toBeNull()
  })
})
