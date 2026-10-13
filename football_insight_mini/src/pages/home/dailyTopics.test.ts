import { describe, expect, test } from 'bun:test'
import { mapDailyTopicEdition, topicPlaybackSignature, formatTopicAudioTime } from './dailyTopics'

function article(id = 'immutable-content-a') {
  return { id, team_id: 77680, title: '蓉城备战下一轮', summary: '训练报道整理。', paragraphs: ['基于原文的新闻整理。'],
    published_at: '2026-10-13T00:30:00Z', sources: [{ title: '原文标题', publisher: '新闻机构', url: 'https://news.example.com/a', published_at: '2026-10-12T23:00:00+08:00' }],
    audio: { status: 'ready', url: 'https://media.example.com/a.m4a', dialect: 'sichuan', duration_seconds: 60 } }
}
function payload(items: unknown[] = [article()]) {
  return { team_id: 77680, edition_date: '2026-10-13', updated_at: '2026-10-13T08:30:00+08:00', items }
}
function rejects(payload: unknown): boolean {
  try { mapDailyTopicEdition(payload); return false } catch { return true }
}
describe('今日话题接口转换', () => {
  test('保留真实文章、来源与发布日期，时间显示固定北京时间', () => {
    const edition = mapDailyTopicEdition(payload())
    expect(edition.dateLabel).toBe('2026/10/13')
    expect(edition.items[0].publishedLabel).toBe('2026/10/13 08:30')
    expect(edition.items[0].sources[0].title).toBe('原文标题')
    expect(edition.items[0].sources[0].dateLabel).toBe('2026/10/12 23:00')
    expect(edition.items[0].audio.url).toBe('https://media.example.com/a.m4a')
  })
  test('按不可变内容 ID 去重，跳过错误球队/缺失来源，最多五篇', () => {
    const edition = mapDailyTopicEdition(payload([article(), article(), { ...article('other'), team_id: 1 }, { ...article('missing'), sources: [] },
      ...Array.from({ length: 8 }, (_, index) => article(`article-${index}`))]))
    expect(edition.items.map(item => item.id)).toEqual(['immutable-content-a', 'article-0', 'article-1', 'article-2', 'article-3'])
  })
  test('成功空数组保持空，没有发布批次就没有日期；异常数据不可当作成功空内容', () => {
    const edition = mapDailyTopicEdition({ ...payload([]), edition_date: null, updated_at: null })
    expect(edition.items).toEqual([])
    expect(edition.dateLabel).toBe('')
    expect(rejects({ ...payload(), team_id: 1 })).toBe(true)
    expect(rejects({ ...payload(), edition_date: '2026-02-30' })).toBe(true)
    expect(rejects(payload([{ ...article(), sources: [] }]))).toBe(true)
  })
  test('ready、HTTPS、四川话同时满足才能播放；pending 不复用 URL', () => {
    for (const audio of [
      { ...article().audio, url: 'http://media.example.com/a.m4a' },
      { ...article().audio, dialect: 'mandarin' },
      { ...article().audio, status: 'pending' },
      { ...article().audio, url: 'https://media.example.com/a.m4a\n' },
    ]) {
      const edition = mapDailyTopicEdition(payload([{ ...article(), audio }]))
      expect(edition.items[0].audio.url).toBeNull()
      expect(topicPlaybackSignature(edition, article().id)).toBe('')
    }
  })
  test('更新批次、替换文章、正文或音频链接、撤下音频均使旧播放失效', () => {
    const original = mapDailyTopicEdition(payload()), signature = topicPlaybackSignature(original, article().id)
    const changed = [
      mapDailyTopicEdition({ ...payload(), updated_at: '2026-10-13T09:00:00+08:00' }),
      mapDailyTopicEdition(payload([article('new-content')])),
      mapDailyTopicEdition(payload([{ ...article(), paragraphs: ['新的正文'] }])),
      mapDailyTopicEdition(payload([{ ...article(), audio: { ...article().audio, url: 'https://media.example.com/b.m4a' } }])),
      mapDailyTopicEdition(payload([{ ...article(), audio: { ...article().audio, status: 'failed', url: null } }])),
      mapDailyTopicEdition(payload([])),
    ]
    expect(signature !== '').toBe(true)
    for (const edition of changed) expect(topicPlaybackSignature(edition, article().id) !== signature).toBe(true)
    expect(topicPlaybackSignature(mapDailyTopicEdition(payload()), article().id)).toBe(signature)
  })
  test('播放器进度兼容异常值', () => {
    expect(formatTopicAudioTime(125.9)).toBe('2:05')
    for (const value of [-1, NaN, Infinity]) expect(formatTopicAudioTime(value)).toBe('0:00')
  })
})
