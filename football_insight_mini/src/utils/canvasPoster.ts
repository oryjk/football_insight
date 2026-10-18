import type { ComponentInternalInstance } from 'vue'
import { extractApiErrorMessage } from './apiError'
import type { ShareCodeImage } from '../types/shareCode'

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

type AlbumPlatform = Pick<typeof uni, 'getSetting' | 'showModal' | 'openSetting' | 'saveImageToPhotosAlbum' | 'showToast'>

/** Shared permission flow, preserved from the season recap poster. */
export async function savePosterWithPermission(path: string, platform: AlbumPlatform = uni, isCurrent = () => true): Promise<void> {
  if (!path || !isCurrent()) return
  const denied = () => new Promise<boolean>(resolve => platform.getSetting({
    success: result => resolve(result.authSetting['scope.writePhotosAlbum'] === false),
    fail: () => resolve(false),
  }))
  const offerSettings = () => platform.showModal({
    title: '需要相册权限', content: '请在设置中允许保存到相册，返回后再次点击保存。', confirmText: '去设置',
    success: result => { if (result.confirm && isCurrent()) platform.openSetting({}) },
  })
  const permissionDenied = await denied()
  if (!isCurrent()) return
  if (permissionDenied) { offerSettings(); return }
  await new Promise<void>(resolve => platform.saveImageToPhotosAlbum({
    filePath: path,
    success: () => { if (isCurrent()) platform.showToast({ title: '已保存到相册', icon: 'success' }); resolve() },
    fail: async () => {
      const permissionDenied = await denied()
      if (isCurrent()) {
        if (permissionDenied) offerSettings()
        else platform.showToast({ title: '保存失败，请重试', icon: 'none' })
      }
      resolve()
    },
  }))
}

export type PosterCodeImage = ShareCodeImage

/** Local image for WeChat canvas; data URL for H5. Caller owns cleanup after export. */
export async function materializePosterCode(code: PosterCodeImage): Promise<{ path: string; cleanup: () => void }> {
  // #ifdef MP-WEIXIN
  const fs = uni.getFileSystemManager()
  const extension = code.mime_type === 'image/jpeg' ? 'jpg' : 'png'
  const path = `${(uni as typeof uni & { env: { USER_DATA_PATH: string } }).env.USER_DATA_PATH}/fi-poster-code-${Date.now()}.${extension}`
  await new Promise<void>((resolve, reject) => fs.writeFile({ filePath: path, data: code.image_base64, encoding: 'base64', success: () => resolve(), fail: reject }))
  return { path, cleanup: () => { fs.unlink({ filePath: path, fail: () => {} }) } }
  // #endif
  // #ifndef MP-WEIXIN
  return { path: `data:${code.mime_type};base64,${code.image_base64}`, cleanup: () => {} }
  // #endif
}
