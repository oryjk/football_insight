import { computed, getCurrentInstance, nextTick, onScopeDispose, ref, watch } from 'vue'
import { getLineupCode } from '../../api/lineup'
import type { PublicLineup } from '../../types/lineup'
import { extractApiErrorMessage, isUnauthorizedError } from '../../utils/apiError'
import { exportCanvasToTempFile, flushCanvas, materializePosterCode, savePosterWithPermission } from '../../utils/canvasPoster'
import { communityDeps } from './lineupCommunity'
import { drawLineupPoster, LINEUP_CARD, LINEUP_POSTER, lineupSharePayload } from './lineupPoster'

export function useLineupShare(contextKey: () => string, returnUrl: () => string) {
  const instance = getCurrentInstance(), deps = communityDeps()
  const snapshot = ref<PublicLineup | null>(null), visible = ref(false), busy = ref(false), error = ref('')
  const cardPath = ref(''), imagePath = ref('')
  let generation = 0, disposed = false
  // Keep the canvas exclusively owned until the current draw/export finishes. A replacement
  // context may open immediately, but cannot draw over an older export still in flight.
  let rendering: Promise<unknown> = Promise.resolve()
  function reset(): void { generation++; snapshot.value = null; visible.value = false; busy.value = false; imagePath.value = ''; cardPath.value = ''; error.value = '' }
  watch(contextKey, reset, { flush: 'sync' })
  const payload = computed(() => snapshot.value ? lineupSharePayload(snapshot.value, cardPath.value) : null)
  async function render(lineup: PublicLineup, codePath: string, card: boolean, current: () => boolean): Promise<string> {
    const previous = rendering
    let release: () => void = () => {}
    rendering = new Promise<void>(resolve => { release = resolve })
    await previous.catch(() => {})
    try {
      if (!current()) return ''
      const urls = [...new Set([...lineup.players.map(player => player.avatar_url), lineup.author.avatar_url, lineup.team.avatar_url].filter((url): url is string => Boolean(url)))]
      const images = new Map<string, string>()
      await Promise.all(urls.map(async url => { try { const path = await imageInfo(url); images.set(url, path) } catch { /* number fallback */ } }))
      await nextTick()
      if (!current()) return ''
      const size = card ? LINEUP_CARD : LINEUP_POSTER
      const canvas = uni.createCanvasContext(size.canvasId, instance)
      drawLineupPoster(canvas, lineup, images, codePath, card)
      await flushCanvas(canvas)
      if (!current()) return ''
      const path = await exportCanvasToTempFile({ ...size, fileType: 'png', instance })
      return current() ? path : ''
    } finally { release() }
  }
  function imageInfo(src: string): Promise<string> {
    return new Promise((resolve, reject) => uni.getImageInfo({ src, success: result => resolve(result.path), fail: reject }))
  }
  async function open(lineup: PublicLineup): Promise<void> {
    if (disposed || busy.value) return
    reset(); snapshot.value = lineup; visible.value = true; busy.value = true
    const owner = generation, key = contextKey(), revision = deps.revision()
    const current = () => !disposed && owner === generation && key === contextKey() && revision === deps.revision()
    try { const path = await render(lineup, '', true, current); if (current()) cardPath.value = path }
    catch { /* native card uses the current page screenshot if drawing is unavailable */ }
    finally { if (current()) busy.value = false }
  }
  async function generate(): Promise<void> {
    if (!snapshot.value || busy.value || disposed) return
    if (!deps.loggedIn()) { deps.login(returnUrl()); return }
    const owner = generation, key = contextKey(), revision = deps.revision(), lineup = snapshot.value
    const current = () => !disposed && owner === generation && key === contextKey() && revision === deps.revision()
    busy.value = true; error.value = ''; imagePath.value = ''
    try {
      const code = await getLineupCode(lineup.share_code)
      if (!current()) return
      const materialized = await materializePosterCode(code)
      try {
        const codePath = await imageInfo(materialized.path)
        if (!current()) return
        const path = await render(lineup, codePath, false, current)
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
  return { visible, busy, error, imagePath, payload, open, generate, preview, save, reset }
}
