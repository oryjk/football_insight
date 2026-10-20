import { describe, expect, test } from 'bun:test'
import type { TransferCandidates, TransferIntent, TransferNominateResult } from '../../types/transfer'
import { ApiRequestError } from '../../utils/apiError'
import { createTransferNominate } from './useTransferNominate'

const candidates = (intent: TransferIntent): TransferCandidates => ({ intent, groups: [{ key: 'squad', title: 'T', players: [] }] })
const result = (created: boolean) => ({ created, view: { card: { share_code: 'ABCD1234' } } }) as unknown as TransferNominateResult

describe('transfer nominate', () => {
  test('candidates are cached per intent and the sheet reopens on the chosen intent', async () => {
    const calls: TransferIntent[] = []
    const state = createTransferNominate({ candidates: async (_, intent) => { calls.push(intent); return candidates(intent) }, loggedIn: () => true })
    await state.open(1, 'in'); await state.changeIntent('out'); await state.changeIntent('in')
    expect(calls).toEqual(['in', 'out'])
    expect(state.visible.value).toBe(true)
    expect(state.intent.value).toBe('in')
  })

  test('submit reports created vs merged and server messages', async () => {
    const notices: string[] = []
    let mode: 'new' | 'merge' | 'limit' = 'new'
    const state = createTransferNominate({
      candidates: async (_, intent) => candidates(intent), loggedIn: () => true, notify: m => notices.push(m),
      nominate: async () => {
        if (mode === 'limit') throw new ApiRequestError('本期这一类最多提名 3 位', 429)
        return result(mode === 'new')
      },
    })
    await state.open(1, 'out')
    expect((await state.submit(9))?.created).toBe(true)
    mode = 'merge'; expect((await state.submit(9))?.created).toBe(false)
    mode = 'limit'; expect(await state.submit(9)).toBe(null)
    expect(notices).toEqual(['提名成功', '已有球迷提名，已为你加一票', '本期这一类最多提名 3 位'])
    // 成功后关闭弹层；之后的失败不会重新打开它
    expect(state.visible.value).toBe(false)
  })

  test('guests are sent to log in before the sheet opens', async () => {
    const logins: string[] = []
    const state = createTransferNominate({ candidates: async (_, intent) => candidates(intent), loggedIn: () => false, login: url => logins.push(url) })
    await state.open(1, 'stay')
    expect(state.visible.value).toBe(false)
    expect(logins).toEqual(['/pages/transfer/board?teamId=1&intent=stay&nominate=1'])
  })
})
