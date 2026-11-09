import type { DailyTopicItem } from '../../types/dailyTopic'
import { buildSharePath, buildShareQuery } from '../../utils/shareSource'
import { isRoundupTopic, topicDetailUrl } from '../../utils/dailyTopics'
import type { DailyTopicEntry } from '../../types/dailyTopic'

export const TOPIC_SHARE_CARD = { width: 1000, height: 800 } as const

export function buildTopicSharePayload(item: DailyTopicItem | null, imageUrl?: string) {
  if (!item || !/^[a-zA-Z0-9_-]{1,128}$/.test(item.id)) {
    return { title: '足球洞察 · 足球新闻', path: buildSharePath('/pages/topics/index', 'topic_detail') }
  }
  return { title: isRoundupTopic(item) ? roundupShareTitle(item) : item.title,
    path: buildSharePath(topicDetailUrl(item.id), 'topic_detail'), ...(imageUrl ? { imageUrl } : {}) }
}

export function buildTopicTimelinePayload(item: DailyTopicItem | null, imageUrl?: string) {
  const share = buildTopicSharePayload(item, imageUrl)
  const id = item && /^[a-zA-Z0-9_-]{1,128}$/.test(item.id) ? `id=${encodeURIComponent(item.id)}&` : ''
  return { title: share.title, query: `${id}${buildShareQuery('topic_detail')}`,
    ...(imageUrl ? { imageUrl } : {}) }
}

/** 结果类字眼越重越靠前：官宣、夺冠、降级这类比「表示」「回顾」更能让人点进来。 */
const IMPACT_WORDS: [RegExp, number][] = [
  [/官宣|正式|确认|宣布/, 3], [/夺冠|冠军|降级|升入|冲超|逆转|绝杀|崩盘|丢冠/, 3],
  [/离队|加盟|签约|续约|解约|下课|换帅|留队|转会/, 2], [/\d+\s*[-:比]\s*\d+/, 2], [/传闻|据报道|有意|接触|商谈|谈判/, 1],
]
function impact(entry: DailyTopicEntry): number {
  const text = entry.title + entry.text
  return IMPACT_WORDS.reduce((score, [pattern, weight]) => score + (pattern.test(text) ? weight : 0), 0)
}
/** 分享用的几条看点：分量最重的优先，同分取更新的；输出仍按时间新到旧。 */
export function roundupShareHighlights(item: DailyTopicItem, count = 3): DailyTopicEntry[] {
  const ranked = item.entries.map((entry, index) => ({ entry, index, score: impact(entry) }))
    .sort((a, b) => b.score - a.score || a.index - b.index).slice(0, count)
  return ranked.sort((a, b) => a.index - b.index).map(({ entry }) => entry)
}
function roundupDate(item: DailyTopicItem): string {
  const date = new Date(Date.parse(item.publishedAt) + 8 * 60 * 60 * 1000)
  return `${date.getUTCMonth() + 1}月${date.getUTCDate()}日`
}
export function roundupShareTitle(item: DailyTopicItem): string {
  const [lead] = roundupShareHighlights(item, 1)
  return `${roundupDate(item)}足坛${item.entries.length}条快讯｜${lead.title}`
}
export function roundupShareDate(item: DailyTopicItem): string { return roundupDate(item) }

/** 分享正文开头，不拿摘要、来源或账号信息代替文章全文。 */
export function topicShareExcerpt(item: DailyTopicItem): string {
  return item.paragraphs.map(paragraph => paragraph.trim()).filter(Boolean).join(' ')
}

export { wrapShareCardText } from '../../utils/shareCardText'

export function topicShareSignature(item: DailyTopicItem): string {
  const firstPhoto = item.images?.[0]
  return JSON.stringify([item.id, item.title, item.publishedLabel, item.paragraphs,
    firstPhoto ? [firstPhoto.url, firstPhoto.width, firstPhoto.height] : null])
}

export interface SharePhotoRect { x: number; y: number; width: number; height: number }

export function containSharePhoto(width: number, height: number, area: SharePhotoRect): SharePhotoRect | null {
  if (![width, height, area.width, area.height].every(value => Number.isFinite(value) && value > 0)
    || !Number.isFinite(area.x) || !Number.isFinite(area.y)) return null
  const scale = Math.min(area.width / width, area.height / height)
  const displayWidth = width * scale, displayHeight = height * scale
  return { x: area.x + (area.width - displayWidth) / 2, y: area.y + (area.height - displayHeight) / 2,
    width: displayWidth, height: displayHeight }
}
