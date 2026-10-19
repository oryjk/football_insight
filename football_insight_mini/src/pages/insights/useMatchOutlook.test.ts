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
  t.loader.dispose()
})
test('entering review or logging out clears results and late replies cannot restore them', async () => {
  const t = setup()
  t.authenticated.value = true
  await t.loader.load()
  t.review.value = true
  expect(t.loader.teams.value).toEqual([])
  t.loader.dispose()
})

const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0))
test('changing teams after kickoff excludes the now-started fixture', async () => {
  let now = Date.parse('2099-01-01T11:00:00Z')
  const loader = createOutlookLoader({
    authenticated: () => true, underReview: () => false, revision: ref(0), selectedTeamId: ref(null),
    now: () => now,
    metadata: async () => metadata as any,
    outlook: async () => ({status: 'unavailable', outlook: null}),
  })
  await loader.load()
  expect(loader.matches.value.length).toBe(1)
  now = Date.parse('2099-01-01T12:00:00Z')
  loader.selectTeam(1)
  expect(loader.matches.value).toEqual([])
  expect(loader.selectedMatchId.value).toBe(null)
  loader.dispose()
})
test('switching the shared fixture between teams uses the cache without a loading frame', async () => {
  const t = setup()
  t.authenticated.value = true
  await t.loader.load()
  t.loader.selectTeam(1)
  expect(t.loader.loading.value).toBe(false)
  expect(t.calls).toEqual([1])
  t.loader.selectTeam(9)
  expect(t.loader.loading.value).toBe(false)
  expect(t.calls).toEqual([1])
  t.loader.dispose()
})

test('prefetched next fixtures share in-flight requests and publish without reloading metadata', async () => {
  const selectedTeamId = ref<number | null>(null)
  const calls: number[] = []
  let metadataCalls = 0
  let resolveNext!: (value: any) => void
  const next = new Promise<any>((resolve) => {
    resolveNext = resolve
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
    underReview: () => false,
    revision: ref(0),
    selectedTeamId,
    metadata: async () => {
      metadataCalls++
      return data as any
    },
    outlook: async (id) => {
      calls.push(id)
      return id === 2
        ? next
        : { status: 'ready', outlook: { match_id: 1 } as any }
    },
  })
  await loader.load()
  await flush()
  expect(calls).toEqual([1, 2])
  expect(loader.loading.value).toBe(false)
  loader.selectTeam(1)
  expect(loader.outlook.value).toBe(null) // never show another team's prediction
  expect(calls).toEqual([1, 2])
  resolveNext({ status: 'ready', outlook: { match_id: 2 } })
  await flush()
  expect(loader.outlook.value?.match_id).toBe(2)
  loader.selectTeam(9)
  expect(loader.outlook.value?.match_id).toBe(1)
  expect(loader.loading.value).toBe(false)
  expect(metadataCalls).toBe(1)
  loader.dispose()
})

test('expired or previous-session forecast cache is not reused', async () => {
  const review = ref(false),
    selectedTeamId = ref<number | null>(null)
  let now = 0,
    calls = 0
  const loader = createOutlookLoader({
    authenticated: () => true,
    underReview: () => review.value,
    revision: ref(0),
    selectedTeamId,
    now: () => now,
    metadata: async () => metadata as any,
    outlook: async () => {
      calls++
      return { status: 'unavailable', outlook: null }
    },
  })
  await loader.load()
  now = 59_999
  loader.selectTeam(1)
  await flush()
  expect(calls).toBe(1)
  now = 60_000
  loader.selectTeam(9)
  await flush()
  expect(calls).toBe(2)
  review.value = true
  review.value = false
  await loader.load()
  expect(calls).toBe(3)
  loader.dispose()
})
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

test('external selection survives slow metadata even when the chosen public team is absent from private metadata', async () => {
  const selectedTeamId = ref<number | null>(9)
  let resolveData!: (value: any) => void
  const loader = createOutlookLoader({
    authenticated: () => true, underReview: () => false, revision: ref(0), selectedTeamId,
    externalTeamSelection: true,
    metadata: () => new Promise(resolve => { resolveData = resolve }),
    outlook: async () => ({ status: 'unavailable', outlook: null }),
  })
  const pending = loader.load()
  selectedTeamId.value = 3
  resolveData(metadata)
  await pending
  expect(selectedTeamId.value).toBe(3)
  expect(loader.matches.value).toEqual([])
  expect(loader.selectedMatchId.value).toBe(null)
  expect(loader.loading.value).toBe(false)
  loader.dispose()
  expect(selectedTeamId.value).toBe(3)
})

test('external null selection waits for the public chooser and private resets preserve its later choice', async () => {
  const selectedTeamId = ref<number | null>(null), review = ref(false), authenticated = ref(true)
  const calls: number[] = []
  const loader = createOutlookLoader({
    authenticated: () => authenticated.value, underReview: () => review.value,
    revision: ref(0), selectedTeamId, externalTeamSelection: true,
    metadata: async () => metadata as any,
    outlook: async id => { calls.push(id); return { status: 'unavailable', outlook: null } },
  })
  await loader.load()
  expect(selectedTeamId.value).toBe(null); expect(loader.selectedMatchId.value).toBe(null)
  selectedTeamId.value = 1; await flush()
  expect(loader.selectedMatchId.value).toBe(1)
  review.value = true
  expect(loader.teams.value).toEqual([]); expect(selectedTeamId.value).toBe(1)
  review.value = false; await loader.load()
  expect(selectedTeamId.value).toBe(1)
  authenticated.value = false
  expect(loader.teams.value).toEqual([]); expect(selectedTeamId.value).toBe(1)
  loader.dispose()
})

test('external choice and new loading survive a previous-account late metadata response', async () => {
  const selectedTeamId = ref<number | null>(1), revision = ref(0), identity = ref('A')
  let resolveOld!: (value: any) => void, resolveCurrent!: (value: any) => void, calls = 0
  const loader = createOutlookLoader({
    authenticated: () => true, underReview: () => false, identity: () => identity.value,
    revision, selectedTeamId, externalTeamSelection: true,
    metadata: () => new Promise(resolve => { if (++calls === 1) resolveOld = resolve; else resolveCurrent = resolve }),
    outlook: async () => ({ status: 'unavailable', outlook: null }),
  })
  const old = loader.load()
  revision.value++; identity.value = 'B'
  const current = loader.load()
  resolveOld({ ...metadata, favorite: 9 }); await old
  expect(loader.teams.value).toEqual([]); expect(loader.loading.value).toBe(true)
  expect(selectedTeamId.value).toBe(1)
  resolveCurrent({ ...metadata, favorite: 9 }); await current
  expect(loader.teams.value.length).toBe(2); expect(loader.loading.value).toBe(false)
  expect(selectedTeamId.value).toBe(1)
  loader.dispose()
})

// 回归证据：outlookLoading 只在 reset() 里被强制清零，而 reset() 由 underReview/authenticated/revision 的变化触发。
// 页面每次 onShow 把 outlookReviewReady 打回 false 再置 true，正是让 underReview 抖动一次、从而 reset() 一次的原因；
// 若把它改成"保持 true"，一次被打断的加载就没有任何路径把 forecastLoading 清掉。
test('onShow 让 underReview 抖动一次：在途的预测加载被 reset 清掉，outlookLoading 不会一直为 true', async () => {
  const authenticated = ref(true),
    review = ref(false),
    revision = ref(0),
    selectedTeamId = ref<number | null>(null)
  let release: () => void = () => {}
  const loader = createOutlookLoader({
    authenticated: () => authenticated.value,
    underReview: () => review.value,
    revision,
    selectedTeamId,
    metadata: async () => metadata as any,
    outlook: () => new Promise((resolve) => { release = () => resolve({ status: 'unavailable', outlook: null } as any) }),
  })
  void loader.load()
  await flush()
  await flush()
  expect(loader.loading.value).toBe(true) // 预测请求在途

  // 页面 onShow：outlookReviewReady false → true，即 underReview 抖动一次
  review.value = true
  review.value = false
  await flush()
  await flush()
  // reset() 已清掉旧的加载态；随后会重新加载，新的在途请求是新的一轮而不是被卡住的旧状态
  release()
  await flush()
  await flush()
  expect(loader.loading.value).toBe(false)
  loader.dispose()
})

// 对照：underReview 全程不变（4131b32 的行为）时，没有任何 reset() 会清理"被打断的加载"。
// 这里让预测请求永远不返回（网络在 1–3 秒的慢链路上被打断的极端情形），确认 outlookLoading 由此一直为 true。
test('对照：underReview 不抖动且预测请求一直不返回时，outlookLoading 一直为 true（没有任何路径清掉它）', async () => {
  const authenticated = ref(true),
    review = ref(false),
    revision = ref(0),
    selectedTeamId = ref<number | null>(null)
  const loader = createOutlookLoader({
    authenticated: () => authenticated.value,
    underReview: () => review.value,
    revision,
    selectedTeamId,
    metadata: async () => metadata as any,
    outlook: () => new Promise(() => {}), // 永不返回
  })
  void loader.load()
  await flush()
  await flush()
  expect(loader.loading.value).toBe(true)
  // 模拟回到页面却不重置 underReview：什么都不触发
  await flush()
  expect(loader.loading.value).toBe(true)
  loader.dispose()
})
