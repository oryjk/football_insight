import type { TransferCardView } from '../../types/transfer'
import { TRANSFER_BOARD_LABELS, transferStance } from '../../utils/transferLinks'

export const TRANSFER_POSTER = { canvasId: 'transfer-share-canvas', width: 750, height: 1200 }
export interface TransferSharer { display_name: string; avatar_url: string | null }

// 位图装饰色，与界面主题 token 无关（沿用 lineupPoster 的约定）。
const palette = { paper: '#ffffff', soft: '#f7f8fa', ink: '#15161b', muted: '#6d7280', red: '#dc2626', redSoft: '#fff1f0', line: '#ececf1' }

function clipped(value: string, limit: number): string {
  const chars = Array.from(value)
  return chars.length > limit ? chars.slice(0, limit - 1).join('') + '…' : value
}

export function transferPosterHeadline(view: TransferCardView): string {
  return `${view.team.team_name}球迷希望他${transferStance(view.card)}`
}

export function drawTransferPoster(context: UniApp.CanvasContext, view: TransferCardView, images: Map<string, string>, codePath: string, sharer: TransferSharer): void {
  if (!codePath) throw new Error('小程序码未就绪，请重试')
  const { card, team, window } = view
  const text = (value: string, x: number, y: number, size: number, color = palette.ink) => {
    context.setFillStyle(color); context.setFontSize(size); context.fillText(value, x, y)
  }
  const circleImage = (path: string | undefined, cx: number, cy: number, r: number, fallback: string, fallbackSize: number) => {
    context.save(); context.beginPath(); context.arc(cx, cy, r, 0, Math.PI * 2); context.clip()
    context.setFillStyle(palette.soft); context.fillRect(cx - r, cy - r, r * 2, r * 2)
    if (path) context.drawImage(path, cx - r, cy - r, r * 2, r * 2)
    else { context.setTextAlign('center'); text(fallback, cx, cy + fallbackSize * 0.35, fallbackSize, palette.muted) }
    context.restore(); context.setTextAlign('left')
  }

  context.setFillStyle(palette.paper); context.fillRect(0, 0, 750, 1200)
  // 顶部：队徽 + 球队名 + 期名
  const crest = team.avatar_url ? images.get(team.avatar_url) : undefined
  if (crest) context.drawImage(crest, 40, 48, 64, 64)
  text(clipped(team.team_name, 12), crest ? 120 : 40, 92, 34)
  text(`${window.name} · 转会表态`, 40, 152, 26, palette.red)

  // 中部：球员大头像、名字、表态
  const avatar = card.player.avatar_url ? images.get(card.player.avatar_url) : undefined
  circleImage(avatar, 375, 360, 130, Array.from(card.player.name)[0] ?? '球', 96)
  context.setTextAlign('center')
  text(clipped(card.player.name, 10), 375, 560, 48)
  text(transferPosterHeadline(view), 375, 628, 36, palette.red)
  context.setTextAlign('left')

  // 数据行：赞成 / 反对 / 名次
  context.setFillStyle(palette.redSoft); context.fillRect(40, 680, 670, 120)
  context.setTextAlign('center')
  text(`赞成 ${card.up}`, 190, 755, 34, palette.red)
  text(`反对 ${card.down}`, 375, 755, 34, palette.ink)
  if (card.rank !== null) text(`${TRANSFER_BOARD_LABELS[card.intent]}第 ${card.rank} 名`, 570, 755, 28, palette.muted)
  context.setTextAlign('left')

  // 底部：分享人 + 小程序码
  context.setStrokeStyle(palette.line); context.setLineWidth(2); context.strokeRect(40, 850, 670, 0)
  const name = sharer.display_name.trim() || '球迷'
  const sharerAvatar = sharer.avatar_url ? images.get(sharer.avatar_url) : undefined
  circleImage(sharerAvatar, 92, 960, 52, Array.from(name)[0] ?? '球', 40)
  text(clipped(name, 12), 164, 945, 30)
  text('邀你表态：你同意吗？', 164, 990, 24, palette.muted)
  text('扫码投一票 · 足球洞察', 40, 1110, 24, palette.muted)
  text('正式版扫码打开', 40, 1150, 20, palette.muted)
  context.drawImage(codePath, 520, 900, 190, 190)
}
