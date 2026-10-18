import { traceRoundRect } from '../../utils/canvasPoster'
import { wrapShareCardText } from '../../utils/shareCardText'
import { TICKET_POSTER, type TicketPosterContent } from './sharePoster'

// Decorative bitmap palette: exported artwork stays readable in either app theme.
const color = { paper: '#ffffff', ink: '#15161b', muted: '#6d7280', accent: '#dc2626', background: '#f6f7fb' }

export function drawTicketPoster(context: UniApp.CanvasContext, content: TicketPosterContent, codePath: string): void {
  const text = (value: string, x: number, y: number, size: number, fill = color.ink) => {
    context.setFillStyle(fill); context.setFontSize(size); context.fillText(value, x, y)
  }
  const wrapped = (value: string, x: number, y: number, size: number, width: number, lines: number, fill = color.ink) => {
    context.setFontSize(size)
    wrapShareCardText(value, value => context.measureText(value).width, width, lines)
      .forEach((line, index) => text(line, x, y + index * (size + 12), size, fill))
  }
  context.setFillStyle(color.background); context.fillRect(0, 0, TICKET_POSTER.width, TICKET_POSTER.height)
  context.setFillStyle(color.paper); traceRoundRect(context, 28, 28, 694, 1024, 28); context.fill()
  text('足球洞察 · 回流看板', 60, 92, 28, color.accent)
  text(content.mode === 'current' ? '当前比赛回流' : '历史回流复盘', 60, 144, 24, color.muted)
  wrapped(content.teams, 60, 206, 38, 630, 2)
  wrapped(content.matchLabel, 60, 308, 23, 630, 1, color.muted)
  text(String(content.totalOccurrences), 60, 414, 84, color.accent)
  text(`张累计回流 · ${content.regionCount} 个区域`, 60, 462, 26)
  text('回流较多的区域', 60, 520, 24, color.muted)
  content.regions.forEach((region, index) => {
    const y = 568 + index * 43
    wrapped(`${region.name} · ¥${region.price}`, 60, y, 25, 470, 1)
    text(`${region.occurrences} 张`, 560, y, 25, color.accent)
  })
  text('统计口径：开售 10 分钟后', 60, 795, 21, color.muted)
  context.drawImage(codePath, 512, 822, 156, 156)
  text('足球洞察', 60, 861, 32)
  text('球迷的回流观察站', 60, 904, 23, color.muted)
  text('扫码看实时回流', 494, 1008, 22, color.muted)
  if (content.cutoffLabel) {
    text('数据读取截止（北京时间）', 60, 940, 20, color.muted)
    text(content.cutoffLabel, 60, 970, 20, color.muted)
  }
  wrapped(content.disclaimer, 60, content.cutoffLabel ? 1004 : 994, 20, 418, 2, color.muted)
}
