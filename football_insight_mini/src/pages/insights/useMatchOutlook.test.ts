import { expect, test } from 'bun:test'
import { ref } from 'vue'
import { createOutlookLoader, loadOutlookFixtures } from './useMatchOutlook'
const metadata = {
  teams: [
    { team_id: 9, team_name: '成都蓉城', rank_no: 1, avatar_storage_url: null },
    { team_id: 1, team_name: '北京国安', rank_no: 2, avatar_storage_url: null },
  ],
  matches: [
    {
      match_id: 1,
      home_team_id: 9,
      away_team_id: 1,
      status: 'scheduled',
      match_date: '2099-01-01',
      match_time: '19:35',
    },
  ],
  season: 2026,
  favorite: 9,
  completedMatches: 221,
}
function setup() {
  const authenticated = ref(false),
    review = ref(false),
    revision = ref(0),
    selectedTeamId = ref<number | null>(null),
    calls: number[] = []
  const loader = createOutlookLoader({
    authenticated: () => authenticated.value,
    underReview: () => review.value,
    revision,
    selectedTeamId,
    metadata: async () => metadata as any,
    outlook: async (id: number) => {
      calls.push(id)
      return { status: 'unavailable', outlook: null }
    },
    record: async () => ({
      season: 2026,
      matches: 0,
      hits: 0,
      accuracy: null,
      rps_model: null,
      rps_baseline: null,
      rps_improvement: null,
    }),
  })
  return { loader, authenticated, review, revision, selectedTeamId, calls }
}
test('guest/review never loads and logged-in low-tier access is independent of membership', async () => {
  const t = setup()
  await t.loader.load()
  expect(t.calls.length).toBe(0)
  t.review.value = true
  t.authenticated.value = true
  await t.loader.load()
  expect(t.calls.length).toBe(0)
  t.review.value = false
  await t.loader.load()
  expect(t.selectedTeamId.value).toBe(9)
  expect(t.calls).toEqual([1])
  expect(t.loader.completedMatches.value).toBe(221) // not the zero public scored count
  t.loader.dispose()
})
test('entering review or logging out clears results and late replies cannot restore them', async () => {
  const t = setup()
  t.authenticated.value = true
  await t.loader.load()
  t.review.value = true
  expect(t.loader.teams.value).toEqual([])
  expect(t.loader.record.value).toBe(null)
  t.loader.dispose()
})

const emptyRecord = {
  season: 2026,
  matches: 0,
  hits: 0,
  accuracy: null,
  rps_model: null,
  rps_baseline: null,
  rps_improvement: null,
}
const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0))
test('auth intent blocks private favorite reads until a replacement user is published', async () => {
  const revision = ref(0),
    identity = ref('A'),
    selectedTeamId = ref<number | null>(null)
  let calls = 0
  const loader = createOutlookLoader({
    authenticated: () => true,
    underReview: () => false,
    revision,
    selectedTeamId,
    identity: () => identity.value,
    metadata: async () => {
      calls++
      return metadata as any
    },
    outlook: async () => ({ status: 'unavailable', outlook: null }),
    record: async () => emptyRecord,
  })
  await loader.load()
  revision.value++
  await flush()
  await loader.load()
  expect(calls).toBe(1)
  identity.value = 'B'
  await flush()
  expect(calls).toBe(2)
  loader.dispose()
})
test('fixtures include future rounds even when the live round is completed', async () => {
  const rounds: number[] = []
  const result = await loadOutlookFixtures(
    2026,
    async () =>
      [
        { season: 2026, round_number: 28, status: 'completed' },
        { season: 2026, round_number: 29, status: 'upcoming' },
        { season: 2026, round_number: 30, status: 'upcoming' },
      ] as any,
    async (request) => {
      rounds.push(request.roundNumber!)
      return {
        current_season: 2026,
        matches: [{ ...metadata.matches[0], match_id: request.roundNumber }],
      } as any
    },
  )
  expect(rounds).toEqual([29, 30])
  expect(result.map((m) => m.match_id)).toEqual([29, 30])
})
test('switching teams during the first forecast drops the old result and preserves the choice on return', async () => {
  const review = ref(false),
    selectedTeamId = ref<number | null>(null),
    revision = ref(0),
    calls: number[] = []
  let resolveFirst!: (value: any) => void
  const first = new Promise<any>((resolve) => {
    resolveFirst = resolve
  })
  const data = {
    ...metadata,
    matches: [
      { ...metadata.matches[0], away_team_id: 99 },
      {
        ...metadata.matches[0],
        match_id: 2,
        home_team_id: 1,
        away_team_id: 98,
      },
    ],
  }
  const loader = createOutlookLoader({
    authenticated: () => true,
    underReview: () => review.value,
    revision,
    selectedTeamId,
    metadata: async () => data as any,
    outlook: async (id) => {
      calls.push(id)
      return id === 1
        ? first
        : { status: 'ready', outlook: { match_id: id } as any }
    },
    record: async () => emptyRecord,
  })
  const pending = loader.load()
  await flush()
  loader.selectTeam(1)
  await flush()
  expect(loader.outlook.value?.match_id).toBe(2)
  expect(loader.loading.value).toBe(false)
  resolveFirst({ status: 'ready', outlook: { match_id: 1 } })
  await pending
  await flush()
  expect(calls).toEqual([1, 2])
  expect(loader.selectedMatchId.value).toBe(2)
  expect(loader.outlook.value?.match_id).toBe(2)
  review.value = true
  review.value = false
  await loader.load()
  expect(selectedTeamId.value).toBe(1)
  revision.value++
  await loader.load()
  expect(selectedTeamId.value).toBe(9)
  loader.dispose()
})
test('a team chosen before metadata arrives is honored, including clubs without a standing', async () => {
  const selectedTeamId = ref<number | null>(null)
  let resolveData!: (value: any) => void
  const loader = createOutlookLoader({
    authenticated: () => true,
    underReview: () => false,
    revision: ref(0),
    selectedTeamId,
    metadata: () =>
      new Promise((resolve) => {
        resolveData = resolve
      }),
    outlook: async () => ({ status: 'unavailable', outlook: null }),
    record: async () => emptyRecord,
  })
  const pending = loader.load()
  loader.selectTeam(1)
  resolveData({
    ...metadata,
    teams: metadata.teams.map((t) => ({ ...t, rank_no: null })),
  })
  await pending
  expect(selectedTeamId.value).toBe(1)
  expect(loader.teams.value.length).toBe(2)
  loader.dispose()
})
