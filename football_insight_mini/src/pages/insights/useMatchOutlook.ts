import { ref, computed, watch, type Ref } from 'vue'
import { getAvailableRounds, getMatches } from '../../api/insight'
import { listSupportTeams, getSupportProfile } from '../../api/support'
import { getMatchOutlook, getOutlookRecord } from '../../api/matchOutlook'
import type { MatchCard } from '../../types/insight'
import type { SupportTeam } from '../../types/support'
import type {
  MatchOutlook,
  OutlookResponse,
  OutlookRecord,
} from '../../types/matchOutlook'

interface Metadata {
  teams: SupportTeam[]
  matches: MatchCard[]
  season: number
  favorite: number | null
  completedMatches?: number
}
interface Dependencies {
  authenticated: () => boolean
  identity?: () => unknown
  underReview: () => boolean
  revision: Readonly<Ref<number>>
  selectedTeamId: Ref<number | null>
  metadata?: () => Promise<Metadata>
  outlook?: (id: number) => Promise<OutlookResponse>
  record?: (year: number) => Promise<OutlookRecord>
}

export async function loadOutlookFixtures(
  season: number,
  rounds = getAvailableRounds,
  fetchMatches = getMatches,
): Promise<MatchCard[]> {
  const available = await rounds(season)
  const pending = available.filter((round) => round.status !== 'completed')
  const lists = await Promise.all(
    pending.map((round) =>
      fetchMatches({ mode: 'round', season, roundNumber: round.round_number }),
    ),
  )
  return [
    ...new Map(
      lists
        .flatMap((list) => list.matches)
        .map((match) => [match.match_id, match]),
    ).values(),
  ]
}

async function loadMetadata(): Promise<Metadata> {
  // /live/matches is one round, not the complete future fixture list.
  const [teams, live, profile] = await Promise.all([
    listSupportTeams(),
    getMatches({
      mode: 'live',
      season: new Date().getFullYear(),
      roundNumber: null,
    }),
    getSupportProfile().catch(() => null),
  ])
  const rounds = await getAvailableRounds(live.current_season)
  const matches = await loadOutlookFixtures(
    live.current_season,
    async () => rounds,
  )
  return {
    teams,
    matches,
    season: live.current_season,
    favorite: profile?.favorite_team?.team_id ?? null,
    completedMatches: rounds.reduce(
      (total, round) => total + round.completed_matches,
      0,
    ),
  }
}

export function createOutlookLoader(deps: Dependencies) {
  const teams = ref<SupportTeam[]>([])
  const allMatches = ref<MatchCard[]>([])
  const outlook = ref<MatchOutlook | null>(null)
  const record = ref<OutlookRecord | null>(null)
  const completedMatches = ref<number | null>(null)
  const selectedMatchId = ref<number | null>(null)
  const metadataLoading = ref(false),
    forecastLoading = ref(false),
    error = ref('')
  const loading = computed(() => metadataLoading.value || forecastLoading.value)
  const matches = computed(() =>
    allMatches.value
      .filter(
        (match) =>
          match.status === 'scheduled' &&
          Date.parse(`${match.match_date}T${match.match_time}+08:00`) >
            Date.now() &&
          (match.home_team_id === deps.selectedTeamId.value ||
            match.away_team_id === deps.selectedTeamId.value),
      )
      .sort((a, b) =>
        `${a.match_date} ${a.match_time}`.localeCompare(
          `${b.match_date} ${b.match_time}`,
        ),
      ),
  )
  let owner = 0,
    forecastOwner = 0,
    inFlight: Promise<void> | null = null
  let awaitingIdentity = false
  let choiceInitialized = false,
    applyingMetadata = false,
    disposed = false
  const allowed = () =>
    !disposed &&
    !awaitingIdentity &&
    deps.authenticated() &&
    !deps.underReview()

  function reset() {
    owner++
    forecastOwner++
    inFlight = null
    teams.value = []
    allMatches.value = []
    outlook.value = null
    record.value = null
    completedMatches.value = null
    selectedMatchId.value = null
    metadataLoading.value = false
    forecastLoading.value = false
    error.value = ''
  }
  async function loadForecast(id: number) {
    const request = ++forecastOwner
    outlook.value = null
    selectedMatchId.value = id
    error.value = ''
    forecastLoading.value = true
    try {
      const result = await (deps.outlook ?? getMatchOutlook)(id)
      if (allowed() && request === forecastOwner) outlook.value = result.outlook
    } catch {
      if (allowed() && request === forecastOwner)
        error.value = '赛前资料暂不可用，请稍后再试'
    } finally {
      if (request === forecastOwner) forecastLoading.value = false
    }
  }
  function chooseNext(): Promise<void> {
    forecastOwner++
    outlook.value = null
    forecastLoading.value = false
    error.value = ''
    selectedMatchId.value = matches.value[0]?.match_id ?? null
    return selectedMatchId.value !== null && allowed()
      ? loadForecast(selectedMatchId.value)
      : Promise.resolve()
  }
  function load(): Promise<void> {
    if (!allowed()) return Promise.resolve()
    if (inFlight) return inFlight
    const request = ++owner
    metadataLoading.value = true
    const work = (async () => {
      try {
        const data = await (deps.metadata ?? loadMetadata)()
        if (!allowed() || request !== owner) return
        teams.value = data.teams
        allMatches.value = data.matches
        completedMatches.value = data.completedMatches ?? null
        if (
          !choiceInitialized ||
          !data.teams.some((team) => team.team_id === deps.selectedTeamId.value)
        ) {
          applyingMetadata = true
          deps.selectedTeamId.value =
            data.teams.find((team) => team.team_id === data.favorite)
              ?.team_id ??
            data.teams.find((team) => team.team_name === '成都蓉城')?.team_id ??
            data.teams[0]?.team_id ??
            null
          applyingMetadata = false
        }
        choiceInitialized = true
        // A superseded forecast must not keep the newly selected team's spinner active.
        metadataLoading.value = false
        const recordWork = (deps.record ?? getOutlookRecord)(data.season)
          .then((result) => {
            if (allowed() && request === owner) record.value = result
          })
          .catch(() => {})
        await Promise.all([chooseNext(), recordWork])
      } catch {
        if (allowed() && request === owner)
          error.value = '赛前资料暂不可用，请稍后再试'
      } finally {
        if (request === owner) {
          metadataLoading.value = false
          inFlight = null
        }
      }
    })()
    inFlight = work
    return work
  }
  const stop = watch(
    [
      deps.authenticated,
      deps.underReview,
      deps.revision,
      deps.identity ?? deps.authenticated,
    ],
    (current, previous) => {
      if (
        deps.identity &&
        current[2] !== previous[2] &&
        current[3] === previous[3]
      )
        awaitingIdentity = true
      if (current[3] !== previous[3]) awaitingIdentity = false
      if (current[0] !== previous[0] || current[2] !== previous[2])
        choiceInitialized = false
      reset()
      const request = owner
      void Promise.resolve().then(() => {
        if (request === owner && allowed()) void load()
      })
    },
    { flush: 'sync' },
  )
  const stopTeam = watch(
    deps.selectedTeamId,
    () => {
      if (!applyingMetadata && allMatches.value.length) void chooseNext()
    },
    { flush: 'sync' },
  )
  function selectTeam(id: number) {
    choiceInitialized = true
    deps.selectedTeamId.value = id
  }
  function selectMatch(id: number) {
    if (matches.value.some((match) => match.match_id === id) && allowed())
      void loadForecast(id)
  }
  function dispose() {
    disposed = true
    stop()
    stopTeam()
    reset()
  }
  return {
    teams,
    matches,
    outlook,
    record,
    completedMatches,
    selectedMatchId,
    loading,
    error,
    load,
    selectTeam,
    selectMatch,
    dispose,
  }
}
