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
    action: snapshot ? '继续排阵' : '开始排阵',
  }
}

export function miniPitchPosition(x: number, y: number): string {
  const clamp = (value: number) => Number.isFinite(value) ? Math.min(94, Math.max(6, value * 100)) : 50
  return `left:${clamp(x)}%;top:${clamp(y)}%`
}
