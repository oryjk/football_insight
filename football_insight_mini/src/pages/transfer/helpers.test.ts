import { expect, test } from 'bun:test'
import type { TransferCandidateGroup } from '../../types/transfer'
import { NOMINATE_INTENT_ITEMS, filterCandidateGroups } from './helpers'

const player = (id: number, name: string) => ({ player_id: id, name, shirt_number: null, position: null, avatar_url: null, current_team: null })
const groups: TransferCandidateGroup[] = [
  { key: 'former', title: '本队旧将', players: [player(1, '张三'), player(2, '李四')] },
  { key: 'team:2', title: '上海海港', players: [player(3, '张飞')] },
]

test('keyword filters players and drops empty groups; blank keeps everything', () => {
  expect(filterCandidateGroups(groups, '  ')).toEqual(groups)
  expect(filterCandidateGroups(groups, '张').map(g => [g.key, g.players.map(p => p.player_id)])).toEqual([['former', [1]], ['team:2', [3]]])
  expect(filterCandidateGroups(groups, '李').map(g => g.key)).toEqual(['former'])
  expect(filterCandidateGroups(groups, '王')).toEqual([])
})

test('nominate intents are labelled as stances', () => {
  expect(NOMINATE_INTENT_ITEMS).toEqual([
    { key: 'out', label: '希望离队' }, { key: 'stay', label: '希望留下' }, { key: 'in', label: '希望买入' },
  ])
})
