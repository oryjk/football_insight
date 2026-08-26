<template>
  <view class="standings-sheet-mask" @click.self="emit('close')" @touchmove.stop.prevent>
    <view class="standings-sheet" @touchmove.stop>
      <view class="section-heading">
        <view>
          <text class="section-kicker">完整积分榜</text>
          <text class="section-title">{{ table.label }}</text>
        </view>
        <view class="standings-sheet__actions">
          <button v-if="posterImagePath" class="standings-sheet__share" open-type="share">分享</button>
          <button v-if="posterImagePath" class="standings-sheet__save" @click="savePosterImage">保存图片</button>
          <button class="standings-sheet__close" @click="emit('close')">关闭</button>
        </view>
      </view>

      <FiLoading
        v-if="generating"
        title="榜单图片生成中"
        caption="正在把当前积分榜整理成一张可保存的图片。"
      />

      <view v-else-if="posterImagePath" class="standings-sheet__poster">
        <image :src="posterImagePath" mode="widthFix" class="standings-sheet__poster-image" />
        <text class="standings-sheet__hint standings-sheet__hint--resolved">{{ posterHintText }}</text>
      </view>

      <view v-if="errorMessage" class="state-card state-card--error standings-sheet__error">
        <text>{{ errorMessage }}</text>
      </view>

      <view v-if="!posterImagePath" class="standings-sheet__list">
        <view v-for="entry in table.entries" :key="`${table.slug}-${entry.team_id}`" class="standings-sheet__row">
          <text class="standings-sheet__rank">{{ entry.rank_no }}</text>
          <text class="standings-sheet__name">{{ entry.team_name }}</text>
          <view class="standings-sheet__metrics">
            <view
              v-for="metric in buildStandingsFallbackMetrics(table, entry)"
              :key="metric.label"
              class="standings-sheet__metric"
            >
              <text class="standings-sheet__metric-value">{{ metric.value }}</text>
              <text class="standings-sheet__metric-label">{{ metric.label }}</text>
            </view>
          </view>
        </view>
      </view>
    </view>

    <canvas canvas-id="standingsPosterCanvas" class="standings-poster-canvas" />
  </view>
</template>

<script setup lang="ts">
import { getCurrentInstance, ref, watch } from 'vue'
import FiLoading from '../../../components/FiLoading.vue'
import type { StandingsTable, StandingsTableEntry } from '../../../types/insight'
import { extractApiErrorMessage } from '../../../utils/apiError'
import {
  buildStandingsFallbackMetrics,
  buildStandingsPosterColumns,
  buildStandingsPosterMetrics,
  buildStandingsPosterTeamLayout,
} from '../poster'
import { buildStandingsPosterSubtitle, truncateStandingsPosterTeamName } from '../helpers'

const props = defineProps<{
  table: StandingsTable
  season: number
  round: number | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'poster-ready', path: string): void
}>()

const posterHintText = '图片右下角已附公众号二维码水印，可直接保存或转发。'
const posterTitleText = '中超积分榜'
const posterBrandText = '足球洞察 · 何止编程'
const posterSourceText = '数据来自当前抓取时点的赛季累计值'
const posterQrCaption = '微信扫码查看'
const posterQrModules = [
  '111111101101101011000101101111111',
  '100000100000000011010110101000001',
  '101110100111010000011110101011101',
  '101110101111101101111010001011101',
  '101110101100101000111111001011101',
  '100000101100011011110110101000001',
  '111111101010101010101010101111111',
  '000000001101001111000111000000000',
  '100010111101110011111111111111001',
  '000110010011011110101011000101111',
  '110011110010000101001111100001010',
  '100000000101101011101111001000001',
  '111110110011111100101001001111010',
  '111000001110010011010001010101100',
  '110010100010110010110111110000010',
  '100010001111000001111101011110000',
  '001101100000011101001111101010110',
  '111101011011010111100111010001110',
  '111100100110101100101001010001010',
  '000101010000000100011011010000011',
  '110010110101011010000011001111010',
  '110100010011101001111001100101110',
  '001110111111010101101111010100010',
  '000111000011100111100100010011001',
  '111000101110110011011101111110000',
  '000000001101001100000001100010111',
  '111111101010101101001110101011010',
  '100000100101010001110100100011010',
  '101110101101111100111000111111001',
  '101110100110101010010111101011010',
  '101110100001001011010001101110000',
  '100000100101110001011110101100000',
  '111111101010110101101111111011101',
]

// 模块级缓存：弹层多次开关、切换榜单表时复用已生成的海报与队徽，避免重复绘制。
const posterCache = new Map<string, string>()
const posterLogoCache = new Map<string, string | null>()

const instance = getCurrentInstance()
const posterImagePath = ref('')
const generating = ref(false)
const errorMessage = ref('')

watch(
  () => props.table.slug,
  () => {
    void generate()
  },
  { immediate: true },
)

async function generate(): Promise<void> {
  const cached = posterCache.get(props.table.slug)
  if (cached) {
    posterImagePath.value = cached
    emit('poster-ready', cached)
    return
  }

  const canvasId = 'standingsPosterCanvas'
  const width = 1080
  const rowHeight = 60
  const headerHeight = 248
  const footerHeight = 196
  const qrSize = 128
  const maxRows = Math.min(props.table.entries.length, 16)
  const height = headerHeight + footerHeight + rowHeight * maxRows
  errorMessage.value = ''
  generating.value = true

  try {
    const context = uni.createCanvasContext(canvasId, instance)
    const posterEntries = props.table.entries.slice(0, maxRows)
    const posterLogoPaths = await resolvePosterLogoPaths(posterEntries)

    context.setFillStyle('#f3f3f6')
    context.fillRect(0, 0, width, height)

    context.setFillStyle('#ffffff')
    roundRect(context, 36, 36, width - 72, height - 72, 34)
    context.fill()

    context.setFillStyle('#121212')
    context.setFontSize(48)
    context.fillText(`${props.season} ${posterTitleText}`, 88, 120)

    context.setFillStyle('#8f9198')
    context.setFontSize(28)
    context.fillText(
      buildStandingsPosterSubtitle({ season: props.season, round: props.round, table: props.table }),
      88,
      166,
    )
    context.fillText(props.table.note, 88, 204)

    context.setStrokeStyle('#ececf1')
    context.setLineWidth(2)
    context.beginPath()
    context.moveTo(88, 228)
    context.lineTo(width - 88, 228)
    context.stroke()

    context.setFillStyle('#8f9198')
    context.setFontSize(24)
    buildStandingsPosterColumns(props.table).forEach((column) => {
      context.fillText(column.label, column.x, 270)
    })

    context.setFillStyle('#121212')
    context.setFontSize(28)

    posterEntries.forEach((entry, index) => {
      const y = 324 + index * rowHeight
      const teamLayout = buildStandingsPosterTeamLayout(Boolean(posterLogoPaths.get(entry.team_id)))
      context.setFillStyle(index < 3 ? '#dc2626' : '#121212')
      context.fillText(String(entry.rank_no), 88, y)
      drawPosterTeamLogo(context, posterLogoPaths.get(entry.team_id) ?? null, teamLayout.logoX, y - 22, teamLayout.logoSize)
      context.setFillStyle('#121212')
      context.fillText(truncateStandingsPosterTeamName(entry.team_name), teamLayout.nameX, y)
      buildStandingsPosterMetrics(props.table, entry).forEach((metric) => {
        if (metric.highlight) {
          context.setFillStyle(String(entry.points_adjustment > 0 ? '#16a34a' : '#dc2626'))
        } else {
          context.setFillStyle('#121212')
        }

        context.setFontSize(metric.compact ? 20 : 28)
        context.fillText(metric.value, metric.x, y)
      })
      context.setFillStyle('#121212')
      context.setFontSize(28)

      context.setStrokeStyle('#f0f1f5')
      context.setLineWidth(1)
      context.beginPath()
      context.moveTo(88, y + 24)
      context.lineTo(width - 88, y + 24)
      context.stroke()
    })

    context.setFillStyle('#8f9198')
    context.setFontSize(24)
    context.fillText(posterBrandText, 88, height - 70)
    context.fillText(posterSourceText, 88, height - 36)

    drawPosterQrCode(context, width - qrSize - 88, height - footerHeight + 24, qrSize)
    context.setFillStyle('#8f9198')
    context.setFontSize(20)
    context.fillText(posterQrCaption, width - qrSize - 88, height - footerHeight + 176)

    await new Promise<void>((resolve) => {
      context.draw(false, () => resolve())
    })

    const tempFilePath = await new Promise<string>((resolve, reject) => {
      uni.canvasToTempFilePath({
        canvasId,
        width,
        height,
        destWidth: width,
        destHeight: height,
        success: (result) => resolve(result.tempFilePath),
        fail: (error) => reject(error),
      }, instance)
    })

    posterCache.set(props.table.slug, tempFilePath)
    posterImagePath.value = tempFilePath
    emit('poster-ready', tempFilePath)
  } catch (error) {
    errorMessage.value = extractApiErrorMessage(error, '积分榜图片生成失败，请稍后重试。')
  } finally {
    generating.value = false
  }
}

async function resolvePosterLogoPaths(entries: StandingsTableEntry[]): Promise<Map<number, string | null>> {
  const resolved = await Promise.all(entries.map(async (entry) => {
    return [entry.team_id, await resolvePosterLogoPath(entry.avatar_storage_url)] as const
  }))

  return new Map(resolved)
}

async function resolvePosterLogoPath(src: string | null): Promise<string | null> {
  if (!src) {
    return null
  }

  if (posterLogoCache.has(src)) {
    return posterLogoCache.get(src) ?? null
  }

  try {
    const result = await new Promise<UniApp.GetImageInfoSuccessData>((resolve, reject) => {
      uni.getImageInfo({
        src,
        success: resolve,
        fail: reject,
      })
    })

    posterLogoCache.set(src, result.path)
    return result.path
  } catch {
    posterLogoCache.set(src, null)
    return null
  }
}

function roundRect(
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

function drawPosterQrCode(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  size: number,
): void {
  const quietZone = 4
  const moduleCount = posterQrModules.length
  const totalCount = moduleCount + quietZone * 2
  const cellSize = size / totalCount

  context.setFillStyle('#ffffff')
  context.fillRect(x, y, size, size)
  context.setFillStyle('#121212')

  posterQrModules.forEach((row, rowIndex) => {
    for (let columnIndex = 0; columnIndex < row.length; columnIndex += 1) {
      if (row[columnIndex] !== '1') {
        continue
      }

      context.fillRect(
        x + (columnIndex + quietZone) * cellSize,
        y + (rowIndex + quietZone) * cellSize,
        cellSize,
        cellSize,
      )
    }
  })
}

function drawPosterTeamLogo(
  context: UniApp.CanvasContext,
  logoPath: string | null,
  x: number,
  y: number,
  size: number,
): void {
  if (!logoPath || size <= 0) {
    return
  }

  const radius = size / 2
  context.save()
  context.beginPath()
  context.arc(x + radius, y + radius, radius, 0, Math.PI * 2, false)
  context.clip()
  context.drawImage(logoPath, x, y, size, size)
  context.restore()
}

function savePosterImage(): void {
  if (!posterImagePath.value) {
    return
  }

  uni.saveImageToPhotosAlbum({
    filePath: posterImagePath.value,
    success: () => {
      uni.showToast({ title: '已保存到相册', icon: 'success' })
    },
    fail: (error) => {
      uni.showToast({
        title: extractApiErrorMessage(error, '保存失败'),
        icon: 'none',
      })
    },
  })
}
</script>

<style scoped lang="css">
.state-card--error text {
  font-size: var(--fi-font-28);
  color: #c03a2b;
}

.standings-sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: var(--fi-space-28);
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}

.standings-sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
  padding: var(--fi-space-28);
  border-radius: 32rpx;
  background: var(--fi-primitive-white);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.standings-sheet__actions {
  display: flex;
  align-items: center;
  gap: var(--fi-space-12);
}

.standings-sheet__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-component-close-bg);
  font-size: var(--fi-component-close-size);
  color: var(--fi-component-close-text);
}

.standings-sheet__share {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: rgba(21, 22, 27, 0.92);
  font-size: var(--fi-font-24);
  color: var(--fi-primitive-white);
}

.standings-sheet__save {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: rgba(var(--fi-primitive-red-rgb), 0.12);
  font-size: var(--fi-font-24);
  color: var(--fi-color-primary);
}

.standings-sheet__poster {
  margin-top: var(--fi-space-18);
}

.standings-sheet__poster-image {
  width: 100%;
  border-radius: var(--fi-radius-md);
}

.standings-sheet__hint {
  display: block;
  margin-top: var(--fi-space-14);
  color: transparent;
  font-size: 0;
  line-height: 0;
}

.standings-sheet__hint--resolved {
  color: #767a84;
  font-size: var(--fi-font-24);
  line-height: 1.6;
}

.standings-sheet__error {
  margin-top: var(--fi-space-18);
}

.standings-sheet__list {
  margin-top: var(--fi-space-18);
  display: grid;
  gap: var(--fi-space-12);
}

.standings-sheet__row {
  display: grid;
  grid-template-columns: 52rpx minmax(0, 1fr) auto;
  gap: var(--fi-space-16);
  align-items: center;
  padding: var(--fi-space-18) 0;
  border-bottom: var(--fi-primitive-border-width) solid #eff1f5;
}

.standings-sheet__rank {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-28);
}

.standings-sheet__name {
  color: var(--fi-color-text-strong);
  font-weight: var(--fi-weight-bold);
  font-size: var(--fi-font-28);
}

.standings-sheet__metrics {
  display: grid;
  grid-template-columns: repeat(3, 64rpx);
  gap: var(--fi-space-10);
  justify-content: end;
}

.standings-sheet__metric {
  display: grid;
  justify-items: end;
  gap: 4rpx;
  min-width: 0;
}

.standings-sheet__metric-value {
  color: var(--fi-color-text-strong);
  font-weight: var(--fi-weight-extrabold);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-none);
}

.standings-sheet__metric:first-child .standings-sheet__metric-value {
  color: var(--fi-color-primary);
}

.standings-sheet__metric-label {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.standings-poster-canvas {
  position: fixed;
  left: -9999rpx;
  top: -9999rpx;
  width: 1080px;
  height: 1600px;
  pointer-events: none;
}
</style>
