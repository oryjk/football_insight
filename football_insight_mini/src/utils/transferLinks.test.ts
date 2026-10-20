import { describe, expect, test } from 'bun:test'
import type { TransferCard } from '../types/transfer'
import {
  applyTally, buildTransferBoardUrl, buildTransferViewUrl, parseTransferBoardQuery, parseTransferCode,
  positionLabel, transferBoardShareTitle, transferCardShareTitle, transferStance, validTransferCode,
} from './transferLinks'

function card(overrides: Partial<TransferCard> = {}): TransferCard {
  return {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 3, down: 1, my_vote: 0, rank: 1, created_at: '',
    player: { player_id: 1, name: '某某', avatar_url: null, position: 'FW', shirt_number: 9, current_team: null, departed: false },
    ...overrides,
  }
}

describe('transfer links', () => {
  test('board and view urls round trip', () => {
    expect(buildTransferBoardUrl(77680, 'in')).toBe('/pages/transfer/board?teamId=77680&intent=in')
    expect(buildTransferBoardUrl(77680, 'out', { nominate: true })).toBe('/pages/transfer/board?teamId=77680&intent=out&nominate=1')
    expect(buildTransferBoardUrl(0)).toBe('/pages/transfer/board')
    expect(parseTransferBoardQuery({ teamId: '77680', intent: 'stay', nominate: '1' })).toEqual({ teamId: 77680, intent: 'stay', nominate: true })
    expect(parseTransferBoardQuery({ teamId: 'x', intent: 'nope' })).toEqual({ teamId: 0, intent: 'out', nominate: false })
    expect(buildTransferViewUrl('ABCD1234')).toBe('/pages/transfer/view?code=ABCD1234')
    expect(buildTransferViewUrl('bad')).toBe('')
    expect(parseTransferCode({ code: 'ABCD1234' })).toBe('ABCD1234')
    expect(parseTransferCode({ code: '../x' })).toBe('')
    expect(validTransferCode('abc')).toBe(false)
  })

  test('stance wording covers returning former players', () => {
    expect(transferStance(card())).toBe('离队')
    expect(transferStance(card({ intent: 'stay' }))).toBe('留下')
    expect(transferStance(card({ intent: 'in' }))).toBe('加盟')
    expect(transferStance(card({ intent: 'in', returning: true }))).toBe('回归')
  })

  test('share titles', () => {
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'out')).toBe('成都蓉城球迷最想送走谁？2026 冬窗转会榜')
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'stay')).toBe('成都蓉城球迷最想留下谁？2026 冬窗转会榜')
    expect(transferBoardShareTitle('成都蓉城', '2026 冬窗', 'in')).toBe('成都蓉城球迷最想买入谁？2026 冬窗转会榜')
    expect(transferCardShareTitle(card({ intent: 'in', returning: true }))).toBe('我希望 某某 回归，你同意吗？')
  })

  test('tally replaces counts and my vote only', () => {
    const next = applyTally(card(), { up: 4, down: 0, my_vote: 1 })
    expect([next.up, next.down, next.my_vote, next.rank, next.share_code]).toEqual([4, 0, 1, 1, 'ABCD1234'])
  })

  test('position labels', () => {
    expect([positionLabel('GK'), positionLabel('DF'), positionLabel('MF'), positionLabel('FW'), positionLabel(null)]).toEqual(['门将', '后卫', '中场', '前锋', ''])
  })
})
