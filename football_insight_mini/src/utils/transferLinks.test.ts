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

describe('transfer eligibility messages', () => {
  test('logged-in users who cannot vote are told why; guests and voters get nothing', async () => {
    const { transferBlockedMessage, TRANSFER_HINTS } = await import('./transferLinks')
    expect(transferBlockedMessage({ loggedIn: true, canVote: false, open: true })).toBe(TRANSFER_HINTS.notFan)
    expect(transferBlockedMessage({ loggedIn: true, canVote: false, open: false })).toBe(TRANSFER_HINTS.closed)
    expect(transferBlockedMessage({ loggedIn: false, canVote: false, open: true })).toBe('')
    expect(transferBlockedMessage({ loggedIn: true, canVote: true, open: true })).toBe('')
    expect(TRANSFER_HINTS.notFan).toBe('只能为你的主队投票，可在首页「我的主队」设置')
  })

  test('the inline hint only shows while voting is open and the viewer cannot vote', async () => {
    const { transferHint, TRANSFER_HINTS } = await import('./transferLinks')
    expect(transferHint({ loggedIn: true, canVote: false, open: true })).toBe(TRANSFER_HINTS.notFan)
    expect(transferHint({ loggedIn: false, canVote: false, open: true })).toBe(TRANSFER_HINTS.guest)
    expect(transferHint({ loggedIn: true, canVote: false, open: false })).toBe('')
    expect(transferHint({ loggedIn: true, canVote: true, open: true })).toBe('')
  })
})

test('board labels read as what fans want, not as feature names', async () => {
  const { TRANSFER_BOARD_LABELS } = await import('./transferLinks')
  expect(TRANSFER_BOARD_LABELS).toEqual({ out: '最想送走', stay: '最想留下', in: '最想买入' })
})

test('scene_parsing_accepts_only_transfer_cards', async () => {
  const { parseTransferCode } = await import('./transferLinks')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=transfer&t=ABCD1234') })).toBe('ABCD1234')
  expect(parseTransferCode({ scene: 's=share&c=transfer&t=ABCD1234' })).toBe('ABCD1234')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=lineup&l=ABCD1234') })).toBe('')
  expect(parseTransferCode({ scene: encodeURIComponent('s=share&c=transfer&t=bad') })).toBe('')
  expect(parseTransferCode({ scene: '%E0%A4%A' })).toBe('')
  expect(parseTransferCode({ code: 'ABCD1234', scene: 'whatever' })).toBe('ABCD1234')
})
