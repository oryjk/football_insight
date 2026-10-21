import { CHENGDU_RONGCHENG_CREST_IMAGE_URL, SEASON_RECAP_POSTER_BACKGROUNDS } from '../../config/assets'
import { traceRoundRect } from '../../utils/canvasPoster'
import { POSTER_BACKGROUND_HEIGHT, POSTER_FOOTER_HEIGHT, POSTER_HEIGHT, POSTER_PALETTES, POSTER_WIDTH, type RecapPosterContent, type RecapPosterTheme } from './poster'
import type { RecapPosterImage } from './posterImages'

const MARGIN = 49
const CONTENT_WIDTH = POSTER_WIDTH - MARGIN * 2
const font = (bold: boolean, size: number) => `${bold ? 'bold ' : ''}${size}px sans-serif`

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

/** 装饰图失败仍能导出真实数据；队徽等比绘制，保留原有形状。 */
function drawImage(context: UniApp.CanvasContext, image: RecapPosterImage | undefined, x: number, y: number, width: number, height: number): boolean {
 if (!image) return false
 try { context.drawImage(image.path, x, y, width, height); return true } catch { return false }
}
function drawCrest(context: UniApp.CanvasContext, image: RecapPosterImage, x: number, y: number, size: number): void {
 const scale = size / Math.max(image.width, image.height)
 const width = image.width * scale, height = image.height * scale
 drawImage(context, image, x + (size - width) / 2, y + (size - height) / 2, width, height)
}

/** 城市间的「>」用折线绘制；字符「›」在部分真机字体里会渲染成「)」。 */
function drawChevron(context: UniApp.CanvasContext, x: number, y: number, stroke: string): void {
 context.setStrokeStyle(stroke); context.setLineWidth(3)
 context.setLineCap('round'); context.setLineJoin('round')
 context.beginPath(); context.moveTo(x - 4, y - 9); context.lineTo(x + 5, y); context.lineTo(x - 4, y + 9); context.stroke()
}

export function drawRecapPoster(
 context: UniApp.CanvasContext,
 content: RecapPosterContent,
 codePath: string,
 images: ReadonlyMap<string, RecapPosterImage> = new Map(),
 theme: RecapPosterTheme = 'dark',
): void {
 const color = POSTER_PALETTES[theme]
 const centerX = POSTER_WIDTH / 2
 const text = (value: string, x: number, y: number, size: number, fill: string | CanvasGradient, bold = false, maxWidth = CONTENT_WIDTH) => {
  const fitted = fitText(context, value, maxWidth, size, bold)
  context.font = font(bold, fitted.size)
  context.setFillStyle(fill)
  context.fillText(fitted.value, x, y)
 }
 // 背景保持位图原比例，只铺上部；底边渐隐到页脚底色，避免拼接硬边。
 const background = images.get(SEASON_RECAP_POSTER_BACKGROUNDS[theme])
 if (drawImage(context, background, 0, 0, POSTER_WIDTH, POSTER_BACKGROUND_HEIGHT)) {
  const fade = context.createLinearGradient(0, POSTER_BACKGROUND_HEIGHT - 70, 0, POSTER_BACKGROUND_HEIGHT)
  fade.addColorStop(0, color.footerClear); fade.addColorStop(1, color.footer)
  context.setFillStyle(fade)
  context.fillRect(0, POSTER_BACKGROUND_HEIGHT - 70, POSTER_WIDTH, 70)
  context.setFillStyle(color.footer)
  context.fillRect(0, POSTER_BACKGROUND_HEIGHT, POSTER_WIDTH, POSTER_FOOTER_HEIGHT)
 } else {
  const base = context.createLinearGradient(0, 0, 0, POSTER_HEIGHT)
  base.addColorStop(0, color.nightTop); base.addColorStop(0.55, color.nightMid); base.addColorStop(1, color.nightBottom)
  context.setFillStyle(base)
  context.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
  context.setTextAlign('center')
  text('我的赛季', centerX, 210, 78, color.gold, true)
 }
 context.setTextBaseline('normal')
 // 队徽与动态赛季沿同一行居中；主标题毛笔字包含在背景位图中。
 const heading = fitText(context, content.kicker, CONTENT_WIDTH - 48, 24, true)
 const headingWidth = context.measureText(heading.value).width
 const crest = images.get(CHENGDU_RONGCHENG_CREST_IMAGE_URL)
 const headingLeft = (POSTER_WIDTH - headingWidth - (crest ? 48 : 0)) / 2
 if (crest) drawCrest(context, crest, headingLeft, 41, 36)
 context.setTextAlign('left')
 text(heading.value, headingLeft + (crest ? 48 : 0), 67, heading.size, color.kicker, true)

 context.setTextAlign('center')
 text('我为蓉城走过', centerX, 540, 24, color.ink, true)
 const total = fitText(context, String(content.totalKm), 420, 146, true)
 const numberWidth = context.measureText(total.value).width
 const numberX = (POSTER_WIDTH - numberWidth - 14 - 56) / 2
 const numberGradient = context.createLinearGradient(0, 666 - total.size * 0.75, 0, 666)
 numberGradient.addColorStop(0, color.numberTop); numberGradient.addColorStop(0.5, color.numberMid); numberGradient.addColorStop(1, color.numberBottom)
 context.setTextAlign('left')
 text(total.value, numberX, 666, total.size, numberGradient, true, 420)
 text('公里', numberX + numberWidth + 14, 660, 28, color.ink, false, 56)
 context.setTextAlign('center')
 if (content.conversion) text(content.conversion, centerX, 701, 24, color.ink)
 const badge = fitText(context, content.badge, CONTENT_WIDTH - 72, 26, true)
 const badgeWidth = context.measureText(badge.value).width + 72
 context.setStrokeStyle(color.badgeBorder); context.setLineWidth(2)
 context.setFillStyle(color.badgeFill)
 traceRoundRect(context, centerX - badgeWidth / 2, 717, badgeWidth, 48, 24)
 context.fill(); context.stroke()
 text(badge.value, centerX, 750, badge.size, color.badgeText, true)

 // 同宽的单行卡片，标签与数值共享基线；不向用户展示评分算法或“已赴”说明。
 const cards = [
  { label: '客场里程', value: `${content.awayKm} 公里`, progress: content.progress.away },
  { label: '主场里程', value: content.homeKm === null ? '—' : `${content.homeKm} 公里`, progress: content.progress.home },
  { label: '足迹', value: `${content.matches} 场 · ${content.cities} 城`, progress: content.progress.total },
 ]
 cards.forEach((card, index) => {
  const y = 803 + index * 114
  context.setFillStyle(color.cardFill); context.setStrokeStyle(color.cardBorder); context.setLineWidth(2)
  traceRoundRect(context, MARGIN, y, CONTENT_WIDTH, 102, 14)
  context.fill(); context.stroke()
  context.setTextAlign('left')
  text(card.label, MARGIN + 24, y + 40, 27, color.ink, true, 180)
  context.setTextAlign('right')
  text(card.value, POSTER_WIDTH - MARGIN - 24, y + 40, 30, color.ink, true, CONTENT_WIDTH - 228)
  if (card.progress === null) return // 旧后端缺整季赛程，不伪造“零进度”。
  const x = MARGIN + 24, width = CONTENT_WIDTH - 48, barY = y + 56, height = 16
  context.setFillStyle(color.track)
  traceRoundRect(context, x, barY, width, height, height / 2); context.fill()
  const progress = Math.max(0, Math.min(1, card.progress))
  if (progress <= 0) return
  // 渐变只跨已填充部分：短条也从暖黄过渡到正红。不用 clip——微信 canvas 里 clip 后渐变填充会丢失。
  const filled = Math.max(height, width * progress)
  const gradient = context.createLinearGradient(x, 0, x + filled, 0)
  gradient.addColorStop(0, color.progressStart); gradient.addColorStop(0.5, color.progressMid); gradient.addColorStop(1, color.progressEnd)
  context.setFillStyle(gradient)
  traceRoundRect(context, x, barY, filled, height, height / 2)
  context.fill()
 })

 const route = content.routeEntries
 if (route) {
  const title = `成都出发${route.totalCities > route.entries.length ? ` · 等 ${route.totalCities} 城` : ''}`
  context.setTextAlign('center')
  text(title, centerX, 1157, 19, color.goldDim)
  const titleHalf = context.measureText(title).width / 2 + 16
  context.setStrokeStyle(color.routeRule); context.setLineWidth(1.5)
  context.beginPath()
  context.moveTo(MARGIN, 1150); context.lineTo(centerX - titleHalf, 1150)
  context.moveTo(centerX + titleHalf, 1150); context.lineTo(POSTER_WIDTH - MARGIN, 1150)
  context.stroke()
  const cellWidth = Math.min(152, CONTENT_WIDTH / route.entries.length)
  const left = (POSTER_WIDTH - cellWidth * route.entries.length) / 2
  route.entries.forEach((entry, index) => {
   const x = left + cellWidth * (index + 0.5)
   const crests = entry.avatarUrls.map(url => images.get(url)).filter((image): image is RecapPosterImage => Boolean(image))
   const size = Math.min(64, (cellWidth - 16) / Math.max(1, crests.length) - 4)
   const groupWidth = crests.length * (size + 4) - 4
   crests.forEach((image, i) => drawCrest(context, image, x - groupWidth / 2 + i * (size + 4), 1170, size))
   context.setTextAlign('center')
   text(entry.city, x, 1285, 24, color.ink, true, cellWidth - 16)
   if (index < route.entries.length - 1) drawChevron(context, left + cellWidth * (index + 1), 1202, color.chevron)
  })
 }

 // 保留可扫码回流的分享页脚；它不参与称号或进度计算。
 context.setStrokeStyle(color.rule); context.setLineWidth(1)
 context.beginPath(); context.moveTo(MARGIN, 1345); context.lineTo(POSTER_WIDTH - MARGIN, 1345); context.stroke()
 context.setTextAlign('left')
 text('足球洞察', MARGIN, 1395, 28, color.ink, true, 390)
 text('这一年，与你同场', MARGIN, 1430, 22, color.muted, false, 390)
 text('自报观赛 · 客场为城市间直线往返估算', MARGIN, 1470, 15, color.faint, false, 490)
 context.setFillStyle(color.codeCard)
 traceRoundRect(context, 585, 1361, 116, 116, 12); context.fill()
 context.drawImage(codePath, 593, 1369, 100, 100)
}
