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
