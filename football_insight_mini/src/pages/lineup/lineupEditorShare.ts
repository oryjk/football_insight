import type { PublicLineup, SavedLineup } from '../../types/lineup'
import { canShareLineup } from './lineupPoster'
import type { SaveResult } from './useLineupPage'

export interface EditorShareSource {
  context: () => string
  saved: () => SavedLineup | null
  /** 当前内容是否与公开快照有差异，不能用 editor.dirty。 */
  dirty: () => boolean
  detail: (code: string) => Promise<PublicLineup>
}
/** 只读取已明确发布的版本；分享不会保存或发布编辑内容。 */
export async function prepareEditorShare(source: EditorShareSource): Promise<{ result: SaveResult | 'needs-publish'; lineup?: PublicLineup }> {
  const snapshot = source.saved()
  if (!snapshot || source.dirty()) return { result: 'needs-publish' }
  if (!canShareLineup(snapshot.slots)) return { result: 'incomplete' }
  const context = source.context()
  let lineup: PublicLineup
  try { lineup = await source.detail(snapshot.share_code) }
  catch (error) { if (source.context() !== context) return { result: 'cancelled' }; throw error }
  if (source.context() !== context || source.dirty() || source.saved()?.share_code !== snapshot.share_code) return { result: 'cancelled' }
  return { result: 'saved', lineup }
}
