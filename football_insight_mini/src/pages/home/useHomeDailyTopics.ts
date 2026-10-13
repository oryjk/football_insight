import { computed, onScopeDispose, ref } from 'vue'
import { getDailyTopics } from '../../api/dailyTopics'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { DAILY_TOPICS_TEAM_ID, emptyDailyTopicEdition, mapDailyTopicEdition } from './dailyTopics'

export function useHomeDailyTopics() {
  const edition = ref(emptyDailyTopicEdition())
  const loading = ref(false)
  const errorMessage = ref('')
  const selectedId = ref<string | null>(null)
  const selectedItem = computed(() => edition.value.items.find(item => item.id === selectedId.value) ?? null)
  let revision = 0
  async function load() {
    const current = ++revision
    loading.value = true
    errorMessage.value = ''
    try {
      const payload = await getDailyTopics(DAILY_TOPICS_TEAM_ID)
      if (current !== revision) return
      const latest = mapDailyTopicEdition(payload)
      if (latest.updatedAt !== edition.value.updatedAt) selectedId.value = null
      edition.value = latest
    } catch (error) {
      if (current !== revision) return
      errorMessage.value = error instanceof ApiRequestError && error.statusCode === 404
        ? '后端接口待接入' : extractApiErrorMessage(error, '话题加载失败，请稍后重试')
    } finally {
      if (current === revision) loading.value = false
    }
  }
  onScopeDispose(() => { revision += 1 })
  return { edition, loading, errorMessage, selectedItem, load,
    openReading: (id: string) => { selectedId.value = id }, closeReading: () => { selectedId.value = null } }
}
