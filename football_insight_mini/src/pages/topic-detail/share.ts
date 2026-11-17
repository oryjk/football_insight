import type { DailyTopicItem } from '../../types/dailyTopic'
import { buildSharePath, buildShareQuery } from '../../utils/shareSource'
import { isRoundupTopic, topicDetailUrl } from '../../utils/dailyTopics'
import {
  roundupShareDate,
  roundupShareHighlights,
  roundupShareTitle,
  TELEGRAPH_SHARE_CARD,
} from '../../utils/telegraphShareCard'

export const TOPIC_SHARE_CARD = TELEGRAPH_SHARE_CARD
export { roundupShareDate, roundupShareHighlights, roundupShareTitle }

export function buildTopicSharePayload(item: DailyTopicItem | null, imageUrl?: string) {
  if (!item || !/^[a-zA-Z0-9_-]{1,128}$/.test(item.id)) {
    return { title: '足球洞察 · 足球新闻', path: buildSharePath('/pages/topics/index', 'topic_detail') }
  }
  return {
    title: isRoundupTopic(item) ? roundupShareTitle(item) : item.title,
    path: buildSharePath(topicDetailUrl(item.id), 'topic_detail'),
    ...(imageUrl ? { imageUrl } : {}),
  }
}

export function buildTopicTimelinePayload(item: DailyTopicItem | null, imageUrl?: string) {
  const share = buildTopicSharePayload(item, imageUrl)
  const id = item && /^[a-zA-Z0-9_-]{1,128}$/.test(item.id) ? `id=${encodeURIComponent(item.id)}&` : ''
  return {
    title: share.title,
    query: `${id}${buildShareQuery('topic_detail')}`,
    ...(imageUrl ? { imageUrl } : {}),
  }
}

/** 分享正文开头，不拿摘要、来源或账号信息代替文章全文。 */
export function topicShareExcerpt(item: DailyTopicItem): string {
  return item.paragraphs.map(paragraph => paragraph.trim()).filter(Boolean).join(' ')
}

export { wrapShareCardText } from '../../utils/shareCardText'

export function topicShareSignature(item: DailyTopicItem): string {
  const firstPhoto = item.images?.[0]
  const base: unknown[] = [
    item.id,
    item.title,
    item.publishedLabel,
    item.paragraphs,
    firstPhoto ? [firstPhoto.url, firstPhoto.width, firstPhoto.height] : null,
  ]
  if (isRoundupTopic(item)) {
    base.push(item.entries.map(entry => [entry.id, entry.category, entry.title, entry.timeLabel]))
  }
  return JSON.stringify(base)
}

export interface SharePhotoRect { x: number; y: number; width: number; height: number }

export function containSharePhoto(width: number, height: number, area: SharePhotoRect): SharePhotoRect | null {
  if (![width, height, area.width, area.height].every(value => Number.isFinite(value) && value > 0)
    || !Number.isFinite(area.x) || !Number.isFinite(area.y)) return null
  const scale = Math.min(area.width / width, area.height / height)
  const displayWidth = width * scale, displayHeight = height * scale
  return {
    x: area.x + (area.width - displayWidth) / 2,
    y: area.y + (area.height - displayHeight) / 2,
    width: displayWidth,
    height: displayHeight,
  }
}
