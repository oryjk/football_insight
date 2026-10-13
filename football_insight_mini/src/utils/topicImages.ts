import type { DailyTopicItem } from '../types/dailyTopic'

export function topicReadingBlocks(item: DailyTopicItem) {
  return item.paragraphs.map((text, index) => ({
    text, images: (item.images ?? []).filter(image => image.afterParagraph === index + 1),
  }))
}
