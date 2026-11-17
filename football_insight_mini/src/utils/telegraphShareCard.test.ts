import { describe, expect, test } from 'bun:test'
import type { DailyTopicEntry, DailyTopicItem } from '../types/dailyTopic'
import {
  buildTelegraphEntrySharePayload,
  buildTelegraphEntryTimelinePayload,
  cleanTelegraphHeadline,
  drawRoundupShareCard,
  drawTelegraphEntryShareCard,
  TELEGRAPH_SHARE_CARD,
  telegraphEntryShareSignature,
  telegraphEntryShareTitle,
} from './telegraphShareCard'

interface RecordedTextCall {
  text: string
  x: number
  y: number
  size: number
  color: string
  align: string
}

function createRecordingCanvasContext() {
  const texts: RecordedTextCall[] = []
  let fontSize = 24
  let fillStyle = '#000000'
  let textAlign = 'left'

  const gradient = {
    addColorStop: () => {},
  }

  const context = {
    font: '24px sans-serif',
    setFillStyle(value: string) {
      fillStyle = value
    },
    setStrokeStyle() {},
    setLineWidth() {},
    setLineCap() {},
    setLineJoin() {},
    setFontSize(size: number) {
      fontSize = size
    },
    setTextAlign(align: 'left' | 'center' | 'right') {
      textAlign = align
    },
    createLinearGradient() {
      return gradient
    },
    measureText(value: string) {
      const width = Array.from(value).reduce((sum, ch) => sum + (ch.charCodeAt(0) <= 0xff ? fontSize * 0.56 : fontSize), 0)
      return { width }
    },
    fillRect() {},
    strokeRect() {},
    beginPath() {},
    moveTo() {},
    lineTo() {},
    arc() {},
    arcTo() {},
    closePath() {},
    fill() {},
    stroke() {},
    fillText(text: string, x: number, y: number) {
      texts.push({ text, x, y, size: fontSize, color: String(fillStyle), align: textAlign })
    },
  } as unknown as UniApp.CanvasContext

  return { context, texts }
}

const sampleEntry = (overrides: Partial<DailyTopicEntry> = {}): DailyTopicEntry => ({
  id: 'entry_1',
  category: 'domestic',
  title: '【据报道斯坦丘承诺加盟克卢日大学，合同到期后正式转会】',
  text: '据罗马尼亚媒体报道，大连英博外援斯坦丘已承诺加盟克卢日大学，他将于2027年1月合同到期后正式转会离开中超。',
  publishedAt: '2026-11-17T06:57:00Z',
  timeLabel: '14:57',
  sources: [
    {
      title: '斯坦丘冬窗告别中超',
      url: 'https://news.example.com/1',
      publisher: '直播吧',
      publishedAt: '2026-11-17T06:57:00Z',
      dateLabel: '2026/11/17 14:57',
    },
    {
      title: '罗媒确认斯坦丘新东家',
      url: 'https://news.example.com/2',
      publisher: '懂球帝',
      publishedAt: '2026-11-17T06:58:00Z',
      dateLabel: '2026/11/17 14:58',
    },
  ],
  likesCount: 12,
  dislikesCount: 1,
  commentsCount: 6,
  likedByMe: false,
  dislikedByMe: false,
  ...overrides,
})

const sampleRoundup = (entries: DailyTopicEntry[]): DailyTopicItem => ({
  id: 'topic_roundup_20261117',
  teamId: 77680,
  title: '每日电报',
  summary: '每日电报摘要',
  paragraphs: ['每日电报正文'],
  publishedAt: '2026-11-17T07:00:00Z',
  publishedLabel: '2026/11/17 15:00',
  sources: sampleEntry().sources,
  sourceLabel: '直播吧 · 懂球帝 · 2 个来源',
  heat: 99,
  commentsEnabled: true,
  commentsCount: 8,
  readMinutes: 2,
  audio: { status: 'ready', url: 'https://audio.example.com/1.mp3', durationSeconds: 60 },
  entries,
  categoryAudio: {},
})

describe('telegraphShareCard drawing', () => {
  test('entry share card draws dark-gold editorial hierarchy and omits all bottom sources', () => {
    const { context, texts } = createRecordingCanvasContext()
    const entry = sampleEntry()

    drawTelegraphEntryShareCard(context, entry, { publishedAt: '2026-11-17T07:00:00Z', commentsVisible: true })

    const joinedText = texts.map(item => item.text).join('')
    const lineText = texts.map(item => item.text).join('\n')
    expect(lineText.includes('每日电报')).toBe(true)
    expect(lineText.includes('国内')).toBe(true)
    expect(lineText.includes('11月17日 14:57')).toBe(true)
    expect(lineText.includes('足球洞察')).toBe(true)
    expect(joinedText.includes('据报道斯坦丘承诺加盟克卢日大学')).toBe(true)
    expect(joinedText.includes('大连英博外援斯坦丘已承诺加盟克卢日大学')).toBe(true)
    expect(lineText.includes('赞同 ')).toBe(true)
    expect(lineText.includes('12')).toBe(true)
    expect(lineText.includes(' · 评论 ')).toBe(true)
    expect(lineText.includes('6')).toBe(true)
    expect(lineText.includes('看热议')).toBe(true)
    expect(texts.some(item => item.size >= 58)).toBe(true)
    expect(texts.some(item => item.size === 44)).toBe(true)

    // 明确约束：去掉底部来源，不绘制任何媒体来源信息。
    expect(joinedText.includes('来源')).toBe(false)
    expect(joinedText.includes('直播吧')).toBe(false)
    expect(joinedText.includes('懂球帝')).toBe(false)

    // 所有 textAlign 保持 left（由绘制侧算好横坐标，避免真机对齐错位）且 y 坐标在画布范围内。
    expect(texts.every(item => item.align === 'left')).toBe(true)
    expect(texts.every(item => item.y > 0 && item.y < TELEGRAPH_SHARE_CARD.height)).toBe(true)
  })

  test('entry share card hides comment wording during mini review restriction', () => {
    const { context, texts } = createRecordingCanvasContext()
    drawTelegraphEntryShareCard(context, sampleEntry({ commentsCount: 8, likesCount: 5 }), { commentsVisible: false })

    const allText = texts.map(item => item.text).join('')
    expect(allText.includes('看详情')).toBe(true)
    expect(allText.includes('5')).toBe(true)
    expect(allText.includes('人赞同')).toBe(true)
    expect(allText.includes('评论')).toBe(false)
    expect(allText.includes('看法')).toBe(false)
    expect(allText.includes('热议')).toBe(false)
    expect(allText.includes('来源')).toBe(false)
  })

  test('entry share card handles zero reactions cleanly', () => {
    const { context, texts } = createRecordingCanvasContext()
    drawTelegraphEntryShareCard(context, sampleEntry({ likesCount: 0, commentsCount: 0 }), { commentsVisible: true })
    const allText = texts.map(item => item.text).join('')
    expect(allText.includes('进来聊聊你的看法')).toBe(true)
  })

  test('roundup share card draws compact header, multi-line highlights, and no sources', () => {
    const entries = [
      sampleEntry({ id: 'e1', category: 'domestic', title: '官方：成都蓉城确认主教练与教练组续约，下赛季继续冲击亚冠名额', timeLabel: '14:57' }),
      sampleEntry({ id: 'e2', category: 'world', title: '曼联官方宣布新帅人选并将在本周末迎来首秀', timeLabel: '13:15' }),
      sampleEntry({ id: 'e3', category: 'abroad', title: '中国留洋小将连续三轮首发并贡献一次关键助攻', timeLabel: '09:11' }),
    ]
    const { context, texts } = createRecordingCanvasContext()
    drawRoundupShareCard(context, sampleRoundup(entries))

    const lineText = texts.map(item => item.text).join('\n')
    const joinedText = texts.map(item => item.text).join('')
    expect(lineText.includes('每日电报')).toBe(true)
    expect(lineText.includes('11月17日')).toBe(true)
    expect(lineText.includes('周二')).toBe(true)
    expect(lineText.includes(' 条快讯')).toBe(true)
    expect(lineText.includes('看全部 3 条')).toBe(true)
    expect(joinedText.includes('官方：成都蓉城确认主教练与教练组续约')).toBe(true)
    expect(lineText.includes('今日重点 · 国内/国际/留洋')).toBe(true)

    // 同样严格禁止绘制来源。
    expect(joinedText.includes('来源')).toBe(false)
    expect(joinedText.includes('直播吧')).toBe(false)
    expect(joinedText.includes('懂球帝')).toBe(false)

    // 子卡片正文标题使用 42px 大字号且底线绝不侵入底栏分割线 (650)。
    const subCardTexts = texts.filter(t => t.size === 42)
    expect(subCardTexts.length >= 3).toBe(true)
    expect(subCardTexts.every(t => t.y < 645)).toBe(true)
  })
})

describe('telegraphShareCard helpers and payloads', () => {
  test('cleanTelegraphHeadline strips matching outer brackets without breaking prefix tags', () => {
    expect(cleanTelegraphHeadline(' 【蓉城取胜】 ')).toBe('蓉城取胜')
    expect(cleanTelegraphHeadline('[快讯] 欧冠焦点战')).toBe('[快讯] 欧冠焦点战')
  })

  test('telegraphEntryShareTitle formats bracketed headline and snippet safely for Unicode emoji', () => {
    const title = telegraphEntryShareTitle(sampleEntry({
      title: '【足球⚽重磅】',
      text: '⚽'.repeat(45),
    }))
    expect(title.startsWith('【足球⚽重磅】')).toBe(true)
    expect(title.endsWith('…')).toBe(true)
    // 确保不会产生孤立的 UTF-16 代理对。
    expect(typeof encodeURIComponent(title)).toBe('string')
  })

  test('buildTelegraphEntrySharePayload and timeline payload attach cover, attribution and fallback safely', () => {
    const entry = sampleEntry()
    expect(buildTelegraphEntrySharePayload({
      topicId: 'topic_1',
      entryId: 'entry_1',
      entry,
      imageUrl: '/tmp/entry.png',
    })).toEqual({
      title: telegraphEntryShareTitle(entry),
      path: '/pages/telegraph-entry/index?topicId=topic_1&entryId=entry_1&src=share&cid=telegraph_entry',
      imageUrl: '/tmp/entry.png',
    })

    expect(buildTelegraphEntrySharePayload({
      topicId: '../hack',
      entryId: 'entry_1',
      entry: null,
    })).toEqual({
      title: '每日电报 · 足球洞察',
      path: '/pages/topics/index?src=share&cid=telegraph_entry',
    })

    expect(buildTelegraphEntryTimelinePayload({
      topicId: 'topic_1',
      entryId: 'entry_1',
      entry,
      imageUrl: '/tmp/entry.png',
    })).toEqual({
      title: '每日电报 · 据报道斯坦丘承诺加盟克卢日大学，合同到期后正式转会',
      query: 'topicId=topic_1&entryId=entry_1&src=share&cid=telegraph_entry',
      imageUrl: '/tmp/entry.png',
    })
  })

  test('telegraphEntryShareSignature invalidates when reaction, comment count or restriction changes', () => {
    const entry = sampleEntry()
    const base = telegraphEntryShareSignature(entry, { publishedAt: '2026-11-17T07:00:00Z', commentsVisible: true })
    expect(telegraphEntryShareSignature({ ...entry, likesCount: 13 }, { publishedAt: '2026-11-17T07:00:00Z', commentsVisible: true }) === base).toBe(false)
    expect(telegraphEntryShareSignature({ ...entry, commentsCount: 7 }, { publishedAt: '2026-11-17T07:00:00Z', commentsVisible: true }) === base).toBe(false)
    expect(telegraphEntryShareSignature(entry, { publishedAt: '2026-11-17T07:00:00Z', commentsVisible: false }) === base).toBe(false)
  })
})
