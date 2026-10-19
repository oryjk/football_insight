import type { PublicLineup } from '../../types/lineup'
import { buildPitchPlayers } from './lineupViewModel'
import { buildLineupViewUrl } from '../../utils/lineupLinks'

export const LINEUP_POSTER = { canvasId: 'lineup-share-canvas', width: 750, height: 1200 }
export const LINEUP_CARD = { canvasId: 'lineup-card-canvas', width: 750, height: 600 }
export function canShareLineup(slots: Array<{ player_id: number | null }>): boolean {
  return slots.length === 11 && slots.every(slot => slot.player_id !== null) && new Set(slots.map(slot => slot.player_id)).size === 11
}
export function publicPitch(lineup: PublicLineup) {
  return buildPitchPlayers(lineup.slots, lineup.formation, {
    team: lineup.team, active: lineup.players.filter(player => !player.former), former: lineup.players.filter(player => player.former),
  })
}
export function lineupSharePayload(lineup: PublicLineup, imageUrl = '') {
  return { title: `${lineup.author.display_name}心中的${lineup.team.team_name} · ${lineup.formation}`, path: buildLineupViewUrl(lineup.share_code, true), ...(imageUrl ? { imageUrl } : {}) }
}
export function posterPlayerPoint(x: number, y: number, card = false) {
  // Coordinates follow the editor: x=0..1 left-to-right, y=0..1 top-to-bottom.
  return card ? { x: 170 + Math.max(0, Math.min(1, x)) * 410, y: 140 + Math.max(0, Math.min(1, y)) * 370 }
    : { x: 90 + Math.max(0, Math.min(1, x)) * 570, y: 245 + Math.max(0, Math.min(1, y)) * 625 }
}
// Decorative bitmap palette, independent of UI theme tokens.
function clippedLabel(value: string, limit: number): string {
  const chars = Array.from(value)
  return chars.length > limit ? chars.slice(0, limit - 1).join('') + '…' : value
}
const palette = { paper: '#ffffff', ink: '#15161b', muted: '#6d7280', grass: '#28734f', alternate: '#2d7c56', line: '#a8c9b6', red: '#d92537' }
export function drawLineupPoster(context: UniApp.CanvasContext, lineup: PublicLineup, images: Map<string, string>, codePath: string, card = false): void {
  if (!card && !codePath) throw new Error('小程序码未就绪，请重试')
  const height = card ? 600 : 1200
  const text = (value: string, x: number, y: number, size: number, color = palette.ink) => {
    context.setFillStyle(color); context.setFontSize(size); context.fillText(value, x, y)
  }
  context.setFillStyle(palette.paper); context.fillRect(0, 0, 750, height)
  text('足球洞察 · 我心中的最佳阵容', 32, 49, card ? 23 : 26, palette.red)
  const crest = lineup.team.avatar_url && images.get(lineup.team.avatar_url)
  if (crest) context.drawImage(crest, 32, card ? 61 : 74, 48, 48)
  text(clippedLabel(lineup.team.team_name, 12), crest ? 96 : 32, card ? 96 : 109, 36)
  text(lineup.formation, 580, card ? 96 : 109, 32, palette.red)
  const field = card ? { x: 135, y: 120, w: 480, h: 420 } : { x: 32, y: 190, w: 686, h: 745 }
  for (let stripe = 0; stripe < 8; stripe++) {
    context.setFillStyle(stripe % 2 ? palette.alternate : palette.grass)
    context.fillRect(field.x, field.y + stripe * field.h / 8, field.w, field.h / 8)
  }
  context.setStrokeStyle(palette.line); context.setLineWidth(2)
  context.strokeRect(field.x + 15, field.y + 15, field.w - 30, field.h - 30)
  context.beginPath(); context.moveTo(field.x + 15, field.y + field.h / 2); context.lineTo(field.x + field.w - 15, field.y + field.h / 2); context.stroke()
  context.beginPath(); context.arc(375, field.y + field.h / 2, field.w * .12, 0, Math.PI * 2); context.stroke()
  context.strokeRect(field.x + field.w * .25, field.y + 15, field.w * .5, field.h * .14)
  context.strokeRect(field.x + field.w * .25, field.y + field.h * .86 - 15, field.w * .5, field.h * .14)
  for (const entry of publicPitch(lineup)) {
    const player = entry.player
    if (!player) continue
    const point = posterPlayerPoint(entry.x, entry.y, card), radius = card ? 19 : 29
    const avatar = player.avatar_url && images.get(player.avatar_url)
    context.save(); context.beginPath(); context.arc(point.x, point.y, radius, 0, Math.PI * 2); context.clip()
    context.setFillStyle(palette.paper); context.fillRect(point.x - radius, point.y - radius, radius * 2, radius * 2)
    if (avatar) context.drawImage(avatar, point.x - radius, point.y - radius, radius * 2, radius * 2)
    else { context.setTextAlign('center'); text(String(player.shirt_number ?? entry.label), point.x, point.y + 8, card ? 16 : 23) }
    context.restore()
    // A real headshot still carries its shirt number. Former players retain the 前 badge.
    if (avatar) {
      const badge = card ? 17 : 23
      context.setFillStyle(palette.paper); context.fillRect(point.x + radius - badge / 2, point.y + radius - badge, badge, badge)
      context.setTextAlign('center'); text(String(player.shirt_number ?? '—'), point.x + radius, point.y + radius - (card ? 4 : 5), card ? 12 : 16)
    }
    if (player.former) {
      const badge = card ? 19 : 25
      context.setFillStyle(palette.red); context.fillRect(point.x - radius - 4, point.y - radius - 4, badge, badge)
      context.setTextAlign('center'); text('前', point.x - radius - 4 + badge / 2, point.y - radius - 4 + badge - 4, card ? 13 : 17, palette.paper)
    }
    context.setTextAlign('center'); text(clippedLabel(player.name, 8), point.x, point.y + radius + (card ? 18 : 25), card ? 16 : 21, palette.paper)
    context.setTextAlign('left')
  }
  if (card) { text(`${clippedLabel(lineup.author.display_name, 16)}的战术板`, 32, 580, 23, palette.muted); return }
  text(`${clippedLabel(lineup.author.display_name, 16)}的战术板`, 32, 161, 24, palette.muted)
  const authorAvatar = lineup.author.avatar_url && images.get(lineup.author.avatar_url)
  if (authorAvatar) context.drawImage(authorAvatar, 40, 980, 62, 62)
  text('我也排一套', authorAvatar ? 120 : 40, 1017, 32)
  text('扫码查看阵容、为球迷点赞', 40, 1070, 23, palette.muted)
  text('正式版扫码打开 · 足球洞察', 40, 1141, 21, palette.muted)
  context.drawImage(codePath, 528, 974, 174, 174)
}
