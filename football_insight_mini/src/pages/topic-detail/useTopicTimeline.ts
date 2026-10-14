import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'
import { getTopicTimeline } from '../../api/dailyTopics'
import type { DailyTopicItem } from '../../types/dailyTopic'
import { extractApiErrorMessage } from '../../utils/apiError'
import { mapTopicTimeline, mergeTopicHistory } from '../../utils/dailyTopics'

export function useTopicTimeline(anchor: Ref<DailyTopicItem | null>) {
  const selectedTeamId = ref<number | null>(null)
  const items = ref<DailyTopicItem[]>([])
  const nextCursor = ref<string | null>(null)
  const loading = ref(false)
  const errorMessage = ref('')
  const tags = computed(() => anchor.value?.teamTags ?? [])
  let revision = 0
  let failedReset = true

  async function load(reset: boolean) {
    const currentAnchor = anchor.value, teamId = selectedTeamId.value
    if (!currentAnchor || !teamId || !tags.value.some(tag => tag.teamId === teamId)) return
    if (!reset && (loading.value || !nextCursor.value)) return
    const current = ++revision, cursor = reset ? null : nextCursor.value
    loading.value = true
    errorMessage.value = ''
    failedReset = reset
    if (reset) { items.value = []; nextCursor.value = null }
    try {
      const response = await getTopicTimeline(currentAnchor.teamId, currentAnchor.id, teamId, cursor)
      if (current !== revision) return
      const page = mapTopicTimeline(response, currentAnchor, teamId)
      if (page.nextCursor && page.nextCursor === cursor) throw new Error('话题追踪加载失败，请重试')
      items.value = mergeTopicHistory(reset ? [] : items.value, page.items)
      nextCursor.value = page.nextCursor
    } catch (error) {
      if (current === revision) errorMessage.value = extractApiErrorMessage(error, '话题追踪加载失败，请稍后重试')
    } finally { if (current === revision) loading.value = false }
  }

  function selectTeam(teamId: number) {
    if (!tags.value.some(tag => tag.teamId === teamId) || selectedTeamId.value === teamId) return
    selectedTeamId.value = teamId
    void load(true)
  }

  watch(anchor, () => {
    revision += 1
    selectedTeamId.value = null
    items.value = []; nextCursor.value = null; loading.value = false; errorMessage.value = ''
    if (tags.value[0]) selectTeam(tags.value[0].teamId)
  })
  onScopeDispose(() => { revision += 1 })
  return { tags, selectedTeamId, items, nextCursor, loading, errorMessage, selectTeam,
    loadMore: () => load(false), retry: () => load(failedReset) }
}
