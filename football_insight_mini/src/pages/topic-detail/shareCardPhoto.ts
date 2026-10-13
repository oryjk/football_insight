import { isTopicSourceUrl } from '../../utils/dailyTopics'

export interface TopicSharePhoto { path: string; width: number; height: number }

/** 下载首图到 canvas 可安全使用的本地路径，加载超时仍允许分享正文。 */
export function loadTopicSharePhoto(url: string | undefined, timeoutMs = 1500): Promise<TopicSharePhoto | null> {
  if (!isTopicSourceUrl(url) || !url.startsWith('https://')) return Promise.resolve(null)
  return new Promise(resolve => {
    let settled = false
    let download: UniApp.DownloadTask | null = null
    const finish = (photo: TopicSharePhoto | null) => {
      if (settled) return
      settled = true
      clearTimeout(timer)
      resolve(photo)
    }
    const timer = setTimeout(() => {
      finish(null)
      try { download?.abort() } catch { /* 已结束的下载无需再次取消。 */ }
    }, timeoutMs)
    try {
      download = uni.downloadFile({
        url, timeout: timeoutMs,
        success: result => {
          if (settled) return
          if (result.statusCode !== 200 || !result.tempFilePath) { finish(null); return }
          try {
            uni.getImageInfo({ src: result.tempFilePath,
              success: image => {
                if (!image.path || ![image.width, image.height].every(value => Number.isFinite(value) && value > 0)) {
                  finish(null); return
                }
                finish({ path: image.path, width: image.width, height: image.height })
              }, fail: () => finish(null),
            })
          } catch { finish(null) }
        }, fail: () => finish(null),
      })
    } catch { finish(null) }
  })
}
