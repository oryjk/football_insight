import { getCurrentInstance, nextTick, onScopeDispose, ref } from 'vue'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { TOPIC_SHARE_CARD, topicShareSignature } from './share'
import { drawTopicShareCard } from './shareCardDrawing'
import { loadTopicSharePhoto } from './shareCardPhoto'
import { exportCanvasToTempFile, flushCanvas } from '../../utils/canvasPoster'

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
      const photo = await loadTopicSharePhoto(item.images?.[0]?.url)
      if (owner !== revision) return ''
      const context = uni.createCanvasContext(TOPIC_SHARE_CANVAS_ID, instance)
      const { width, height } = TOPIC_SHARE_CARD
      drawTopicShareCard(context, item, photo)
      await flushCanvas(context)
      if (owner !== revision) return ''
      const path = await exportCanvasToTempFile({ canvasId: TOPIC_SHARE_CANVAS_ID, width, height, fileType: 'png', instance })
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
