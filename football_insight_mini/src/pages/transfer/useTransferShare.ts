import { getCurrentInstance, nextTick, onScopeDispose, ref, watch } from 'vue'
import { getTransferCode } from '../../api/transfer'
import { transferVoteDeps } from '../../composables/useTransferVotes'
import { useAppSession } from '../../stores/appSession'
import type { TransferCardView } from '../../types/transfer'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { avatarCandidates } from '../../utils/avatarThumb'
import { exportCanvasToTempFile, flushCanvas, materializePosterCode, savePosterWithPermission } from '../../utils/canvasPoster'
import { drawTransferPoster, TRANSFER_POSTER, type TransferSharer } from './transferPoster'

/** 单卡页"分享拉票"：好友卡片走页面 onShareAppMessage；这里负责生成、预览、保存海报。 */
export function useTransferShare(contextKey: () => string, returnUrl: () => string) {
  const instance = getCurrentInstance(), deps = transferVoteDeps(), session = useAppSession()
  const snapshot = ref<TransferCardView | null>(null), visible = ref(false), busy = ref(false), error = ref(''), imagePath = ref('')
  let generation = 0, disposed = false
  // 画布同一时间只给一次绘制使用，避免新的绘制覆盖仍在导出的旧图。
  let rendering: Promise<unknown> = Promise.resolve()
  function reset(): void { generation++; snapshot.value = null; visible.value = false; busy.value = false; imagePath.value = ''; error.value = '' }
  watch(contextKey, reset, { flush: 'sync' })

  function imageInfo(src: string): Promise<string> {
    return new Promise((resolve, reject) => uni.getImageInfo({ src, success: result => resolve(result.path), fail: reject }))
  }
  // 海报大头像优先取 240px 小图，小图缺失或加载失败时回退原图；结果仍以原地址为键，绘制函数不用改。
  async function loadImage(url: string): Promise<string> {
    for (const candidate of avatarCandidates(url, 240)) {
      try { return await imageInfo(candidate) } catch { /* 试下一个 */ }
    }
    throw new Error('image unavailable')
  }
  async function render(view: TransferCardView, codePath: string, sharer: TransferSharer, current: () => boolean): Promise<string> {
    const previous = rendering
    let release: () => void = () => {}
    rendering = new Promise<void>(resolve => { release = resolve })
    await previous.catch(() => {})
    try {
      if (!current()) return ''
      const urls = [...new Set([view.team.avatar_url, view.card.player.avatar_url, sharer.avatar_url].filter((url): url is string => Boolean(url)))]
      const images = new Map<string, string>()
      // 头像或队徽加载失败不阻断海报，绘制时用文字占位。
      await Promise.all(urls.map(async url => { try { images.set(url, await loadImage(url)) } catch { /* fallback */ } }))
      await nextTick()
      if (!current()) return ''
      const canvas = uni.createCanvasContext(TRANSFER_POSTER.canvasId, instance)
      drawTransferPoster(canvas, view, images, codePath, sharer)
      await flushCanvas(canvas)
      if (!current()) return ''
      const path = await exportCanvasToTempFile({ ...TRANSFER_POSTER, fileType: 'png', instance })
      return current() ? path : ''
    } finally { release() }
  }

  function open(view: TransferCardView): void {
    if (disposed) return
    reset(); snapshot.value = view; visible.value = true
  }

  async function generate(): Promise<void> {
    const view = snapshot.value
    if (!view || busy.value || disposed) return
    if (!deps.loggedIn()) { deps.login(returnUrl()); return }
    const owner = generation, key = contextKey(), revision = deps.revision()
    const current = () => !disposed && owner === generation && key === contextKey() && revision === deps.revision()
    busy.value = true; error.value = ''; imagePath.value = ''
    try {
      const user = await session.ensureSessionReady()
      if (!current()) return
      if (!user) { deps.login(returnUrl()); return }
      const code = await getTransferCode(view.card.share_code)
      if (!current()) return
      const materialized = await materializePosterCode(code)
      try {
        const codePath = await imageInfo(materialized.path)
        if (!current()) return
        const path = await render(view, codePath, { display_name: user.display_name, avatar_url: user.avatar_url }, current)
        if (current()) imagePath.value = path
      } finally { materialized.cleanup() }
    } catch (cause) {
      if (!current()) return
      if (isUnauthorizedError(cause)) { deps.invalidate(revision); deps.login(returnUrl()); return }
      error.value = extractApiErrorMessage(cause, '分享图生成失败，请重试')
    } finally { if (current()) busy.value = false }
  }
  function preview(): void { if (imagePath.value) uni.previewImage({ urls: [imagePath.value] }) }
  function save(): void {
    const path = imagePath.value, owner = generation
    if (path) void savePosterWithPermission(path, uni, () => !disposed && generation === owner && imagePath.value === path)
  }
  onScopeDispose(() => { disposed = true; reset() })
  return { visible, busy, error, imagePath, open, generate, preview, save, reset }
}
