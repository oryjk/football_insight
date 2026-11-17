import { getCurrentInstance, nextTick, onScopeDispose, ref } from 'vue'
import type { DailyTopicEntry } from '../../types/dailyTopic'
import { exportCanvasToTempFile, flushCanvas } from '../../utils/canvasPoster'
import {
  drawTelegraphEntryShareCard,
  TELEGRAPH_SHARE_CARD,
  telegraphEntryShareSignature,
} from '../../utils/telegraphShareCard'

export const TELEGRAPH_ENTRY_SHARE_CANVAS_ID = 'telegraphEntryShareCanvas'

export interface PrepareTelegraphEntryShareOptions {
  publishedAt?: string
  commentsVisible?: boolean
}

export function useTelegraphEntryShareCard() {
  const instance = getCurrentInstance()
  const imageUrl = ref('')
  let revision = 0
  let signature = ''
  let pending: Promise<string> | null = null
  // 同一个 canvas 的绘制串行执行，避免连续点赞或评论时导出到上一帧旧内容。
  let drawing: Promise<unknown> = Promise.resolve()

  function reset(): void {
    revision += 1
    imageUrl.value = ''
    signature = ''
    pending = null
  }

  function prepare(entry: DailyTopicEntry, options?: PrepareTelegraphEntryShareOptions): Promise<string> {
    const nextSignature = telegraphEntryShareSignature(entry, options)
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
      const context = uni.createCanvasContext(TELEGRAPH_ENTRY_SHARE_CANVAS_ID, instance)
      const { width, height } = TELEGRAPH_SHARE_CARD
      drawTelegraphEntryShareCard(context, entry, options)
      await flushCanvas(context)
      if (owner !== revision) return ''
      const path = await exportCanvasToTempFile({
        canvasId: TELEGRAPH_ENTRY_SHARE_CANVAS_ID,
        width,
        height,
        fileType: 'png',
        instance,
      })
      if (owner !== revision) return ''
      imageUrl.value = path
      return path
    }).catch(() => '')
    const request = pending
    drawing = request
    void request.finally(() => {
      if (owner === revision) pending = null
    })
    return request
  }

  onScopeDispose(reset)
  return { imageUrl, prepare, reset }
}
