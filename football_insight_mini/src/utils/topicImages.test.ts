import { expect, test } from 'bun:test'
import { mapTopicDetail } from './dailyTopics'
import { topicReadingBlocks } from './topicImages'

test('配图插在指定段落之后，旧话题仍返回完整文字', () => {
  const source = { title: '原文', url: 'https://news.example.com/a', publisher: '媒体', published_at: '2026-10-13T08:00:00Z' }
  const original = { id: 'article', team_id: 77680, title: '标题', summary: '摘要', published_at: source.published_at,
    paragraphs: ['段落一', '段落二'], sources: [source], audio: { status: 'pending' } }
  const photo = { url: 'https://media.example.com/photo.jpg', width: 640, height: 444,
    source_url: source.url, publisher: source.publisher, after_paragraph: 2 }
  const blocks = topicReadingBlocks(mapTopicDetail({ ...original, images: [photo] }))
  expect(blocks.map(block => block.text)).toEqual(original.paragraphs)
  expect(blocks[0].images).toEqual([])
  expect(blocks[1].images[0].url).toBe(photo.url)
  expect(topicReadingBlocks(mapTopicDetail(original)).map(block => block.images)).toEqual([[], []])
})
