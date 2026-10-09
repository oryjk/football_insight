import { computed, ref } from 'vue'
import { getAvailableRounds, getMatches } from '../../api/insight'
import type { MatchCard, RoundReference } from '../../types/insight'
import { extractApiErrorMessage } from '../../utils/apiError'
import { hasMatchTechStats } from './helpers'
import { buildUpcomingSections, groupFinishedMatches, mergeMatchLists } from './presentation'

// The embedded content renders live results immediately and tolerates a failed
// future-round request. The retained standalone entry keeps its original
// all-or-nothing loading behavior.
export function useMatchesPage(
  { progressiveLoad = true }: { progressiveLoad?: boolean } = {},
  source = { getAvailableRounds, getMatches },
) {
  const season = new Date().getFullYear()
  const loading = ref(true)
  const errorMessage = ref('')
  const rounds = ref<RoundReference[]>([])
  const matches = ref<MatchCard[]>([])
  const pageNowIso = ref(new Date().toISOString())
  const roundDialogLoading = ref(false)
  const roundDialogErrorMessage = ref('')
  const selectedRoundNumber = ref<number | null>(null)
  const selectedRoundMatches = ref<MatchCard[]>([])
  const selectedTechStatsMatch = ref<MatchCard | null>(null)
  const groupedMatches = computed(() => groupFinishedMatches(matches.value, pageNowIso.value))
  const upcomingSections = computed(() => buildUpcomingSections(rounds.value, matches.value, pageNowIso.value))

  async function loadPage(): Promise<void> {
    loading.value = true
    errorMessage.value = ''
    pageNowIso.value = new Date().toISOString()
    try {
      const [roundsResponse, liveMatchesResponse] = await Promise.all([
        source.getAvailableRounds(season),
        source.getMatches({ mode: 'live', season, roundNumber: null }),
      ])
      rounds.value = roundsResponse
      matches.value = liveMatchesResponse.matches
      if (progressiveLoad) loading.value = false

      const futureRoundNumbers = roundsResponse
        .filter((round) => round.status === 'current' || round.status === 'upcoming')
        .map((round) => round.round_number)
        .slice(0, 2)
      const requests = futureRoundNumbers.map((roundNumber) => source.getMatches({ mode: 'round', season, roundNumber }))
      if (progressiveLoad) {
        const responses = await Promise.allSettled(requests)
        const futureLists: MatchCard[][] = []
        for (const response of responses) {
          if (response.status === 'fulfilled') futureLists.push(response.value.matches)
          else console.warn('[matches] failed to load future round matches', response.reason)
        }
        matches.value = mergeMatchLists(liveMatchesResponse.matches, ...futureLists)
      } else {
        const responses = await Promise.all(requests)
        matches.value = mergeMatchLists(liveMatchesResponse.matches, ...responses.map((response) => response.matches))
      }
    } catch (error) {
      errorMessage.value = extractApiErrorMessage(error, '赛程加载失败，请稍后重试。')
    } finally {
      loading.value = false
    }
  }

  async function openRoundDialog(roundNumber: number): Promise<void> {
    selectedRoundNumber.value = roundNumber
    roundDialogLoading.value = true
    roundDialogErrorMessage.value = ''
    try {
      const response = await source.getMatches({ mode: 'round', season, roundNumber })
      selectedRoundMatches.value = response.matches
    } catch (error) {
      roundDialogErrorMessage.value = extractApiErrorMessage(error, '这轮对阵暂时没有加载出来，请稍后重试。')
    } finally {
      roundDialogLoading.value = false
    }
  }

  function closeRoundDialog(): void {
    selectedRoundNumber.value = null
    selectedRoundMatches.value = []
    roundDialogErrorMessage.value = ''
  }

  function openMatchTechStats(match: MatchCard): void {
    if (!hasMatchTechStats(match)) {
      uni.showToast({ title: '这场比赛暂时还没有技术统计', icon: 'none' })
      return
    }
    selectedTechStatsMatch.value = match
  }

  function closeMatchTechStats(): void {
    selectedTechStatsMatch.value = null
  }

  return {
    loading, errorMessage, rounds, matches, pageNowIso, groupedMatches, upcomingSections,
    roundDialogLoading, roundDialogErrorMessage, selectedRoundNumber, selectedRoundMatches,
    selectedTechStatsMatch, loadPage, openRoundDialog, closeRoundDialog,
    openMatchTechStats, closeMatchTechStats,
  }
}
