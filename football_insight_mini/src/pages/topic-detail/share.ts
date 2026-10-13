import type { DailyTopicItem } from '../../types/dailyTopic'
import { topicDetailUrl } from '../../utils/dailyTopics'

export const TOPIC_SHARE_CARD = { width: 1000, height: 800 } as const

export function buildTopicSharePayload(item: DailyTopicItem | null, imageUrl?: string) {
  if (!item || !/^[a-zA-Z0-9_-]{1,128}$/.test(item.id)) {
    return { title: '足球洞察 · 成都蓉城新闻', path: '/pages/topics/index' }
  }
  return { title: item.title, path: topicDetailUrl(item.id), ...(imageUrl ? { imageUrl } : {}) }
}

/** 分享正文开头，不拿摘要、来源或账号信息代替文章全文。 */
export function topicShareExcerpt(item: DailyTopicItem): string {
  return item.paragraphs.map(paragraph => paragraph.trim()).filter(Boolean).join(' ')
}

export function wrapShareCardText(
  text: string, measure: (text: string) => number, width: number, maxLines: number,
): string[] {
  if (width <= 0 || maxLines <= 0) return []
  const lines: string[] = []
  let line = ''
  for (const character of Array.from(text.replace(/\s+/g, ' ').trim())) {
    if (!line || measure(line + character) <= width) { line += character; continue }
    lines.push(line.trim())
    if (lines.length === maxLines) {
      const last = Array.from(lines[lines.length - 1])
      while (last.length && measure(last.join('') + '…') > width) last.pop()
      lines[lines.length - 1] = last.join('') + '…'
      return lines
    }
    line = character.trimStart()
  }
  if (line) lines.push(line.trim())
  return lines
}

export function topicShareSignature(item: DailyTopicItem): string {
  return JSON.stringify([item.id, item.title, item.publishedLabel, item.paragraphs])
}
