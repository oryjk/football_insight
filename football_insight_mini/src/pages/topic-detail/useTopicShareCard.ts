import { getCurrentInstance, nextTick, onScopeDispose, ref } from 'vue'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { TOPIC_SHARE_CARD, topicShareExcerpt, topicShareSignature, wrapShareCardText } from './share'

export const TOPIC_SHARE_CANVAS_ID = 'topicShareCanvas'

export function useTopicShareCard() {
  const instance = getCurrentInstance()
  const imageUrl = ref('')
  let revision = 0
  let signature = ''
  let pending: Promise<string> | null = null
  // 同一个 canvas 的绘制串行执行，旧稿画完后新稿才开始，避免导出到旧正文。
  let drawing: Promise<unknown> = Promise.resolve()

  function reset(): void { revision += 1; imageUrl.value = ''; signature = ''; pending = null }

  function prepare(item: DailyTopicItem): Promise<string> {
    const nextSignature = topicShareSignature(item)
    if (signature === nextSignature) {
      if (imageUrl.value) return Promise.resolve(imageUrl.value)
      if (pending) return pending
    }
    reset()
    signature = nextSignature
    const owner = revision
    pending = drawing.then(async () => {
      await nextTick()
      if (owner !== revision) return ''
      const context = uni.createCanvasContext(TOPIC_SHARE_CANVAS_ID, instance)
      const { width, height } = TOPIC_SHARE_CARD
      // Canvas 配色属于分享图片，使用明确的绘制颜色，不受页面主题影响。
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
      context.setFillStyle('#3f424b')
      context.setFontSize(36)
      const bodyLines = wrapShareCardText(topicShareExcerpt(item), text => context.measureText(text).width, width - 128, 7)
      bodyLines.forEach((text, index) => context.fillText(text, 64, metaY + 62 + index * 52))
      context.setFillStyle('#eb3f41')
      context.setFontSize(28)
      context.fillText('打开阅读全文', 64, height - 64)
      context.setFillStyle('#8c8f99')
      context.setFontSize(24)
      context.fillText('成都蓉城新闻', width - 228, height - 64)
      await new Promise<void>(resolve => context.draw(false, () => resolve()))
      if (owner !== revision) return ''
      const path = await new Promise<string>((resolve, reject) => {
        uni.canvasToTempFilePath({ canvasId: TOPIC_SHARE_CANVAS_ID, width, height, destWidth: width, destHeight: height,
          fileType: 'png', success: result => resolve(result.tempFilePath), fail: reject }, instance)
      })
      if (owner !== revision) return ''
      imageUrl.value = path
      return path
    }).catch(() => '')
    const request = pending
    drawing = request
    void request.finally(() => { if (owner === revision) pending = null })
    return request
  }

  onScopeDispose(reset)
  return { imageUrl, prepare, reset }
}
