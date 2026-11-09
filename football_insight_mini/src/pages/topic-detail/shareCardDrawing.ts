import type { DailyTopicItem } from '../../types/dailyTopic'
import { containSharePhoto, roundupShareDate, roundupShareHighlights, TOPIC_SHARE_CARD, topicShareExcerpt, wrapShareCardText } from './share'
import { isRoundupTopic } from '../../utils/dailyTopics'
import type { TopicSharePhoto } from './shareCardPhoto'

/** Canvas 颜色属于导出的分享图片，使用明确的绘制颜色。 */
export function drawTopicShareCard(context: UniApp.CanvasContext, item: DailyTopicItem, photo: TopicSharePhoto | null): void {
  if (isRoundupTopic(item)) { drawRoundupShareCard(context, item); return }
  const { width, height } = TOPIC_SHARE_CARD
  context.setFillStyle('#f6f5f2')
  context.fillRect(0, 0, width, height)
  context.setFillStyle('#ffffff')
  context.fillRect(28, 28, width - 56, height - 56)
  context.setFillStyle('#eb3f41')
  context.setFontSize(28)
  context.fillText('足球洞察 · 足球新闻', 64, 84)
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
  context.fillText('足球新闻', width - 228, height - 64)
}

function font(context: UniApp.CanvasContext, size: number, bold = false): void {
  context.setFontSize(size)
  // 旧版 CanvasContext 只有 setFontSize；支持 font 属性的基础库才能加粗，不支持时仍按字号绘制。
  ;(context as unknown as { font: string }).font = `${bold ? 'bold ' : ''}${size}px sans-serif`
}

/**
 * 每日电报分享图：聊天列表里卡片只有拇指大小，所以只放大日期、条数和三条最有分量的标题，
 * 深底白字红点缀，底部一条红色行动按钮。
 */
function drawRoundupShareCard(context: UniApp.CanvasContext, item: DailyTopicItem): void {
  const { width, height } = TOPIC_SHARE_CARD
  const left = 64, textWidth = width - left * 2
  context.setFillStyle('#121318')
  context.fillRect(0, 0, width, height)
  context.setFillStyle('#eb3f41')
  context.fillRect(0, 0, 14, height - 112)

  context.fillRect(left, 52, 196, 56)
  context.setFillStyle('#ffffff')
  font(context, 32, true)
  context.fillText('每日电报', left + 34, 92)
  context.setFillStyle('#8c8f99')
  font(context, 28)
  context.setTextAlign('right')
  context.fillText('足球洞察', width - left, 92)
  context.setTextAlign('left')

  context.setFillStyle('#ffffff')
  font(context, 96, true)
  const date = roundupShareDate(item)
  context.fillText(date, left, 222)
  const dateWidth = context.measureText(date).width
  context.setFillStyle('#eb3f41')
  font(context, 44, true)
  context.fillText(`${item.entries.length} 条快讯`, left + dateWidth + 28, 222)

  roundupShareHighlights(item, 3).forEach((entry, index) => {
    const top = 300 + index * 128
    context.setFillStyle('#eb3f41')
    font(context, 30, true)
    context.fillText(entry.timeLabel, left, top)
    context.setFillStyle('#ffffff')
    font(context, 48, true)
    const [line] = wrapShareCardText(entry.title, text => context.measureText(text).width, textWidth, 1)
    context.fillText(line ?? '', left, top + 60)
  })

  context.setFillStyle('#eb3f41')
  context.fillRect(0, height - 112, width, 112)
  context.setFillStyle('#ffffff')
  font(context, 40, true)
  context.setTextAlign('center')
  context.fillText(`点开看全部 ${item.entries.length} 条  ›`, width / 2, height - 42)
  context.setTextAlign('left')
}
