import { describe, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { TransferSummary } from '../../types/transfer'
import { useInsightsTransfer, type InsightsTransferOptions } from './useInsightsTransfer'

function summary(teamId: number, canVote = true): TransferSummary {
  return {
    team: { team_id: teamId, team_name: `T${teamId}`, avatar_url: null },
    window: { code: 'w', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
    can_vote: canVote,
    boards: [{ intent: 'out', total: 1, items: [{
      share_code: 'ABCD1234', intent: 'out', returning: false, up: 1, down: 0, my_vote: 0, rank: 1, created_at: '',
      player: { player_id: 1, name: 'X', avatar_url: null, position: null, shirt_number: null, current_team: null, departed: false },
    }] }, { intent: 'stay', total: 0, items: [] }, { intent: 'in', total: 0, items: [] }],
  }
}
// 只提供 useInsightsTransfer 用到的会话字段；整体断言为参数类型以通过 type-check（测试文件在 tsconfig 范围内）。
function session(loggedIn = true): InsightsTransferOptions['session'] {
  return {
    currentUser: ref(loggedIn ? { id: 'u' } : null), hasAccessToken: ref(loggedIn),
    status: ref(loggedIn ? 'authenticated' : 'guest'),
    revision: ref(0), invalidateSession: () => {},
  } as unknown as InsightsTransferOptions['session']
}
const flush = async () => { await nextTick(); await Promise.resolve(); await Promise.resolve(); await nextTick() }

describe('insights transfer', () => {
  test('loads only while active and follows the selected team', async () => {
    const calls: number[] = []
    const active = ref(false), team = ref<number | null>(1)
    const state = useInsightsTransfer({ session: session(), selectedTeamId: team, active, deps: { summary: async id => { calls.push(id); return summary(id) } } })
    await flush(); expect(calls).toEqual([])
    active.value = true; await flush(); expect(calls).toEqual([1])
    team.value = 2; await flush(); expect(calls).toEqual([1, 2])
    expect(state.summary.value?.team.team_id).toBe(2)
    state.dispose()
  })

  test('a slow response for the previous team is dropped', async () => {
    let releaseFirst: (s: TransferSummary) => void = () => {}
    const team = ref<number | null>(1)
    const state = useInsightsTransfer({ session: session(), selectedTeamId: team, active: ref(true), deps: {
      summary: id => id === 1 ? new Promise(r => { releaseFirst = r }) : Promise.resolve(summary(2)),
    } })
    await flush(); team.value = 2; await flush()
    releaseFirst(summary(1)); await flush()
    expect(state.summary.value?.team.team_id).toBe(2)
    state.dispose()
  })

  test('votes patch the card; non-fans get a hint instead of a request', async () => {
    const votes: Array<[string, number]> = [], notices: string[] = []
    const fan = useInsightsTransfer({ session: session(), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id), vote: async (code, value) => { votes.push([code, value]); return { up: 2, down: 0, my_vote: 1 } }, notify: m => notices.push(m),
    } })
    await flush()
    await fan.vote(fan.summary.value!.boards[0].items[0], 1)
    expect(votes).toEqual([['ABCD1234', 1]])
    expect(fan.summary.value!.boards[0].items[0].up).toBe(2)
    fan.dispose()
    const rival = useInsightsTransfer({ session: session(), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id, false), vote: async () => { throw new Error('should not be called') }, notify: m => notices.push(m),
    } })
    await flush()
    expect(rival.hint.value).toBe('只能为你的主队投票，可在首页「我的主队」设置')
    await rival.vote(rival.summary.value!.boards[0].items[0], 1)
    expect(notices).toEqual(['只能为你的主队投票，可在首页「我的主队」设置'])
    // 提名同样先拦截，不打开弹层
    expect(rival.nominateBlocked()).toBe('只能为你的主队投票，可在首页「我的主队」设置')
    rival.dispose()
  })

  test('guests see a login hint and voting asks to log in', async () => {
    const logins: string[] = []
    const guest = useInsightsTransfer({ session: session(false), selectedTeamId: ref(1), active: ref(true), deps: {
      summary: async id => summary(id, false), login: url => logins.push(url),
    } })
    await flush()
    expect(guest.hint.value).toBe('登录并设置主队后可以投票')
    // 游客提名交给登录流程，不在这里拦
    expect(guest.nominateBlocked()).toBe('')
    await guest.vote(guest.summary.value!.boards[0].items[0], -1)
    expect(logins).toEqual(['/pages/insights/index'])
    guest.dispose()
  })
})
