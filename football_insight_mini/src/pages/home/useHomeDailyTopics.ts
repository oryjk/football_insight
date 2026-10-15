import { onScopeDispose, ref } from 'vue'
import { syncTopicHeat } from '../../stores/topicHeat'
import { getDailyTopics } from '../../api/dailyTopics'
import { ApiRequestError, extractApiErrorMessage } from '../../utils/apiError'
import { DAILY_TOPICS_TEAM_ID, emptyDailyTopicEdition, mapDailyTopicEdition } from './dailyTopics'

export function useHomeDailyTopics() {
  const edition = ref(emptyDailyTopicEdition())
  // 首次 onShow 才发起 load；初始为 true，避免先闪一帧「暂无已发布话题」再切到加载态。
  const loading = ref(true)
  const errorMessage = ref('')
  let revision = 0
  async function load() {
    const current = ++revision
    loading.value = true
    errorMessage.value = ''
    try {
      const payload = await getDailyTopics(DAILY_TOPICS_TEAM_ID)
      if (current !== revision) return
      const latest = mapDailyTopicEdition(payload)
      syncTopicHeat(latest.items)
      edition.value = latest
    } catch (error) {
      if (current !== revision) return
      errorMessage.value = error instanceof ApiRequestError && error.statusCode === 404
        ? '话题暂时未能加载' : extractApiErrorMessage(error, '话题加载失败，请稍后重试')
    } finally {
      if (current === revision) loading.value = false
    }
  }
  onScopeDispose(() => { revision += 1 })
  return { edition, loading, errorMessage, load }
}
