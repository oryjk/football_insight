import { expect, test } from 'bun:test'
import { loadTopicSharePhoto } from './shareCardPhoto'

type DownloadOptions = Parameters<typeof uni.downloadFile>[0]
type ImageOptions = Parameters<typeof uni.getImageInfo>[0]

async function withMedia(media: object, run: () => Promise<void>) {
  const runtime = globalThis as unknown as { uni: typeof uni }
  const original = runtime.uni
  runtime.uni = media as typeof uni
  try { await run() } finally { runtime.uni = original }
}

test('share photo uses the downloaded local file and decoded dimensions', async () => {
  await withMedia({
    downloadFile(options: DownloadOptions) {
      expect(options.url).toBe('https://media.example.com/first.jpg')
      options.success?.({ statusCode: 200, tempFilePath: '/tmp/first.jpg' })
      return { abort() {} }
    },
    getImageInfo(options: ImageOptions) {
      expect(options.src).toBe('/tmp/first.jpg')
      options.success?.({ path: '/tmp/decoded.jpg', width: 640, height: 426, orientation: 'up', type: 'jpeg' })
    },
  }, async () => {
    expect(await loadTopicSharePhoto('https://media.example.com/first.jpg')).toEqual({ path: '/tmp/decoded.jpg', width: 640, height: 426 })
  })
})

test('missing or unsafe image does not prevent a text share card', async () => {
  await withMedia({ downloadFile() { throw new Error('must not request an invalid image') } }, async () => {
    for (const url of [undefined, '', 'http://media.example.com/a.jpg', 'https://user:secret@media.example.com/a.jpg']) {
      expect(await loadTopicSharePhoto(url)).toBeNull()
    }
  })
})

test('failed download falls back instead of using an error document as a picture', async () => {
  await withMedia({ downloadFile(options: DownloadOptions) {
    options.success?.({ statusCode: 404, tempFilePath: '/tmp/error.html' })
    return { abort() {} }
  } }, async () => { expect(await loadTopicSharePhoto('https://media.example.com/a.jpg')).toBeNull() })
})

test('image loading times out and ignores a late completed download', async () => {
  let options: DownloadOptions | null = null, aborted = false, decoded = false
  await withMedia({
    downloadFile(value: DownloadOptions) { options = value; return { abort() { aborted = true } } },
    getImageInfo() { decoded = true },
  }, async () => {
    expect(await loadTopicSharePhoto('https://media.example.com/a.jpg', 20)).toBeNull()
    expect(aborted).toBe(true)
    const late = options as DownloadOptions | null
    late?.success?.({ statusCode: 200, tempFilePath: '/tmp/late.jpg' })
    expect(decoded).toBe(false)
  })
})
