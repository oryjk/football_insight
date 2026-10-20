import { describe, expect, test } from 'bun:test'
import type { TransferCard, TransferTally } from '../types/transfer'
import { ApiRequestError } from '../utils/apiError'
import { createTransferVotes, type TransferVoteDeps } from './useTransferVotes'

function card(my_vote: -1 | 0 | 1 = 0): TransferCard {
  return {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 2, down: 1, my_vote, rank: 1, created_at: '',
    player: { player_id: 1, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
  }
}
function deps(overrides: Partial<TransferVoteDeps> = {}) {
  const calls: Array<[string, number]> = [], logins: string[] = [], invalidated: number[] = []
  let revision = 0
  const base: TransferVoteDeps = {
    vote: async (code, value) => { calls.push([code, value]); return { up: 3, down: 0, my_vote: value } as TransferTally },
    loggedIn: () => true,
    revision: () => revision,
    invalidate: r => { invalidated.push(r) },
    login: url => { logins.push(url) },
    ...overrides,
  }
  return { base, calls, logins, invalidated, bump: () => { revision++ } }
}
const always = () => true

describe('transfer votes', () => {
  test('tapping the same side clears it, the other side replaces it', async () => {
    const d = deps(); const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    await votes.toggle(card(0), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    await votes.toggle(card(1), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    await votes.toggle(card(1), -1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    expect(d.calls).toEqual([['ABCD1234', 1], ['ABCD1234', 0], ['ABCD1234', -1]])
    expect(published.length).toBe(3)
  })

  test('guest_vote_remembers_the_card_url', async () => {
    const d = deps({ loggedIn: () => false }); const votes = createTransferVotes(d.base)
    expect(await votes.toggle(card(), 1, { returnUrl: '/pages/transfer/view?code=ABCD1234', current: always, publish: () => {} })).toBe('')
    expect(d.logins).toEqual(['/pages/transfer/view?code=ABCD1234'])
    expect(d.calls).toEqual([])
  })

  test('a card is locked while its request is in flight', async () => {
    let release: (t: TransferTally) => void = () => {}
    const d = deps({ vote: async () => new Promise<TransferTally>(r => { release = r }) })
    const votes = createTransferVotes(d.base)
    const first = votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: () => {} })
    expect(votes.pending.value).toEqual(['ABCD1234'])
    expect(await votes.toggle(card(), -1, { returnUrl: '/u', current: always, publish: () => {} })).toBe('')
    release({ up: 1, down: 0, my_vote: 1 }); await first
    expect(votes.pending.value).toEqual([])
  })

  test('server_rejection_keeps_tally_and_returns_message', async () => {
    const d = deps({ vote: async () => { throw new ApiRequestError('本期投票已结束', 409) } })
    const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    expect(await votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })).toBe('本期投票已结束')
    expect(published).toEqual([])
  })

  test('401 invalidates the session and asks to log in', async () => {
    const d = deps({ vote: async () => { throw new ApiRequestError('请登录后继续', 401) } })
    const votes = createTransferVotes(d.base)
    expect(await votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: () => {} })).toBe('')
    expect(d.invalidated).toEqual([0]); expect(d.logins).toEqual(['/u'])
  })

  test('a response from an older identity or context is dropped', async () => {
    let release: (t: TransferTally) => void = () => {}
    const d = deps({ vote: async () => new Promise<TransferTally>(r => { release = r }) })
    const votes = createTransferVotes(d.base); const published: TransferTally[] = []
    const pending = votes.toggle(card(), 1, { returnUrl: '/u', current: always, publish: t => published.push(t) })
    d.bump(); release({ up: 1, down: 0, my_vote: 1 }); await pending
    expect(published).toEqual([])
    let stillCurrent = true
    const second = votes.toggle(card(), 1, { returnUrl: '/u', current: () => stillCurrent, publish: t => published.push(t) })
    stillCurrent = false; release({ up: 1, down: 0, my_vote: 1 }); await second
    expect(published).toEqual([])
  })
})
