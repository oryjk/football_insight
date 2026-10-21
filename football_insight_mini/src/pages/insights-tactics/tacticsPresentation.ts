import type { SavedLineup, SavedLineupDraft } from '../../types/lineup'

export function myTacticsSummary(draft: SavedLineupDraft | null, published: SavedLineup | null) {
  const snapshot = draft ?? published
  const canonical = (value: SavedLineupDraft) => JSON.stringify([value.formation,
    [...value.slots].sort((a, b) => a.slot - b.slot).map(({ slot, player_id, x, y }) => [slot, player_id, x, y])])
  const unpublished = Boolean(draft && (!published || canonical(draft) !== canonical(published)))
  return {
    snapshot,
    filled: snapshot?.slots.filter(slot => slot.player_id !== null).length ?? 0,
    status: unpublished ? (published ? '有未发布的修改' : '草稿已保存') : published ? '已发布' : '还没有战术',
    // 按用户所处阶段邀请：还没排 → 排我的阵容；有没发布的草稿 → 继续排阵；已发布且无改动 → 改我的阵容。
    action: !snapshot ? '排我的阵容' : unpublished ? '继续排阵' : '改我的阵容',
  }
}

export function miniPitchPosition(x: number, y: number): string {
  const clamp = (value: number) => Number.isFinite(value) ? Math.min(94, Math.max(6, value * 100)) : 50
  return `left:${clamp(x)}%;top:${clamp(y)}%`
}
