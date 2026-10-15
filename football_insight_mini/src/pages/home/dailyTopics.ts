import type { DailyTopicItem } from '../../types/dailyTopic'
import { isRongchengTopic } from '../../utils/dailyTopics'

export * from '../../utils/dailyTopics'

export function homeTopicCategory(item: Pick<DailyTopicItem, 'teamTags'>): string {
  return isRongchengTopic(item) ? '蓉城动态' : '其他动态'
}
