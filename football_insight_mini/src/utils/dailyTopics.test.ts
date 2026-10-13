import { expect, test } from 'bun:test'
import { mapTopicHistory, mapTopicDetail, mergeTopicHistory, topicDetailUrl, formatTopicHeat } from './dailyTopics'
const article = (id: string, time: string) => ({ id, team_id: 77680, title: '新闻标题', summary: '摘要', paragraphs: ['正文'], published_at: time,
  sources: [{ title: '来源', publisher: '新闻', url: 'https://news.example.com/a', published_at: time }], audio: { status: 'pending' } })
const page = (items: unknown[], next: string | null = null) => ({ team_id: 77680, items, next_cursor: next, has_more: next !== null })
test('历史列表跨日期排序且不受首页五篇上限限制', () => {
  const items = Array.from({ length: 8 }, (_, i) => article(`id-${i}`, `2026-10-${String(10 + i).padStart(2, '0')}T08:00:00+08:00`))
  expect(mapTopicHistory(page(items, 'cursor')).items.map(i => i.id)).toEqual(items.map(i => i.id).reverse())
})
test('合并分页按时间倒序，同一 ID 使用更新内容；同时间按 ID 稳定排序', () => {
  const old = mapTopicHistory(page([article('b', '2026-10-12T08:00:00Z'), article('a', '2026-10-12T08:00:00Z')])).items
  const next = mapTopicHistory(page([{ ...article('a', '2026-10-12T08:00:00Z'), title: '更新正文' }, article('c', '2026-10-11T08:00:00Z')])).items
  const merged = mergeTopicHistory(old, next)
  expect(merged.map(i => i.id)).toEqual(['a', 'b', 'c'])
  expect(merged[0].title).toBe('更新正文')
})
test('异常历史、分页元数据与详情不当成成功空数组', () => {
  for (const value of [{ ...page([]), team_id: 1 }, page([], 'cursor'), { ...page([]), has_more: true }, page([{ ...article('x', '2026-10-12T08:00:00Z'), sources: [] }])]) {
    expect(() => mapTopicHistory(value)).toThrow()
  }
  expect(mapTopicHistory(page([])).items).toEqual([])
  expect(() => mapTopicDetail({})).toThrow()
})
test('详情保留所有段落、来源和音频，路由编码内容 ID', () => {
  const value = { ...article('article-a', '2026-10-12T08:00:00Z'), paragraphs: ['首段', '第二段'], audio: { status: 'ready', dialect: 'sichuan', url: 'https://media.example.com/a.m4a', duration_seconds: 10 } }
  const item = mapTopicDetail(value)
  expect(item.paragraphs).toEqual(value.paragraphs)
  expect(item.audio.url).toBe(value.audio.url)
  expect(topicDetailUrl('a&b')).toBe('/pages/topic-detail/index?id=a%26b')
})

test('热度兼容旧接口并拒绝负值、非数值和不安全整数', () => {
  const value = article('heat-topic','2026-10-13T08:00:00Z')
  expect(mapTopicDetail(value).heat).toBe(0)
  expect(mapTopicDetail({ ...value, heat: 12345 }).heat).toBe(12345)
  for (const heat of [-1, NaN, Infinity, '12', 1.2, Number.MAX_SAFE_INTEGER + 1]) {
    expect(mapTopicDetail({ ...value, heat }).heat).toBe(0)
  }
})

test('热度显示统一以万、亿简写，小于一万保留整数', () => {
  expect([0, 99, 9999, 10000, 12345, 100000000].map(formatTopicHeat)).toEqual(['0', '99', '9999', '1万', '1.2万', '1亿'])
})

test('正文配图兼容旧话题并过滤未引用来源和错误图片', () => {
  const value = article('photo-topic', '2026-10-13T08:00:00Z')
  const photo = { url: 'https://media.example.com/photo.jpg', width: 640, height: 444,
    source_url: 'https://news.example.com/a', publisher: '新闻', after_paragraph: 1 }
  expect(mapTopicDetail(value).images).toEqual([])
  const item = mapTopicDetail({ ...value, images: [photo, { ...photo, url: 'http://media.example.com/x' },
    { ...photo, source_url: 'https://news.example.com/uncited' }, { ...photo, width: -1 },
    { ...photo, after_paragraph: 99 }, null] })
  expect(item.images).toEqual([{ url: photo.url, width: 640, height: 444,
    sourceUrl: photo.source_url, publisher: '新闻', afterParagraph: 1 }])
  expect(item.paragraphs).toEqual(['正文'])
})
