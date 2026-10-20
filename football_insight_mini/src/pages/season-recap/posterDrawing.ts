import { traceRoundRect } from '../../utils/canvasPoster'
import { POSTER_HEIGHT, POSTER_PALETTE as color, POSTER_WIDTH, type RecapPosterContent } from './poster'

const MARGIN = 60
const CONTENT_WIDTH = POSTER_WIDTH - MARGIN * 2

function font(bold: boolean, size: number): string {
 return `${bold ? 'bold ' : ''}${size}px sans-serif`
}

/** 文字超过 maxWidth 时逐级缩小字号（需先设置好 bold），返回实际字号。 */
function fitText(context: UniApp.CanvasContext, value: string, maxWidth: number, size: number, bold: boolean): number {
 let current = size
 context.font = font(bold, current)
 while (current > 16 && context.measureText(value).width > maxWidth) {
  current -= 4
  context.font = font(bold, current)
 }
 return current
}

export function drawRecapPoster(context: UniApp.CanvasContext, content: RecapPosterContent, codePath: string): void {
 const centerX = POSTER_WIDTH / 2
 const text = (value: string, x: number, y: number, size: number, fill: string | CanvasGradient, bold = false) => {
  context.font = font(bold, size)
  context.setFillStyle(fill)
  context.fillText(value, x, y)
 }
 // —— 背景：深夜色纵向渐变 ——
 const night = context.createLinearGradient(0, 0, 0, POSTER_HEIGHT)
 night.addColorStop(0, color.nightTop)
 night.addColorStop(0.55, color.nightMid)
 night.addColorStop(1, color.nightBottom)
 context.setFillStyle(night)
 context.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
 // —— 顶部两侧斜向灯光束 ——
 for (const flip of [false, true]) {
  const beam = context.createLinearGradient(flip ? 690 : 60, 0, flip ? 400 : 350, 620)
  beam.addColorStop(0, color.beam)
  beam.addColorStop(1, 'rgba(255, 255, 255, 0)')
  context.setFillStyle(beam)
  context.beginPath()
  if (flip) {
   context.moveTo(750, 0); context.lineTo(470, 0); context.lineTo(660, 640); context.lineTo(790, 640)
  } else {
   context.moveTo(0, 0); context.lineTo(280, 0); context.lineTo(90, 640); context.lineTo(-40, 640)
  }
  context.closePath()
  context.fill()
 }
 // —— 球场线暗纹：中圈弧线 + 中线 ——
 context.setStrokeStyle(color.pitchLine)
 context.setLineWidth(2)
 context.beginPath()
 context.arc(centerX, POSTER_HEIGHT + 260, 520, 0, 2 * Math.PI)
 context.stroke()
 context.beginPath()
 context.moveTo(0, POSTER_HEIGHT - 190)
 context.lineTo(POSTER_WIDTH, POSTER_HEIGHT - 190)
 context.stroke()
 // —— 大数字背后的红色径向光晕 ——
 const glow = context.createCircularGradient(centerX, 470, 340)
 glow.addColorStop(0, color.glow)
 glow.addColorStop(1, 'rgba(220, 38, 38, 0)')
 context.setFillStyle(glow)
 context.fillRect(centerX - 340, 130, 680, 680)
 // —— 顶部：小字（字距拉开）+ 标题 ——
 context.setTextAlign('left')
 text([...content.kicker].join(' '), MARGIN, 110, 24, color.goldDim)
 text('我的远征足迹', MARGIN, 200, 64, color.ink, true)
 context.setFillStyle(color.gold)
 traceRoundRect(context, MARGIN, 232, 56, 6, 3)
 context.fill()
 // —— 主视觉：超大总里程 + 公里 ——
 const total = String(content.totalKm)
 const numberSize = fitText(context, total, 480, 210, true)
 context.font = font(true, numberSize)
 const numberWidth = context.measureText(total).width
 context.font = font(false, 34)
 const unitWidth = context.measureText('公里').width
 const numberX = Math.max(MARGIN, (POSTER_WIDTH - numberWidth - 20 - unitWidth) / 2)
 const numberGradient = context.createLinearGradient(0, 560 - numberSize, 0, 560)
 numberGradient.addColorStop(0, color.ink)
 numberGradient.addColorStop(1, color.gold)
 context.setTextAlign('left')
 text(total, numberX, 560, numberSize, numberGradient, true)
 text('公里', numberX + numberWidth + 20, 560, 34, color.muted)
 // —— 趣味换算 + 称号徽章（居中） ——
 context.setTextAlign('center')
 if (content.conversion) text(content.conversion, centerX, 650, 26, color.gold)
 context.font = font(true, 26)
 const badgeWidth = context.measureText(content.badge).width + 68
 const badgeY = 700
 context.setStrokeStyle(color.gold)
 context.setLineWidth(2)
 traceRoundRect(context, centerX - badgeWidth / 2, badgeY, badgeWidth, 62, 31)
 context.stroke()
 text(content.badge, centerX, badgeY + 41, 26, color.gold, true)
 // —— 数据卡：三块半透明玻璃卡 ——
 const cards: Array<{ label: string; value: string }> = [
  { label: '客场里程', value: `${content.awayKm} 公里` },
  { label: '主场里程', value: content.homeKm === null ? '—' : `${content.homeKm} 公里` },
  { label: '足迹', value: `${content.matches} 场 · ${content.cities} 城` },
 ]
 const cardY = 806, cardHeight = 150, cardWidth = (CONTENT_WIDTH - 40) / 3
 cards.forEach((card, index) => {
  const x = MARGIN + index * (cardWidth + 20)
  context.setFillStyle(color.glass)
  context.setStrokeStyle(color.glassBorder)
  context.setLineWidth(1)
  traceRoundRect(context, x, cardY, cardWidth, cardHeight, 24)
  context.fill()
  context.stroke()
  const cardCenter = x + cardWidth / 2
  context.setTextAlign('center')
  text(card.label, cardCenter, cardY + 48, 22, color.muted)
  const valueSize = fitText(context, card.value, cardWidth - 28, 36, true)
  text(card.value, cardCenter, cardY + 110, valueSize, color.ink, true)
 })
 // —— 足迹路线 ——
 if (content.route) {
  context.setTextAlign('center')
  const routeSize = fitText(context, content.route, CONTENT_WIDTH, 24, false)
  text(content.route, centerX, 1030, routeSize, color.muted)
 }
 // —— 底部：品牌文案 + 白底小程序码 ——
 context.setTextAlign('left')
 text('足球洞察', MARGIN, 1160, 32, color.ink, true)
 text('这一年，与你同场', MARGIN, 1208, 23, color.muted)
 context.setFillStyle(color.codeCard)
 traceRoundRect(context, 536, 1086, 164, 164, 20)
 context.fill()
 context.drawImage(codePath, 546, 1096, 144, 144)
 context.setTextAlign('center')
 text('扫码生成你的赛季足迹', 618, 1292, 19, color.muted)
 context.setTextAlign('left')
 text('自报观赛 · 客场为城市间直线往返估算', MARGIN, 1312, 18, color.faint)
 context.setTextAlign('left')
}
