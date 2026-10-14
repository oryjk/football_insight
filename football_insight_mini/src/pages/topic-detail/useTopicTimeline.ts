import { computed, onScopeDispose, reactive, ref, watch, type Ref } from 'vue'
import { getTopicTimeline } from '../../api/dailyTopics'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { extractApiErrorMessage } from '../../utils/apiError'
import { mapTopicTimeline, mergeTopicHistory } from '../../utils/dailyTopics'

interface TeamTimeline {
  items: DailyTopicItem[]
  nextCursor: string | null
  loading: boolean
  loaded: boolean
  errorMessage: string
  failedReset: boolean
}

export function useTopicTimeline(anchor: Ref<DailyTopicItem | null>) {
  const selectedTeamId = ref<number | null>(null)
  // Page-local cache: loaded pages and in-flight requests belong to their own team.
  const teams = reactive(new Map<number, TeamTimeline>())
  const selected = computed(() => selectedTeamId.value ? teams.get(selectedTeamId.value) : undefined)
  const items = computed(() => selected.value?.items ?? [])
  const nextCursor = computed(() => selected.value?.nextCursor ?? null)
  const loading = computed(() => selected.value?.loading ?? false)
  const errorMessage = computed(() => selected.value?.errorMessage ?? '')
  const tags = computed(() => anchor.value?.teamTags ?? [])
  let revision = 0

  async function load(reset: boolean) {
    const currentAnchor = anchor.value, teamId = selectedTeamId.value
    if (!currentAnchor || !teamId || !tags.value.some(tag => tag.teamId === teamId)) return
    const state = teams.get(teamId)
    if (!state || state.loading || (!reset && !state.nextCursor)) return
    const current = revision, cursor = reset ? null : state.nextCursor
    state.loading = true
    state.errorMessage = ''
    state.failedReset = reset
    try {
      const response = await getTopicTimeline(currentAnchor.teamId, currentAnchor.id, teamId, cursor)
      if (current !== revision) return
      const page = mapTopicTimeline(response, currentAnchor, teamId)
      if (page.nextCursor && page.nextCursor === cursor) throw new Error('话题追踪加载失败，请重试')
      state.items = mergeTopicHistory(reset ? [] : state.items, page.items)
      state.nextCursor = page.nextCursor
      state.loaded = true
    } catch (error) {
      if (current === revision) state.errorMessage = extractApiErrorMessage(error, '话题追踪加载失败，请稍后重试')
    } finally { if (current === revision) state.loading = false }
  }

  function selectTeam(teamId: number) {
    if (!tags.value.some(tag => tag.teamId === teamId) || selectedTeamId.value === teamId) return
    if (!teams.has(teamId)) teams.set(teamId, {
      items: [], nextCursor: null, loading: false, loaded: false, errorMessage: '', failedReset: true,
    })
    selectedTeamId.value = teamId
    if (!teams.get(teamId)!.loaded) void load(true)
  }

  watch(anchor, () => {
    revision += 1
    teams.clear()
    selectedTeamId.value = null
    if (tags.value[0]) selectTeam(tags.value[0].teamId)
  }, { immediate: true, flush: 'sync' })
  onScopeDispose(() => { revision += 1 })
  return { tags, selectedTeamId, items, nextCursor, loading, errorMessage, selectTeam,
    loadMore: () => load(false), retry: () => load(selected.value?.failedReset ?? true) }
}
