import type { DailyTopicItem } from '../../types/dailyTopic'

export * from '../../utils/dailyTopics'

export function homeTopicCategory(item: Pick<DailyTopicItem, 'teamTags'>): string {
  // The edition owner is shared by all clubs; only article tags identify Rongcheng.
  return item.teamTags?.some(tag => tag.teamId === 77680) ? '蓉城动态' : '其他动态'
}
