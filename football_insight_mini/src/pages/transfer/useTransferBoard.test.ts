import { describe, expect, test } from 'bun:test'
import type { TransferBoard, TransferIntent } from '../../types/transfer'
import { createTransferBoard } from './useTransferBoard'

function page(intent: TransferIntent, pageNo: number, total = 25): TransferBoard {
  const start = (pageNo - 1) * 20
  return {
    team: { team_id: 1, team_name: 'T1', avatar_url: null },
    window: { code: 'w', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
    intent, page: pageNo, page_size: 20, total, can_vote: true,
    items: Array.from({ length: Math.max(0, Math.min(20, total - start)) }, (_, i) => ({
      share_code: `C${String(start + i).padStart(7, '0')}`, intent, returning: false, up: 1, down: 0, my_vote: 0 as const, rank: start + i + 1, created_at: '',
      player: { player_id: start + i, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
    })),
  }
}
const base = { revision: () => 0, loggedIn: () => true, invalidate: () => {}, login: () => {}, notify: () => {} }

describe('transfer board page', () => {
  test('paginates without duplicates and stops at total', async () => {
    const calls: Array<[TransferIntent, number]> = []
    const state = createTransferBoard({ ...base, board: async (_, intent, p) => { calls.push([intent, p ?? 1]); return page(intent, p ?? 1) } })
    await state.load(1, 'in')
    await state.more(); await state.more()
    expect(calls).toEqual([['in', 1], ['in', 2]])
    expect(state.items.value.length).toBe(25)
  })

  test('switching intent drops the old list and late responses', async () => {
    let releaseOut: (b: TransferBoard) => void = () => {}
    const state = createTransferBoard({ ...base, board: (_, intent, p) => intent === 'out' ? new Promise(r => { releaseOut = r }) : Promise.resolve(page(intent, p ?? 1, 3)) })
    const first = state.load(1, 'out')
    await state.changeIntent('stay')
    releaseOut(page('out', 1)); await first
    expect(state.intent.value).toBe('stay')
    expect(state.items.value.every(item => item.intent === 'stay')).toBe(true)
  })

  test('votes patch in place; closed windows notify instead of writing', async () => {
    const notices: string[] = [], votes: string[] = []
    const state = createTransferBoard({ ...base, notify: m => notices.push(m),
      board: async (_, intent, p) => ({ ...page(intent, p ?? 1, 2), can_vote: false, window: { code: 'w', name: 'w', opens_at: '', closes_at: '', open: false } }),
      vote: async code => { votes.push(code); return { up: 9, down: 0, my_vote: 1 } } })
    await state.load(1, 'out')
    await state.vote(state.items.value[0], 1)
    expect(votes).toEqual([]); expect(notices).toEqual(['本期投票已结束'])
    const open = createTransferBoard({ ...base, board: async (_, intent, p) => page(intent, p ?? 1, 2), vote: async () => ({ up: 9, down: 0, my_vote: 1 }) })
    await open.load(1, 'out')
    await open.vote(open.items.value[1], 1)
    expect(open.items.value[1].up).toBe(9)
  })

  test('non-fans are stopped before the nominate sheet opens; fans and guests are not', async () => {
    const fan = createTransferBoard({ ...base, board: async (_, intent, p) => page(intent, p ?? 1, 2) })
    await fan.load(1, 'out')
    expect(fan.nominateBlocked()).toBe('')
    const rival = createTransferBoard({ ...base, board: async (_, intent, p) => ({ ...page(intent, p ?? 1, 2), can_vote: false }) })
    await rival.load(1, 'out')
    expect(rival.nominateBlocked()).toBe('只能为你的主队投票，可在首页「我的主队」设置')
    const guest = createTransferBoard({ ...base, loggedIn: () => false, board: async (_, intent, p) => ({ ...page(intent, p ?? 1, 2), can_vote: false }) })
    await guest.load(1, 'out')
    expect(guest.nominateBlocked()).toBe('')
  })
})
