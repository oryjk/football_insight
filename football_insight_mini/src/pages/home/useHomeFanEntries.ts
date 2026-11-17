import { computed, ref, watch, type Ref } from 'vue'
import { getLineupConsensus } from '../../api/lineup'
import { getTransferSummary } from '../../api/transfer'
import type { AppSession } from '../../stores/appSession'
import type { LineupConsensus } from '../../types/lineup'
import type { SupportProfile } from '../../types/support'
import type { TransferSummary } from '../../types/transfer'
import { createLatestGuard } from '../../utils/latestGuard'
import { resolveHomeFanTeam } from './homeFanEntriesHelpers'

export interface UseHomeFanEntriesOptions {
  session: Pick<AppSession, 'hasAccessToken' | 'revision'>
  supportProfile: Readonly<Ref<SupportProfile | null>>
  supportLoading: Readonly<Ref<boolean>>
  deps?: {
    getLineupConsensus?: typeof getLineupConsensus
    getTransferSummary?: typeof getTransferSummary
  }
}

export function useHomeFanEntries(options: UseHomeFanEntriesOptions) {
  const fetchConsensus = options.deps?.getLineupConsensus ?? getLineupConsensus
  const fetchTransferSummary = options.deps?.getTransferSummary ?? getTransferSummary

  const consensus = ref<LineupConsensus | null>(null)
  const transferSummary = ref<TransferSummary | null>(null)

  const resolvedTeam = computed(() =>
    resolveHomeFanTeam({
      hasAuthToken: options.session.hasAccessToken.value,
      supportLoading: options.supportLoading.value,
      profile: options.supportProfile.value,
    }),
  )

  const guard = createLatestGuard(() => [
    options.session.revision.value,
    resolvedTeam.value.teamId,
  ])

  let loadedTeamId: number | null = null

  async function refresh(): Promise<void> {
    if (!resolvedTeam.value.ready) return

    const teamId = resolvedTeam.value.teamId
    if (loadedTeamId !== null && loadedTeamId !== teamId) {
      consensus.value = null
      transferSummary.value = null
    }
    loadedTeamId = teamId

    const token = guard.begin()
    const [consensusResult, transferResult] = await Promise.allSettled([
      fetchConsensus(teamId, { silentNetworkError: true }),
      fetchTransferSummary(teamId, { silentNetworkError: true }),
    ])
    if (!token.isCurrent()) return

    if (consensusResult.status === 'fulfilled') {
      consensus.value = consensusResult.value
    }
    if (transferResult.status === 'fulfilled') {
      transferSummary.value = transferResult.value
    }
  }

  watch(
    () => [
      options.session.revision.value,
      resolvedTeam.value.ready,
      resolvedTeam.value.teamId,
    ] as const,
    ([revision, ready, teamId], [prevRevision]) => {
      if (revision !== prevRevision) {
        guard.invalidate()
        consensus.value = null
        transferSummary.value = null
        loadedTeamId = null
      }
      if (ready) {
        void refresh()
      } else if (loadedTeamId !== null && loadedTeamId !== teamId) {
        consensus.value = null
        transferSummary.value = null
        loadedTeamId = null
      }
    },
  )

  return {
    resolvedTeam,
    consensus,
    transferSummary,
    refresh,
  }
}
