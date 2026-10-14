import { ref, type Ref } from 'vue'
import type { AppSession } from '../../stores/appSession'
import type { SupportMatchDetail, SupportMatchTeam } from '../../types/support'
import { extractApiErrorMessage } from '../../utils/apiError'
import { exportCanvasToTempFile, flushCanvas } from '../../utils/canvasPoster'
import { createLatestGuard } from '../../utils/latestGuard'
import { SUPPORT_POSTER_CANVAS_ID, SUPPORT_POSTER_SIZE, drawSupportPoster } from './poster'

/** 拉票海报生成：数据替换/换号/换比赛后，迟到的绘制结果与错误不再发布。 */
export function useSupportPoster(
  session: Pick<AppSession, 'revision'>,
  matchId: Ref<number | null>,
) {
  const posterImagePath = ref('')
  const posterGenerating = ref(false)
  const guard = createLatestGuard(() => [session.revision.value, matchId.value])

  function invalidatePoster(): void {
    guard.invalidate()
    posterImagePath.value = ''
  }

  async function generatePoster(detail: SupportMatchDetail | null, supportedTeam: SupportMatchTeam | null): Promise<void> {
    if (!detail || posterGenerating.value) {
      return
    }

    const token = guard.begin()
    posterGenerating.value = true
    try {
      const context = uni.createCanvasContext(SUPPORT_POSTER_CANVAS_ID)
      drawSupportPoster(context, detail, supportedTeam)
      await flushCanvas(context)
      if (!token.isCurrent()) return

      const imagePath = await exportCanvasToTempFile({ canvasId: SUPPORT_POSTER_CANVAS_ID, ...SUPPORT_POSTER_SIZE })
      if (!token.isCurrent()) return
      posterImagePath.value = imagePath
      uni.showToast({ title: '海报已生成', icon: 'success' })
    } catch (error) {
      if (!token.isCurrent()) return
      uni.showToast({ title: extractApiErrorMessage(error, '海报生成失败'), icon: 'none' })
    } finally {
      posterGenerating.value = false
    }
  }

  return { posterImagePath, posterGenerating, invalidatePoster, generatePoster }
}
