import type { TransferCandidateGroup, TransferIntent } from '../../types/transfer'

export const NOMINATE_INTENT_ITEMS: Array<{ key: TransferIntent; label: string }> = [
  { key: 'out', label: '希望离队' }, { key: 'stay', label: '希望留下' }, { key: 'in', label: '希望买入' },
]

/** 本地按名字过滤候选（关键字不提交后端）；过滤后为空的分组不显示。 */
export function filterCandidateGroups(groups: TransferCandidateGroup[], keyword: string): TransferCandidateGroup[] {
  const word = keyword.trim()
  if (!word) return groups
  return groups
    .map(group => ({ ...group, players: group.players.filter(player => player.name.includes(word)) }))
    .filter(group => group.players.length > 0)
}
