import type { PublicLineup, SavedLineup } from '../../types/lineup'
import { canShareLineup } from './lineupPoster'
import type { SaveResult } from './useLineupPage'

export interface EditorShareSource {
  context: () => string
  slots: () => Array<{ player_id: number | null }>
  save: () => Promise<SaveResult>
  saved: () => SavedLineup | null
  dirty: () => boolean
  detail: (code: string) => Promise<PublicLineup>
}
/** Capture identity before saving, then confirm the exact editor revision before publishing. */
export async function prepareEditorShare(source: EditorShareSource): Promise<{ result: SaveResult | 'incomplete'; lineup?: PublicLineup }> {
  if (!canShareLineup(source.slots())) return { result: 'incomplete' }
  const context = source.context()
  const saved = await source.save()
  if (saved !== 'saved') return { result: saved }
  if (source.context() !== context) return { result: 'cancelled' }
  const snapshot = source.saved()
  if (!snapshot || source.dirty() || !canShareLineup(snapshot.slots)) return { result: 'cancelled' }
  let lineup: PublicLineup
  try { lineup = await source.detail(snapshot.share_code) }
  catch (error) { if (source.context() !== context) return { result: 'cancelled' }; throw error }
  if (source.context() !== context || source.dirty()) return { result: 'cancelled' }
  return { result: 'saved', lineup }
}
