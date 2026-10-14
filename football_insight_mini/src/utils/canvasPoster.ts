import type { ComponentInternalInstance } from 'vue'
import { extractApiErrorMessage } from './apiError'

/** 海报类 canvas 的共用原语：圆角路径、提交绘制、导出临时图片、保存到相册。 */

export function traceRoundRect(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  context.beginPath()
  context.moveTo(x + radius, y)
  context.arcTo(x + width, y, x + width, y + height, radius)
  context.arcTo(x + width, y + height, x, y + height, radius)
  context.arcTo(x, y + height, x, y, radius)
  context.arcTo(x, y, x + width, y, radius)
  context.closePath()
}

export function flushCanvas(context: UniApp.CanvasContext): Promise<void> {
  return new Promise((resolve) => {
    context.draw(false, () => resolve())
  })
}

export interface CanvasExportOptions {
  canvasId: string
  width: number
  height: number
  fileType?: 'png' | 'jpg'
  /** 组件内 canvas 需要传组件实例，页面级 canvas 省略。 */
  instance?: ComponentInternalInstance | null
}

export function exportCanvasToTempFile(options: CanvasExportOptions): Promise<string> {
  const { canvasId, width, height, fileType, instance } = options
  return new Promise((resolve, reject) => {
    uni.canvasToTempFilePath({
      canvasId,
      width,
      height,
      destWidth: width,
      destHeight: height,
      ...(fileType ? { fileType } : {}),
      success: result => resolve(result.tempFilePath),
      fail: reject,
    }, instance ?? undefined)
  })
}

export function savePosterToAlbum(filePath: string): void {
  if (!filePath) {
    return
  }

  uni.saveImageToPhotosAlbum({
    filePath,
    success: () => {
      uni.showToast({ title: '已保存到相册', icon: 'success' })
    },
    fail: (error) => {
      uni.showToast({ title: extractApiErrorMessage(error, '保存失败'), icon: 'none' })
    },
  })
}
