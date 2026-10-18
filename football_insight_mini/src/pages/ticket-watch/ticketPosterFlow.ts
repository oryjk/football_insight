import { ref } from 'vue'
import type { PosterCodeImage } from '../../utils/canvasPoster'
import type { TicketPosterContent } from './sharePoster'

export interface TicketPosterDependencies {
  getCode: () => Promise<PosterCodeImage>
  render: (content: TicketPosterContent, code: PosterCodeImage, isCurrent: () => boolean) => Promise<string>
}

/** Single canvas queue plus operation ownership: reset never lets an old task publish. */
export function createTicketPosterFlow(dependencies: TicketPosterDependencies) {
  const imagePath = ref(''), generating = ref(false), error = ref('')
  let revision = 0
  let drawing: Promise<unknown> = Promise.resolve()
  function reset(): void { revision++; imagePath.value = ''; generating.value = false; error.value = '' }
  async function generate(content: TicketPosterContent): Promise<void> {
    if (generating.value) return
    reset()
    const owner = revision
    const isCurrent = () => owner === revision
    generating.value = true
    try {
      let code: PosterCodeImage
      try { code = await dependencies.getCode() } catch {
        if (isCurrent()) error.value = '二维码暂时获取失败，请重试'
        return
      }
      if (!isCurrent()) return
      const job = drawing.then(async () => isCurrent() ? dependencies.render(content, code, isCurrent) : '')
      drawing = job.catch(() => {})
      const path = await job
      if (isCurrent()) {
        if (!path) throw new Error('empty poster export')
        imagePath.value = path
      }
    } catch {
      if (isCurrent()) error.value = '分享图生成失败，请重试'
    } finally {
      if (isCurrent()) generating.value = false
    }
  }
  return { imagePath, generating, error, generate, reset }
}
