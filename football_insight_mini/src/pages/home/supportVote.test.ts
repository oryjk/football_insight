import { expect, test } from 'bun:test'
import { ref } from 'vue'
import { useHomeSupportData } from './useHomeSupportData'
import { resolveHomeSupportVoteState } from './helpers'
import type { AppSession } from '../../stores/appSession'
import type { SupportMatchDetail, SupportProfile } from '../../types/support'

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: unknown) => void
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej })
  return { promise, resolve, reject }
}

async function expectFailure(promise: Promise<unknown>, message: string) {
  let failure: unknown
  try { await promise } catch (error) { failure = error }
  expect(failure).toBeInstanceOf(Error)
  expect((failure as Error).message.includes(message)).toBe(true)
}

function profile(): SupportProfile {
  const team = (id: number) => ({ team_id: id, team_name: `球队${id}`, avatar_storage_url: null,
    score: '-', support_count: 0, support_share_pct: 0, season_support_rank: null })
  return {
    favorite_team: { team_id: 2, team_name: '球队2', avatar_storage_url: null, rank_no: 1 },
    next_match: { match_id: 10, season: 2026, round_number: 28, match_date: '2030-10-18',
      match_time: '20:00', kickoff_at: '2030-10-18T12:00:00Z', status: 'scheduled',
      support_window_status: 'open', countdown_seconds: 100, total_support_count: 0,
      home_team: team(1), away_team: team(2),
      viewer: { favorite_team_id: 2, supported_team_id: null, has_supported: false, can_support: true } },
  }
}

function voted(): SupportMatchDetail {
  const match = profile().next_match!
  return { ...match, total_support_count: 1,
    away_team: { ...match.away_team, support_count: 1, support_share_pct: 100 },
    viewer: { ...match.viewer, has_supported: true, can_support: false, supported_team_id: 2 } }
}

function fixture() {
  const revision = ref(1)
  const hasAccessToken = ref(true)
  const writes: Array<{ matchId: number; teamId: number; request: ReturnType<typeof deferred<SupportMatchDetail>> }> = []
  const reads: Array<ReturnType<typeof deferred<SupportProfile>>> = []
  const session = { revision, hasAccessToken, currentUser: ref({ id: 'fixture' }),
    ensureSessionReady: async () => ({ id: 'fixture' }),
    invalidateSession: (expected: number) => { if (revision.value === expected) { revision.value++; hasAccessToken.value = false } },
  } as unknown as AppSession
  const support = useHomeSupportData({ session, listSupportTeams: async () => [],
    getSupportProfile: () => { const request = deferred<SupportProfile>(); reads.push(request); return request.promise },
    castSupportVote: (matchId, payload) => {
      const request = deferred<SupportMatchDetail>()
      writes.push({ matchId, teamId: payload.supported_team_id, request })
      return request.promise
    },
  })
  support.supportProfile.value = profile()
  return { support, revision, hasAccessToken, writes, reads }
}

test('首页一键助力传递当前比赛和球队，使用 POST 结果更新票数与已助力状态', async () => {
  const { support, writes } = fixture()
  const submit = support.submitSupportVote(2)
  expect(writes[0]).toMatchObject({ matchId: 10, teamId: 2 })
  expect(support.voteSubmittingTeamId.value).toBe(2)
  writes[0].request.resolve(voted())
  expect(await submit).toBe(true)
  expect(support.supportProfile.value?.next_match?.away_team.support_count).toBe(1)
  expect(support.supportProfile.value?.next_match?.viewer.has_supported).toBe(true)
  expect(support.voteSubmittingTeamId.value).toBeNull()
})

test('双击或在提交中点击另一队，只发一次 POST', async () => {
  const { support, writes } = fixture()
  const submit = support.submitSupportVote(2)
  expect(await support.submitSupportVote(2)).toBe(false)
  expect(await support.submitSupportVote(1)).toBe(false)
  expect(writes.length).toBe(1)
  writes[0].request.resolve(voted())
  await submit
})

test('助力失败不增加票数，按钮解锁后可以重试', async () => {
  const { support, writes } = fixture()
  const submit = support.submitSupportVote(2)
  writes[0].request.reject(new Error('网络失败'))
  await expectFailure(submit, '网络失败')
  expect(support.supportProfile.value?.next_match?.total_support_count).toBe(0)
  expect(support.voteSubmittingTeamId.value).toBeNull()
  const retry = support.submitSupportVote(2)
  writes[1].request.resolve(voted())
  expect(await retry).toBe(true)
})

test('未登录、已助力、比赛关闭或球队不属于本场时，不发 POST', async () => {
  const { support, hasAccessToken, writes } = fixture()
  hasAccessToken.value = false
  await expectFailure(support.submitSupportVote(2), '登录')
  hasAccessToken.value = true
  await expectFailure(support.submitSupportVote(99), '本场比赛')
  await expectFailure(support.submitSupportVote(1), '关注的主队')
  support.supportProfile.value!.next_match!.viewer.can_support = false
  await expectFailure(support.submitSupportVote(2), '暂时不能助力')
  support.supportProfile.value!.next_match!.viewer.can_support = true
  support.supportProfile.value!.next_match!.viewer.has_supported = true
  await expectFailure(support.submitSupportVote(2), '已经助力')
  support.supportProfile.value!.next_match!.viewer.has_supported = false
  support.supportProfile.value!.next_match!.kickoff_at = '2020-01-01T00:00:00Z'
  await expectFailure(support.submitSupportVote(2), '关闭')
  support.supportProfile.value!.next_match!.kickoff_at = 'invalid'
  await expectFailure(support.submitSupportVote(2), '关闭')
  expect(writes.length).toBe(0)
})

test('双方按钮按主队身份、提交状态、已助力和开球时刻判断可用性', () => {
  const match = profile().next_match!
  const now = Date.parse('2026-10-14T12:00:00Z')
  expect(resolveHomeSupportVoteState(match, 1, null, now).disabled).toBe(true)
  expect(resolveHomeSupportVoteState(match, 2, null, now)).toEqual({ label: '助力', disabled: false, supported: false })
  expect(resolveHomeSupportVoteState(match, 2, 2, now)).toEqual({ label: '提交中', disabled: true, supported: false })
  expect(resolveHomeSupportVoteState(match, 2, null, now, true).disabled).toBe(true)
  expect(resolveHomeSupportVoteState(voted(), 2, null, now)).toEqual({ label: '已助力', disabled: true, supported: true })
  expect(resolveHomeSupportVoteState(match, 2, null, Date.parse(match.kickoff_at))).toEqual({ label: '已结束', disabled: true, supported: false })
})

test('当前比赛已换场时，旧助力结果不覆盖新的比赛', async () => {
  const { support, writes } = fixture()
  const submit = support.submitSupportVote(2)
  support.supportProfile.value = { ...profile(), next_match: { ...profile().next_match!, match_id: 20 } }
  writes[0].request.resolve(voted())
  expect(await submit).toBe(false)
  expect(support.supportProfile.value.next_match?.match_id).toBe(20)
  expect(support.voteSubmittingTeamId.value).toBeNull()
})

test('换号后旧助力成功与失败均不发布到新账号，也不清新账号的提交状态', async () => {
  for (const fail of [false, true]) {
    const { support, revision, writes } = fixture()
    const oldSubmit = support.submitSupportVote(2)
    revision.value++
    support.resetSessionScopedState()
    support.supportProfile.value = profile()
    const newSubmit = support.submitSupportVote(2)
    if (fail) writes[0].request.reject(new Error('旧账号失败'))
    else writes[0].request.resolve(voted())
    expect(await oldSubmit).toBe(false)
    expect(support.voteSubmittingTeamId.value).toBe(2)
    expect(support.supportProfile.value?.next_match?.total_support_count).toBe(0)
    writes[1].request.resolve(voted())
    expect(await newSubmit).toBe(true)
  }
})

test('助力前在途的旧 profile 迟到时，不覆盖助力成功后的票数', async () => {
  const { support, reads, writes } = fixture()
  const staleRead = support.loadSupportData()
  await Promise.resolve()
  const submit = support.submitSupportVote(2)
  writes[0].request.resolve(voted())
  await submit
  reads[0].resolve(profile())
  await staleRead
  expect(support.supportProfile.value?.next_match?.total_support_count).toBe(1)
})
