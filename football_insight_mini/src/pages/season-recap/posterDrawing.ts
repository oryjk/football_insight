import { CHENGDU_RONGCHENG_CREST_IMAGE_URL, SEASON_RECAP_POSTER_BG_IMAGE_URL } from '../../config/assets'
import { traceRoundRect } from '../../utils/canvasPoster'
import { POSTER_HEIGHT, POSTER_PALETTE as color, POSTER_WIDTH, type RecapPosterContent } from './poster'
import type { RecapPosterImage } from './posterImages'

const MARGIN = 60
const CONTENT_WIDTH = POSTER_WIDTH - MARGIN * 2

function font(bold: boolean, size: number): string {
 return `${bold ? 'bold ' : ''}${size}px sans-serif`
}

/** 旧 canvas 用 measureText 缩字，极长文案到最小字号后截断。 */
function fitText(context: UniApp.CanvasContext, value: string, maxWidth: number, size: number, bold: boolean) {
 let current = size
 context.font = font(bold, current)
 while (current > 14 && context.measureText(value).width > maxWidth) {
  current = Math.max(14, current - 2)
  context.font = font(bold, current)
 }
 let fitted = value
 while (fitted.length && context.measureText(fitted).width > maxWidth) {
  fitted = `${fitted.replace(/…$/, '').slice(0, -1)}…`
  if (fitted === '…') break
 }
 return { value: fitted, size: current }
}

/** 装饰图只消费 getImageInfo 的本地路径，绘制异常也不阻断导出。 */
function drawImage(context: UniApp.CanvasContext, image: RecapPosterImage | undefined, x: number, y: number, width: number, height: number): boolean {
 if (!image) return false
 try { context.drawImage(image.path, x, y, width, height); return true } catch { return false }
}

function drawCrest(context: UniApp.CanvasContext, image: RecapPosterImage, x: number, y: number, size: number): void {
 context.save()
 try {
  context.beginPath()
  context.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  context.clip()
  const scale = (size - 6) / Math.max(image.width, image.height)
  const width = image.width * scale, height = image.height * scale
  drawImage(context, image, x + (size - width) / 2, y + (size - height) / 2, width, height)
 } finally { context.restore() }
}

export function drawRecapPoster(
 context: UniApp.CanvasContext,
 content: RecapPosterContent,
 codePath: string,
 images: ReadonlyMap<string, RecapPosterImage> = new Map(),
): void {
 const centerX = POSTER_WIDTH / 2
 const text = (value: string, x: number, y: number, size: number, fill: string | CanvasGradient, bold = false, maxWidth = CONTENT_WIDTH) => {
  const fitted = fitText(context, value, maxWidth, size, bold)
  context.font = font(bold, fitted.size)
  context.setFillStyle(fill)
  context.fillText(fitted.value, x, y)
 }
 // 成品背景已融合真实队徽与暗色蒙层；无图时使用同构暗红渐变及真实队徽。
 if (!drawImage(context, images.get(SEASON_RECAP_POSTER_BG_IMAGE_URL), 0, 0, POSTER_WIDTH, POSTER_HEIGHT)) {
  const night = context.createLinearGradient(0, 0, 0, POSTER_HEIGHT)
  night.addColorStop(0, color.nightTop)
  night.addColorStop(0.55, color.nightMid)
  night.addColorStop(1, color.nightBottom)
  context.setFillStyle(night)
  context.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
  const crest = images.get(CHENGDU_RONGCHENG_CREST_IMAGE_URL)
  if (crest) {
   const scale = 206 / Math.max(crest.width, crest.height)
   context.save()
   context.setGlobalAlpha(0.35)
   drawImage(context, crest, centerX - crest.width * scale / 2, 166, crest.width * scale, crest.height * scale)
   context.restore()
  }
 }
 // —— 片头与中央真实队徽留白 ——
 context.setTextAlign('center')
 text(content.kicker, centerX, 70, 24, color.goldDim)
 text('我的远征足迹', centerX, 130, 46, color.ink, true)
 context.setStrokeStyle(color.rule)
 context.setLineWidth(1)
 context.beginPath()
 context.moveTo(MARGIN, 394); context.lineTo(POSTER_WIDTH - MARGIN, 394)
 context.stroke()
 text('我为蓉城走过', centerX, 434, 22, color.muted)
 // —— 主视觉：总里程大数字 ——
 const total = fitText(context, String(content.totalKm), 510, 176, true)
 const numberWidth = context.measureText(total.value).width
 const numberX = (POSTER_WIDTH - numberWidth - 18 - 68) / 2
 const numberGradient = context.createLinearGradient(0, 594 - total.size, 0, 594)
 numberGradient.addColorStop(0, color.ink)
 numberGradient.addColorStop(1, color.gold)
 context.setTextAlign('left')
 text(total.value, numberX, 594, total.size, numberGradient, true, 510)
 text('公里', numberX + numberWidth + 18, 592, 30, color.muted, false, 68)
 context.setTextAlign('center')
 if (content.conversion) text(content.conversion, centerX, 650, 25, color.gold)
 const badge = fitText(context, content.badge, CONTENT_WIDTH - 68, 27, true)
 const badgeWidth = context.measureText(badge.value).width + 68
 context.setStrokeStyle(color.goldDim)
 context.setFillStyle(color.glass)
 context.setLineWidth(1)
 traceRoundRect(context, centerX - badgeWidth / 2, 698, badgeWidth, 60, 30)
 context.fill(); context.stroke()
 text(badge.value, centerX, 738, badge.size, color.gold, true)
 // —— 三块数据卡 ——
 const cards = [
  { label: '客场里程', value: `${content.awayKm} 公里` },
  { label: '主场里程', value: content.homeKm === null ? '—' : `${content.homeKm} 公里` },
  { label: '足迹', value: `${content.matches} 场 · ${content.cities} 城` },
 ]
 const cardY = 804, cardHeight = 138, cardWidth = (CONTENT_WIDTH - 24) / 3
 cards.forEach((card, index) => {
  const x = MARGIN + index * (cardWidth + 12)
  context.setFillStyle(color.glass)
  context.setStrokeStyle(color.glassBorder)
  traceRoundRect(context, x, cardY, cardWidth, cardHeight, 18)
  context.fill(); context.stroke()
  text(card.label, x + cardWidth / 2, cardY + 43, 22, color.muted)
  text(card.value, x + cardWidth / 2, cardY + 99, 32, color.ink, true, cardWidth - 24)
 })
 // —— 按日期去重的客场足迹：至多六列，同城不同球队的圆徽并排 ——
 const route = content.routeEntries
 if (route) {
  context.setTextAlign('left')
  text(`成都出发${route.totalCities > route.entries.length ? ` · 等 ${route.totalCities} 城` : ''}`, MARGIN, 982, 19, color.goldDim)
  const cellWidth = CONTENT_WIDTH / route.entries.length
  route.entries.forEach((entry, index) => {
   const x = MARGIN + cellWidth * (index + 0.5)
   const crests = entry.avatarUrls.map(url => images.get(url)).filter((image): image is RecapPosterImage => Boolean(image))
   const size = Math.min(32, (cellWidth - 12) / Math.max(1, crests.length) - 4)
   const groupWidth = crests.length * (size + 4) - 4
   crests.forEach((crest, crestIndex) => drawCrest(context, crest, x - groupWidth / 2 + crestIndex * (size + 4), 997, size))
   context.setTextAlign('center')
   text(entry.city, x, 1058, 22, color.muted, false, cellWidth - 16)
   if (index < route.entries.length - 1) text('›', MARGIN + cellWidth * (index + 1), 1058, 20, color.goldDim, false, 14)
  })
 }
 // —— 品牌与小程序码 ——
 context.setStrokeStyle(color.rule)
 context.beginPath()
 context.moveTo(MARGIN, 1090); context.lineTo(POSTER_WIDTH - MARGIN, 1090)
 context.stroke()
 context.setTextAlign('left')
 text('足球洞察', MARGIN, 1160, 32, color.ink, true, 420)
 text('这一年，与你同场', MARGIN, 1208, 23, color.muted, false, 420)
 context.setFillStyle(color.codeCard)
 traceRoundRect(context, 526, 1114, 164, 164, 18)
 context.fill()
 context.drawImage(codePath, 536, 1124, 144, 144)
 context.setTextAlign('center')
 text('扫码生成你的赛季足迹', 608, 1300, 17, color.muted, false, 180)
 context.setTextAlign('left')
 text('自报观赛 · 客场为城市间直线往返估算', MARGIN, 1312, 18, color.faint, false, 450)
}
