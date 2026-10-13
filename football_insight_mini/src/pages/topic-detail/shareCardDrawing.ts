import type { DailyTopicItem } from '../../types/dailyTopic'
import { containSharePhoto, TOPIC_SHARE_CARD, topicShareExcerpt, wrapShareCardText } from './share'
import type { TopicSharePhoto } from './shareCardPhoto'

/** Canvas 颜色属于导出的分享图片，使用明确的绘制颜色。 */
export function drawTopicShareCard(context: UniApp.CanvasContext, item: DailyTopicItem, photo: TopicSharePhoto | null): void {
  const { width, height } = TOPIC_SHARE_CARD
  context.setFillStyle('#f6f5f2')
  context.fillRect(0, 0, width, height)
  context.setFillStyle('#ffffff')
  context.fillRect(28, 28, width - 56, height - 56)
  context.setFillStyle('#eb3f41')
  context.setFontSize(28)
  context.fillText('足球洞察 · 成都蓉城', 64, 84)
  context.setFillStyle('#15161b')
  context.setFontSize(52)
  const titleLines = wrapShareCardText(item.title, text => context.measureText(text).width, width - 128, 2)
  titleLines.forEach((text, index) => context.fillText(text, 64, 160 + index * 68))
  context.setFillStyle('#8c8f99')
  context.setFontSize(26)
  const metaY = 160 + titleLines.length * 68
  context.fillText(item.publishedLabel, 64, metaY)

  let textX = 64, textY = metaY + 62, textWidth = width - 128, maxLines = 7
  if (photo) {
    const area = { x: 64, y: metaY + 40, width: 420, height: 300 }
    const rect = containSharePhoto(photo.width, photo.height, area)
    if (rect) {
      context.setFillStyle('#f6f5f2')
      context.fillRect(area.x, area.y, area.width, area.height)
      context.drawImage(photo.path, rect.x, rect.y, rect.width, rect.height)
      textX = area.x + area.width + 28
      textY = area.y + 36
      textWidth = width - 64 - textX
      maxLines = Math.min(7, Math.floor((height - 132 - textY) / 52) + 1)
    }
  }
  context.setFillStyle('#3f424b')
  context.setFontSize(36)
  const bodyLines = wrapShareCardText(topicShareExcerpt(item), text => context.measureText(text).width, textWidth, maxLines)
  bodyLines.forEach((text, index) => context.fillText(text, textX, textY + index * 52))
  context.setFillStyle('#eb3f41')
  context.setFontSize(28)
  context.fillText('打开阅读全文', 64, height - 64)
  context.setFillStyle('#8c8f99')
  context.setFontSize(24)
  context.fillText('成都蓉城新闻', width - 228, height - 64)
}
