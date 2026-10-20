export interface RecapPosterImage { path: string; width: number; height: number }
type ImagePlatform = Pick<typeof uni, 'getImageInfo'>

/** 装饰图并行限时转本地路径；失败、无效尺寸和迟到回调均不会阻断海报。 */
export async function loadRecapPosterImages(
 urls: string[],
 allowWebp: boolean,
 platform: ImagePlatform = uni,
 timeoutMs = 4000,
): Promise<Map<string, RecapPosterImage>> {
 const entries = await Promise.all([...new Set(urls)].map(url => new Promise<[string, RecapPosterImage] | null>(resolve => {
  let settled = false
  const finish = (image: RecapPosterImage | null) => {
   if (settled) return
   settled = true
   clearTimeout(timer)
   resolve(image ? [url, image] : null)
  }
  const timer = setTimeout(() => finish(null), timeoutMs)
  if (!allowWebp && /\.webp(?:[?#]|$)/i.test(url)) { finish(null); return }
  try {
   platform.getImageInfo({
    src: url,
    success: image => {
     if (!allowWebp && image.type?.toLowerCase() === 'webp') { finish(null); return }
     finish(image.path && image.width > 0 && image.height > 0
      ? { path: image.path, width: image.width, height: image.height } : null)
    },
    fail: () => finish(null),
   })
  } catch { finish(null) }
 })))
 return new Map(entries.filter((entry): entry is [string, RecapPosterImage] => entry !== null))
}
