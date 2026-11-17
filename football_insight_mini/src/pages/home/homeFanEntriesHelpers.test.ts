import { describe, expect, test } from 'bun:test'
import { nextTick, ref } from 'vue'
import type { LineupConsensus } from '../../types/lineup'
import type { SupportProfile } from '../../types/support'
import type { TransferCard, TransferIntent, TransferSummary } from '../../types/transfer'
import {
  buildHomeLineupEntryViewModel,
  buildHomeTransferEntryViewModel,
  DEFAULT_FAN_ENTRY_TEAM_ID,
  DEFAULT_FAN_ENTRY_TEAM_NAME,
  resolveHomeFanTeam,
} from './homeFanEntriesHelpers'
import { useHomeFanEntries } from './useHomeFanEntries'

function createTransferCard(id: number, name: string, intent: TransferIntent, up: number): TransferCard {
  return {
    share_code: `CODE${id}`,
    intent,
    returning: false,
    up,
    down: 0,
    my_vote: 0,
    rank: 1,
    created_at: '2026-10-21T11:00:00Z',
    player: {
      player_id: id,
      name,
      avatar_url: `https://example.com/${id}.jpg`,
      position: 'FW',
      shirt_number: 9,
      current_team: null,
      departed: false,
    },
  }
}

function createConsensus(overrides: Partial<LineupConsensus> = {}): LineupConsensus {
  return {
    team: { team_id: 77680, team_name: '成都蓉城', avatar_url: null },
    total: 10,
    threshold: 10,
    unlocked: true,
    formation: '4-2-3-1',
    formation_share: 30,
    players: [],
    ...overrides,
  }
}

function createSummary(boards: TransferSummary['boards']): TransferSummary {
  return {
    team: { team_id: 77680, team_name: '成都蓉城', avatar_url: null },
    window: { code: '2026-winter', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
    can_vote: true,
    boards,
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

describe('resolveHomeFanTeam', () => {
  test('uses default team immediately for guests and waits for profile when logged in', () => {
    expect(resolveHomeFanTeam({ hasAuthToken: false, supportLoading: true, profile: null })).toEqual({
      teamId: DEFAULT_FAN_ENTRY_TEAM_ID,
      teamName: DEFAULT_FAN_ENTRY_TEAM_NAME,
      ready: true,
    })

    expect(resolveHomeFanTeam({ hasAuthToken: true, supportLoading: true, profile: null })).toEqual({
      teamId: DEFAULT_FAN_ENTRY_TEAM_ID,
      teamName: DEFAULT_FAN_ENTRY_TEAM_NAME,
      ready: false,
    })

    const profile = {
      favorite_team: { team_id: 101, team_name: '上海申花', avatar_storage_url: null, rank_no: 2 },
      next_match: null,
    } as SupportProfile
    expect(resolveHomeFanTeam({ hasAuthToken: true, supportLoading: false, profile })).toEqual({
      teamId: 101,
      teamName: '上海申花',
      ready: true,
    })
  })
})

describe('buildHomeLineupEntryViewModel', () => {
  test('builds 11 formation dots and dynamic copy from consensus state', () => {
    const fallback = buildHomeLineupEntryViewModel(null, '成都蓉城')
    expect(fallback.pitchDots.length).toBe(11)
    expect(fallback.subtitle).toBe('排一套成都蓉城最佳 11 人')

    const progress = buildHomeLineupEntryViewModel(createConsensus({ unlocked: false, total: 4, formation: null }), '成都蓉城')
    expect(progress.pitchDots.length).toBe(11)
    expect(progress.subtitle).toBe('已有 4 套 · 汇聚最佳阵')

    const unlocked = buildHomeLineupEntryViewModel(createConsensus(), '成都蓉城')
    expect(unlocked.pitchDots.length).toBe(11)
    expect(unlocked.subtitle).toBe('已有 10 套 · 热门 4-2-3-1')
  })
})

describe('buildHomeTransferEntryViewModel', () => {
  test('prioritizes stay players, dedupes avatars up to 3, and highlights lead like count', () => {
    const empty = buildHomeTransferEntryViewModel(null, '成都蓉城')
    expect(empty.avatars).toEqual([])
    expect(empty.likeBadgeText).toBe('热赞')
    expect(empty.subtitle).toBe('留谁走谁 · 给成都蓉城球员点赞')

    const summary = createSummary([
      {
        intent: 'out',
        total: 11,
        items: [
          createTransferCard(900, '教练A', 'out', 28),
          createTransferCard(204, '费利佩', 'out', 3),
        ],
      },
      {
        intent: 'stay',
        total: 3,
        items: [
          createTransferCard(204, '费利佩', 'stay', 25),
          createTransferCard(184, '韦世豪', 'stay', 18),
        ],
      },
      { intent: 'in', total: 0, items: [] },
    ])

    const vm = buildHomeTransferEntryViewModel(summary, '成都蓉城')
    expect(vm.title).toBe('球员点赞')
    expect(vm.likeBadgeText).toBe('25')
    expect(vm.subtitle).toBe('费利佩等 14 人热议中')
    expect(vm.avatars.map(a => a.name)).toEqual(['费利佩', '韦世豪', '教练A'])
  })
})

describe('useHomeFanEntries', () => {
  test('drops stale responses when team or session revision changes', async () => {
    const hasAccessToken = ref(false)
    const revision = ref(1)
    const supportProfile = ref<SupportProfile | null>(null)
    const supportLoading = ref(false)

    const firstConsensus = deferred<LineupConsensus>()
    const firstSummary = deferred<TransferSummary>()
    const secondConsensus = deferred<LineupConsensus>()
    const secondSummary = deferred<TransferSummary>()

    let call = 0
    const entries = useHomeFanEntries({
      session: { hasAccessToken, revision },
      supportProfile,
      supportLoading,
      deps: {
        getLineupConsensus: () => (call === 1 ? firstConsensus.promise : secondConsensus.promise),
        getTransferSummary: () => {
          const currentCall = call
          return currentCall === 1 ? firstSummary.promise : secondSummary.promise
        },
      },
    })

    call = 1
    const firstLoad = entries.refresh()

    // Simulate user login resolving favorite team 101 before team 77680 finished.
    call = 2
    revision.value = 2
    hasAccessToken.value = true
    supportProfile.value = {
      favorite_team: { team_id: 101, team_name: '上海申花', avatar_storage_url: null, rank_no: 2 },
      next_match: null,
    } as SupportProfile
    await nextTick()

    firstConsensus.resolve(createConsensus({ team: { team_id: 77680, team_name: '成都蓉城', avatar_url: null }, total: 10 }))
    firstSummary.resolve(createSummary([{ intent: 'stay', total: 1, items: [createTransferCard(1, '旧球员', 'stay', 99)] }]))
    await firstLoad

    // Stale 77680 data must not be published after switching to team 101.
    expect(entries.consensus.value).toBeNull()
    expect(entries.transferSummary.value).toBeNull()

    secondConsensus.resolve(createConsensus({ team: { team_id: 101, team_name: '上海申花', avatar_url: null }, total: 5, unlocked: false }))
    secondSummary.resolve(createSummary([{ intent: 'stay', total: 2, items: [createTransferCard(2, '新球员', 'stay', 12)] }]))
    await nextTick()
    await Promise.resolve()
    await Promise.resolve()

    expect(entries.consensus.value?.team.team_id).toBe(101)
    expect(entries.transferSummary.value?.boards[0].items[0].player.name).toBe('新球员')
  })
})
